use rand::{RngCore, thread_rng};
use sha2::{Digest, Sha256};
use std::{fmt::Debug, time::Instant};

use ed25519_dalek::SigningKey;

// TODO: decide on the exact number, or make it adaptive based on something?
const TARGET_DIFFICULTY: u8 = 20;

pub struct Keypair {
    signing_key: SigningKey,
    nonce: u64,
    name: [u8; 5],
}

impl Keypair {
    fn generate() -> Keypair {
        // TODO: use generate_random() instead
        let mut rng = thread_rng();
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);

        let signing_key = SigningKey::from_bytes(&seed);
        let public_key = signing_key.verifying_key();
        let public_key = public_key.as_bytes();

        let mut nonce = 0u64;

        let name;

        let start_pow = Instant::now();
        let mut hasher = Sha256::new();
        loop {
            if let Some(_name) = calculate_name(&mut hasher, public_key, nonce, TARGET_DIFFICULTY) {
                name = _name;

                break;
            }

            nonce += 1;

            if nonce % 50000 == 0 {
                let elapsed = start_pow.elapsed().as_secs();
                tracing::debug!("PoW attempt {}, elapsed: {}s", nonce, elapsed);
            }
        }

        Keypair {
            signing_key,
            name,
            nonce,
        }
    }

    pub fn name(&self) -> String {
        let input = base32::encode(base32::Alphabet::Z, &self.name);

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

fn calculate_name(
    hasher: &mut Sha256,
    public_key: &[u8; 32],
    nonce: u64,
    target_difficulty: u8,
) -> Option<[u8; 5]> {
    let pow_hash = hash_pow(public_key, nonce);
    if !check_pow_target(&pow_hash, target_difficulty) {
        return None;
    };

    hasher.update("mns.alt");
    hasher.update(pow_hash);
    let hash = &hasher.finalize_reset()[..];

    let mut id = [0u8; 5];
    id.copy_from_slice(&hash[0..5]);

    Some(id)
}

fn hash_pow(public_key: &[u8; 32], nonce: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(nonce.to_le_bytes());
    hasher.update(&public_key);
    hasher.finalize()[..].try_into().unwrap()
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
    use super::*;

    #[test]
    fn basic() {
        let x = Keypair::generate();

        println!("Generated {x:?}");
    }
}
