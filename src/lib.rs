use equix::{EquiX, Solution, verify_bytes};
use rand::{RngCore, thread_rng};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sha2::{Digest, Sha256};
use std::fmt::Debug;

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

mod name;

pub use name::Name;

const DOMAIN_SUFFIX: &str = ".mns.alt";

// TODO: benchmark to get appropriate difficulties

// TODO: make Name generation take as long as the user wants (with minimum),
// by increasing the pow... then in the case of collision, fallback to the timestamp.

/// Controls how fast can a user generate (and examine) a name from the genesis public key.
///
/// It should be small enough that a user can generate a name in couple of second on a mobile phone,
/// in case they got a specially unlucky random name.
///
/// It should be high enough to make it significantly more difficult for an attacker to find
/// another genesis public key and nonce that hashes to the same Name.
const NAME_GENERATION_DIFFICULTY: u8 = 10;
/// Controls how fast can someone timestamp newly generated Names and claim them forever.
///
/// Usually a user would generate the name first, and only publish their SingnedPacket once
/// they find a valid timestamping work for this difficulty.
///
/// The longer this takes, the slower initial publishing may take, but the harder it would be
/// for squatting.
const TIMESTAMPING_DIFFICULTY: u8 = 20;

const NAME_GENERATION_TAG: &[u8; 19] = b"mns/name-generation";
const TIMESTAMPING_TAG: &[u8; 16] = b"mns/timestamping";

// Claim Proof: Name Generation Work + Timestamp Work + Timestamp Proof
//
// TODO: Adaptive difficulties based on Bitcoin difficulty...
// TODO: Timestamping has to happen within a day/week? of generating the Timestamping work...
// This disables optimistic proofs... so maybe not!

// TODO: P2P Treap of fresh names pending timestamping... to be timestamped at once ...
// Basically a p2p version of OTS.
//
// TODO: builtin key rotation

// TODO: versioning?
pub struct Claim {
    public_key: VerifyingKey,
    name_generation_proof: [u8; 88],
}

impl Claim {
    /// Generate an initial claim, doing only enough work to examine the generated [name][Claim::name].
    pub fn generate() -> Self {
        // TODO: use generate_random() instead
        let mut rng = thread_rng();
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);

        let signing_key = SigningKey::from_bytes(&seed);

        Claim::generate_from_signing_key(&signing_key)
    }

    pub fn generate_from_signing_key(signing_key: &SigningKey) -> Claim {
        let public_key = signing_key.verifying_key();

        let name_generation_proof = (0u64..u64::MAX)
            .into_par_iter()
            .map(|nonce| {
                let nonce_bytes = nonce.to_be_bytes();
                let challenge = hash(&[NAME_GENERATION_TAG, &nonce_bytes]);

                let signature = signing_key.sign(&challenge);
                let signature_bytes = signature.to_bytes();

                if let Ok(Some(solution)) =
                    EquiX::new(&challenge).map(|x| x.solve().first().map(|x| x.to_bytes()))
                {
                    let pow = hash(&[
                        NAME_GENERATION_TAG,
                        &nonce_bytes,
                        &signature_bytes,
                        &solution,
                    ]);

                    if check_pow_target(&pow, NAME_GENERATION_DIFFICULTY) {
                        let mut result = [0_u8; 88];
                        result[0..8].copy_from_slice(&nonce_bytes);
                        result[8..72].copy_from_slice(&signature_bytes);
                        result[72..].copy_from_slice(&solution);

                        return Some(result);
                    };
                };

                None
            })
            .find_any(|result| result.is_some())
            .flatten()
            .expect("find pow");

        Claim {
            public_key,
            name_generation_proof,
        }
    }

    pub fn name(&self) -> Name {
        hash(&[&self.name_generation_proof])[..5]
            .try_into()
            .expect("prefix")
    }

    // pub fn complete(&mut self) {
    //     (0u64..u64::MAX)
    //         .into_par_iter()
    //         .map(|nonce| {
    //             attempt_pow(
    //                 TIMESTAMPING_TAG,
    //                 &keypair.signing_key,
    //                 TIMESTAMPING_DIFFICULTY,
    //                 nonce,
    //             )
    //             .map(|signature| (signature, nonce))
    //         })
    //         .find_any(|result| result.is_some())
    //         .flatten()
    // }

    pub fn verify_name(&self) -> bool {
        let nonce_bytes = &self.name_generation_proof[0..8];
        let challenge = hash(&[NAME_GENERATION_TAG, &nonce_bytes]);

        let signature_bytes = &self.name_generation_proof[8..72];
        let solution_bytes = &self.name_generation_proof[72..];

        // Cheaper verification first

        let pow = hash(&[
            NAME_GENERATION_TAG,
            &nonce_bytes,
            &signature_bytes,
            &solution_bytes,
        ]);

        if !check_pow_target(&pow, NAME_GENERATION_DIFFICULTY) {
            return false;
        };

        if !self
            .public_key
            .verify(
                &challenge,
                &Signature::from_bytes(signature_bytes.try_into().unwrap()),
            )
            .is_ok()
        {
            return false;
        };

        let solution: [u8; 16] = self.name_generation_proof[72..].try_into().unwrap();

        let equix = EquiX::new(&challenge).unwrap();

        equix
            .verify(&Solution::try_from_bytes(&solution).unwrap())
            .is_ok()
    }
}

impl Debug for Claim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Claim ({}{DOMAIN_SUFFIX})", self.name())
    }
}

// pub fn verify_timestamp_pow(
//     keypair: &Claim,
//     timestamp_signature: &[u8; 64],
//     timestamping_nonce: u64,
// ) -> bool {
//     let mut hasher = Sha256::new();
//
//     if !check_powsig_signature(&mut hasher, TIMESTAMPING_DIFFICULTY, timestamp_signature) {
//         return false;
//     }
//
//     let mut msg = vec![];
//     msg.extend_from_slice(TIMESTAMPING_TAG);
//     msg.extend_from_slice(&timestamping_nonce.to_be_bytes());
//
//     if keypair
//         .signing_key
//         .verifying_key()
//         .verify(&msg, &timestamp_signature.try_into().unwrap())
//         .is_err()
//     {
//         return false;
//     }
//
//     Self::verify_name(keypair)
// }

fn check_pow_target(hash: &[u8], required_zero_bits: u8) -> bool {
    let mut zero_bits = 0u8;

    for &byte in hash {
        if byte == 0 {
            zero_bits += 8;
        } else {
            zero_bits += byte.leading_zeros() as u8;
            break;
        }
    }

    zero_bits >= required_zero_bits
}

fn hash(msgs: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for msg in msgs {
        hasher.update(msg);
    }
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[test]
    fn basic() {
        let start = Instant::now();

        println!("Generating a random name..");
        let claim = Claim::generate();
        println!("Generated {claim:?} in {} ms", start.elapsed().as_millis());

        let start = Instant::now();
        println!("Verifying the claim on a {}..", claim.name());
        assert!(claim.verify_name());
        println!("Validated {claim:?} claim in {:?}...", start.elapsed());

        // let start = Instant::now();
        // let (timestamping_signature, timestamping_nonce) = claim.complete(&keypair);
        // println!(
        //     "Generated the timestamp PoW for {keypair:?} in {} ms",
        //     start.elapsed().as_millis()
        // );
        //
        // let start = Instant::now();
        // assert!(verify_timestamp_pow(
        //     &keypair,
        //     &timestamping_signature,
        //     timestamping_nonce
        // ));
        // println!(
        //     "Validated {keypair:?} timestamping pow in {:?}...",
        //     start.elapsed()
        // );
    }
}
