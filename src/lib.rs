use equix::{EquiX, Solution, verify_bytes};
use rand::{RngCore, thread_rng};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sha2::{Digest, Sha256};
use std::fmt::Debug;

use ed25519_dalek::{Signer, SigningKey, Verifier};

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

pub struct Claim {
    owner: Vec<u8>,
    name_generation_proof: ([u8; 16], u64),
}

impl Claim {
    pub fn name(&self) -> String {
        let name = &self.owner[1..5];

        let input = base32::encode(base32::Alphabet::Z, name);

        let first_part = &input[..4];
        let second_part = &input[4..];

        format!("{}_{}", first_part, second_part)
    }
}

impl Debug for Claim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Claim ({}.mns.alt)", self.name())
    }
}

pub fn generate() -> Claim {
    let owner = owner();

    let name_generation_proof = (0u64..u64::MAX)
        .into_par_iter()
        .map(|nonce| {
            attempt_pow(
                NAME_GENERATION_TAG,
                &owner,
                NAME_GENERATION_DIFFICULTY,
                nonce,
            )
            .map(|solution| (solution, nonce))
        })
        .find_any(|result| result.is_some())
        .flatten()
        .expect("find pow");

    Claim {
        owner,
        name_generation_proof,
    }
}

pub fn verify_name(claim: &Claim) -> bool {
    let (solution, nonce) = claim.name_generation_proof;

    let mut challenge = vec![];
    challenge.extend_from_slice(NAME_GENERATION_TAG);
    challenge.extend_from_slice(&claim.owner);
    challenge.extend_from_slice(&nonce.to_be_bytes());

    let mut hasher = Sha256::new();
    hasher.update(&challenge);
    hasher.update(&solution);
    let pow = &hasher.finalize()[..];

    let start = std::time::Instant::now();
    dbg!(start.elapsed());
    let start = std::time::Instant::now();

    if !check_pow_target(&pow, NAME_GENERATION_DIFFICULTY) {
        return false;
    };

    return equix::verify_bytes(&challenge, &solution).is_ok();

    dbg!(start.elapsed());
    let start = std::time::Instant::now();

    dbg!(start.elapsed());
    let start = std::time::Instant::now();
    let solution = &Solution::try_from_bytes(&solution).unwrap();
    dbg!(("solution", start.elapsed()));
    let start = std::time::Instant::now();

    let x = EquiX::new(&challenge).unwrap();
    dbg!(("new", start.elapsed()));
    let start = std::time::Instant::now();

    let x = x.verify(solution).is_ok();
    dbg!(("verify", start.elapsed()));

    x
}

// pub fn timestamp_pow(keypair: &Claim) -> ([u8; 64], u64) {
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
//         .expect("find pow")
// }
//
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

fn attempt_pow(tag: &[u8], data: &[u8], target: u8, nonce: u64) -> Option<[u8; 16]> {
    let mut challenge = vec![];
    challenge.extend_from_slice(tag);
    challenge.extend_from_slice(data);
    challenge.extend_from_slice(&nonce.to_be_bytes());

    if let Ok(Some(solution)) =
        EquiX::new(&challenge).map(|x| x.solve().first().map(|x| x.to_bytes()))
    {
        challenge.extend_from_slice(&solution);

        let pow = hash(&challenge);

        if check_pow_target(&pow, target) {
            return Some(solution);
        };
    };

    None
}

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

fn hash(msg: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(msg);
    hasher.finalize().into()
}

fn owner() -> Vec<u8> {
    // TODO: use generate_random() instead
    let mut rng = thread_rng();
    let mut seed = [0u8; 32];
    rng.fill_bytes(&mut seed);

    let signing_key = SigningKey::from_bytes(&seed);
    let public_key = signing_key.verifying_key();

    let signature = signing_key.sign(NAME_GENERATION_TAG);

    let mut owner = vec![];
    owner.extend_from_slice(public_key.as_bytes());
    owner.extend_from_slice(signature.r_bytes());
    owner.extend_from_slice(signature.s_bytes());

    owner
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[test]
    fn basic() {
        let start = Instant::now();
        println!("init {:?}", start.elapsed());

        let start = Instant::now();
        let claim = generate();

        println!("Generated {claim:?} in {} ms", start.elapsed().as_millis());

        let start = Instant::now();
        assert!(verify_name(&claim));
        println!("Validated {claim:?} claim in {:?}...", start.elapsed());

        // let start = Instant::now();
        // let (timestamping_signature, timestamping_nonce) = timestamp_pow(&keypair);
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
