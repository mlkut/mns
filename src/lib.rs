use argon2::Argon2;
use rand::{RngCore, thread_rng};
use std::fmt::Debug;

use ed25519_dalek::SigningKey;

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
const NAME_GENERATION_DIFFICULTY: u8 = 5;
/// Controls how fast can someone timestamp newly generated Names and claim them forever.
///
/// Usually a user would generate the name first, and only publish their SingnedPacket once
/// they find a valid timestamping work for this difficulty.
///
/// The longer this takes, the slower initial publishing may take, but the harder it would be
/// for squatting.
const TIMESTAMPING_DIFFICULTY: u8 = 16;

const NAME_GENERATION_SALT: &str = "mns/name-generation_salt";
const TIMESTAMPING_SALT: &str = "mns/timestamping_salt";

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
    argon2: Argon2<'static>,
}

impl Mns {
    pub fn init() -> Self {
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(
                2 * 1024, // Faster verification and hopefully still GPU resistant!
                1,        // Faster verification
                argon2::Params::DEFAULT_P_COST.into(),
                argon2::Params::DEFAULT_OUTPUT_LEN.into(),
            )
            .unwrap(),
        );

        Self { argon2 }
    }

    pub fn generate(&self) -> Keypair {
        // TODO: use generate_random() instead
        let mut rng = thread_rng();
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);

        let signing_key = SigningKey::from_bytes(&seed);
        let public_key = signing_key.verifying_key();
        let public_key = public_key.as_bytes();

        let mut name_generation_nonce = 0u64;

        let name_hash = loop {
            if let Some(name_hash) =
                generate_name_hash(&self.argon2, public_key, name_generation_nonce)
            {
                break name_hash;
            }

            name_generation_nonce += 1;
        };

        Keypair {
            signing_key,
            name_hash,
            name_generation_nonce,
        }
    }

    pub fn verify_name(&self, keypair: &Keypair) -> bool {
        let expected_name_hash = generate_name_hash(
            &self.argon2,
            &keypair.public_key(),
            keypair.name_generation_nonce,
        );

        if let Some(expected_name_hash) = expected_name_hash {
            return keypair.name_hash == expected_name_hash;
        }

        false
    }

    pub fn timestamp_pow(&self, keypair: &Keypair) -> u64 {
        let mut timestamping_nonce = 0_u64;
        loop {
            let timestamping_pow = hash_pow(
                &self.argon2,
                TIMESTAMPING_SALT,
                &keypair.name_hash,
                timestamping_nonce,
            );
            if check_pow_target(&timestamping_pow, TIMESTAMPING_DIFFICULTY) {
                break;
            };

            timestamping_nonce += 1;
        }

        timestamping_nonce
    }

    pub fn verify_timestamp_pow(&self, keypair: &Keypair, timestamping_nonce: u64) -> bool {
        let expected_name_hash = generate_name_hash(
            &self.argon2,
            &keypair.public_key(),
            keypair.name_generation_nonce,
        );

        if let Some(expected_name_hash) = expected_name_hash {
            if keypair.name_hash != expected_name_hash {
                return false;
            }
        }

        let timestamping_pow = &hash_pow(
            &self.argon2,
            TIMESTAMPING_SALT,
            &keypair.name_hash,
            timestamping_nonce,
        );

        check_pow_target(timestamping_pow, TIMESTAMPING_DIFFICULTY)
    }
}

pub struct Keypair {
    signing_key: SigningKey,
    name_generation_nonce: u64,
    name_hash: [u8; 32],
}

impl Keypair {
    pub fn name(&self) -> String {
        let name = &self.name_hash[0..5];

        let input = base32::encode(base32::Alphabet::Z, name);

        let first_part = &input[..4];
        let second_part = &input[4..];

        format!("{}_{}", first_part, second_part)
    }

    pub fn public_key(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }
}

impl Debug for Keypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Keypair ({}.mns.alt)", self.name())
    }
}

fn generate_name_hash(argon2: &Argon2, public_key: &[u8; 32], nonce: u64) -> Option<[u8; 32]> {
    let name_genreation_work = hash_pow(argon2, NAME_GENERATION_SALT, public_key, nonce);
    if !check_pow_target(&name_genreation_work, NAME_GENERATION_DIFFICULTY) {
        return None;
    };

    let mut hash = [0u8; 32];
    argon2
        .hash_password_into(public_key, NAME_GENERATION_SALT.as_bytes(), &mut hash)
        .expect("Argon2 hash failed");

    Some(hash)
}

fn hash_pow(argon2: &Argon2, salt: &str, base: &[u8; 32], nonce: u64) -> [u8; 32] {
    let mut bytes = vec![];
    bytes.extend_from_slice(salt.as_bytes());
    bytes.extend_from_slice(base);
    bytes.extend_from_slice(&nonce.to_be_bytes());

    let mut hash = [0u8; 32];
    argon2
        .hash_password_into(&bytes, salt.as_bytes(), &mut hash)
        .expect("Argon2 hash failed");

    hash
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
        let mns = Mns::init();
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
        let timestamping_nonce = mns.timestamp_pow(&keypair);
        println!(
            "Generated the timestamp PoW for {keypair:?} in {} ms",
            start.elapsed().as_millis()
        );

        let start = Instant::now();
        assert!(mns.verify_timestamp_pow(&keypair, timestamping_nonce));
        println!(
            "Validated {keypair:?} timestamping pow in {:?}...",
            start.elapsed()
        );
    }
}
