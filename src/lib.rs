use rand::{RngCore, thread_rng};
use sha2::{Digest, Sha256};
use std::fmt::Debug;

use ed25519_dalek::SigningKey;

/// Controls how fast can a user generate (and examine) a name from the genesis public key.
///
/// It should be small enough that a user can generate a name in couple of second on a mobile phone,
/// in case they got a specially unlucky random name.
///
/// It should be high enough to make it significantly more difficult for an attacker to find
/// another genesis public key and nonce that hashes to the same Name.
const NAME_GENERATION_DIFFICULTY: u8 = 17;
/// Controls how fast can someone timestamp newly generated Names and claim them forever.
///
/// Usually a user would generate the name first, and only publish their SingnedPacket once
/// they find a valid timestamping work for this difficulty.
///
/// The longer this takes, the slower initial publishing may take, but the harder it would be
/// for squatting.
const TIMESTAMPING_DIFFICULTY: u8 = 24;

const NAME_GENERATION_SALT: &str = "mns/name_generation_salt";
const TIMESTAMPING_SALT: &str = "mns/timestamping_salt";

// Claim Proof: Name Generation Work + Timestamp Work + Timestamp Proof
//
// TODO: Adaptive difficulties based on Bitcoin difficulty...
// Timestamping has to happen within a day/week? of generating the Timestamping work...

// TODO: P2P Treap of fresh names pending timestamping... to be timestamped at once ...
// Basically a p2p version of OTS.
//
// TODO: builtin key rotation

pub struct Keypair {
    hasher: Sha256,
    signing_key: SigningKey,
    name_generation_nonce: u64,
    name_hash: [u8; 32],
}

impl Keypair {
    pub fn generate() -> Keypair {
        // TODO: use generate_random() instead
        let mut rng = thread_rng();
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);

        let signing_key = SigningKey::from_bytes(&seed);
        let public_key = signing_key.verifying_key();
        let public_key = public_key.as_bytes();

        let mut name_generation_nonce = 0u64;

        let mut hasher = Sha256::new();
        let name_hash = loop {
            if let Some(name_hash) =
                generate_name_hash(&mut hasher, public_key, name_generation_nonce)
            {
                break name_hash;
            }

            name_generation_nonce += 1;
        };

        Keypair {
            hasher,
            signing_key,
            name_hash,
            name_generation_nonce,
        }
    }

    pub fn genesis(&mut self) -> ([u8; 32], u64, u64) {
        let mut timestamping_nonce = 0;

        loop {
            let timestamping_work = hash_pow(
                &mut self.hasher,
                TIMESTAMPING_SALT,
                &self.name_hash,
                timestamping_nonce,
            );
            if check_pow_target(&timestamping_work, TIMESTAMPING_DIFFICULTY) {
                break;
            };

            timestamping_nonce += 1;
        }

        (
            // TODO: signature over SignedPacket
            self.signing_key.verifying_key().as_bytes().to_owned(),
            self.name_generation_nonce,
            timestamping_nonce,
        )
    }

    pub fn name(&self) -> String {
        let name = &self.name_hash[0..5];

        let input = base32::encode(base32::Alphabet::Z, name);

        let first_part = &input[..4];
        let second_part = &input[4..];

        format!("{}_{}", first_part, second_part)
    }
}

impl Debug for Keypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Keypair ({}.mns.alt)", self.name())
    }
}

fn generate_name_hash(hasher: &mut Sha256, public_key: &[u8; 32], nonce: u64) -> Option<[u8; 32]> {
    let name_genreation_work = hash_pow(hasher, NAME_GENERATION_SALT, public_key, nonce);
    if !check_pow_target(&name_genreation_work, NAME_GENERATION_DIFFICULTY) {
        return None;
    };

    hasher.update(NAME_GENERATION_SALT);
    hasher.update(name_genreation_work);
    // TODO: better type casting.
    let hash = hasher.finalize_reset()[..].try_into().expect("infallible");

    Some(hash)
}

fn hash_pow(hasher: &mut Sha256, salt: &str, base: &[u8; 32], nonce: u64) -> [u8; 32] {
    hasher.update(salt.as_bytes());
    hasher.update(&base);
    hasher.update(nonce.to_le_bytes());
    hasher.finalize_reset()[..].try_into().unwrap()
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
        let mut x = Keypair::generate();

        println!("Generated {x:?} in {} ms", start.elapsed().as_millis());

        let start = Instant::now();
        let genesis = x.genesis();
        println!(
            "Generated the genesis for {x:?} in {} ms",
            start.elapsed().as_millis()
        );
    }
}
