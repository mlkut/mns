use rand::{RngCore, thread_rng};
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
const NAME_GENERATION_DIFFICULTY: u8 = 16;
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

pub struct Mns {
    hasher: Sha256,
}

impl Mns {
    pub fn init() -> Self {
        Self {
            hasher: Sha256::new(),
        }
    }

    pub fn generate(&mut self) -> Claim {
        // TODO: use generate_random() instead
        let mut rng = thread_rng();
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);

        let signing_key = SigningKey::from_bytes(&seed);

        let mut nonce: u64 = 0;

        let signature = loop {
            if let Some(signature) = sigpow(
                &mut self.hasher,
                NAME_GENERATION_TAG,
                &signing_key,
                NAME_GENERATION_DIFFICULTY,
                nonce,
            ) {
                break signature;
            };

            nonce += 1;
        };

        Claim {
            signing_key,
            signature,
            name_generation_nonce: nonce,
        }
    }

    pub fn verify_name(&mut self, keypair: &Claim) -> bool {
        if !check_powsig_signature(
            &mut self.hasher,
            NAME_GENERATION_DIFFICULTY,
            &keypair.signature,
        ) {
            return false;
        }

        let mut msg = vec![];
        msg.extend_from_slice(NAME_GENERATION_TAG);
        msg.extend_from_slice(&keypair.name_generation_nonce.to_be_bytes());

        keypair
            .signing_key
            .verifying_key()
            .verify(&msg, &keypair.signature.try_into().unwrap())
            .is_ok()
    }

    pub fn timestamp_pow(&mut self, keypair: &Claim) -> ([u8; 64], u64) {
        let mut nonce: u64 = 0;

        let signature = loop {
            if let Some(signature) = sigpow(
                &mut self.hasher,
                TIMESTAMPING_TAG,
                &keypair.signing_key,
                TIMESTAMPING_DIFFICULTY,
                nonce,
            ) {
                break signature;
            };

            nonce += 1;
        };

        (signature, nonce)
    }

    pub fn verify_timestamp_pow(
        &mut self,
        keypair: &Claim,
        timestamp_signature: &[u8; 64],
        timestamping_nonce: u64,
    ) -> bool {
        if !check_powsig_signature(
            &mut self.hasher,
            TIMESTAMPING_DIFFICULTY,
            timestamp_signature,
        ) {
            return false;
        }

        let mut msg = vec![];
        msg.extend_from_slice(TIMESTAMPING_TAG);
        msg.extend_from_slice(&timestamping_nonce.to_be_bytes());

        if keypair
            .signing_key
            .verifying_key()
            .verify(&msg, &timestamp_signature.try_into().unwrap())
            .is_err()
        {
            return false;
        }

        self.verify_name(keypair)
    }
}

pub struct Claim {
    signing_key: SigningKey,
    name_generation_nonce: u64,
    signature: [u8; 64],
}

impl Claim {
    pub fn name(&self) -> String {
        let name = &self.signature[0..5];

        let input = base32::encode(base32::Alphabet::Z, name);

        let first_part = &input[..4];
        let second_part = &input[4..];

        format!("{}_{}", first_part, second_part)
    }

    pub fn public_key(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }
}

impl Debug for Claim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Claim ({}.mns.alt)", self.name())
    }
}

fn sigpow(
    hasher: &mut Sha256,
    tag: &[u8],
    signing_key: &SigningKey,
    target: u8,
    nonce: u64,
) -> Option<[u8; 64]> {
    let mut msg = vec![];
    msg.extend_from_slice(tag);
    msg.extend_from_slice(&nonce.to_be_bytes());

    let signature = signing_key.sign(&msg).to_bytes();

    if check_powsig_signature(hasher, target, &signature) {
        return Some(signature);
    };

    None
}

fn check_powsig_signature(hasher: &mut Sha256, target: u8, signature: &[u8; 64]) -> bool {
    hasher.update(&signature);
    // TODO: better type casting
    let pow = &hasher.finalize_reset()[..];

    if check_pow_target(pow, target) {
        return true;
    };

    return false;
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

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[test]
    fn basic() {
        let start = Instant::now();
        let mut mns = Mns::init();
        println!("init {:?}", start.elapsed());

        let start = Instant::now();
        let keypair = mns.generate();

        println!(
            "Generated {keypair:?} in {} ms",
            start.elapsed().as_millis()
        );

        let start = Instant::now();
        assert!(mns.verify_name(&keypair));
        println!("Validated {keypair:?} name in {:?}...", start.elapsed());

        let start = Instant::now();
        let (timestamping_signature, timestamping_nonce) = mns.timestamp_pow(&keypair);
        println!(
            "Generated the timestamp PoW for {keypair:?} in {} ms",
            start.elapsed().as_millis()
        );

        let start = Instant::now();
        assert!(mns.verify_timestamp_pow(&keypair, &timestamping_signature, timestamping_nonce));
        println!(
            "Validated {keypair:?} timestamping pow in {:?}...",
            start.elapsed()
        );
    }
}
