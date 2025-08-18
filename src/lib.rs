use rand::{RngCore, thread_rng};
use randomx_rs::{RandomXCache, RandomXFlag, RandomXVM};
use std::fmt::Debug;

use ed25519_dalek::SigningKey;

// TODO: benchmark to get appropriate difficulties

/// Controls how fast can a user generate (and examine) a name from the genesis public key.
///
/// It should be small enough that a user can generate a name in couple of second on a mobile phone,
/// in case they got a specially unlucky random name.
///
/// It should be high enough to make it significantly more difficult for an attacker to find
/// another genesis public key and nonce that hashes to the same Name.
const NAME_GENERATION_DIFFICULTY: u8 = 8;
/// Controls how fast can someone timestamp newly generated Names and claim them forever.
///
/// Usually a user would generate the name first, and only publish their SingnedPacket once
/// they find a valid timestamping work for this difficulty.
///
/// The longer this takes, the slower initial publishing may take, but the harder it would be
/// for squatting.
const TIMESTAMPING_DIFFICULTY: u8 = 16;

const RANDOMX_KEY: &str = "mns/randomx/key";
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
    vm: RandomXVM,
}

impl Mns {
    pub fn init() -> Self {
        // TODO: check that this works the same everywhere...
        let flags = RandomXFlag::get_recommended_flags();
        let cache = RandomXCache::new(flags, RANDOMX_KEY.as_bytes()).unwrap();
        let vm = RandomXVM::new(flags, Some(cache.clone()), None).unwrap();

        Self { vm }
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
            if let Some(name_hash) = generate_name_hash(&self.vm, public_key, name_generation_nonce)
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
            &self.vm,
            &keypair.public_key(),
            keypair.name_generation_nonce,
        );

        if let Some(expected_name_hash) = expected_name_hash {
            return keypair.name_hash == expected_name_hash;
        }

        false
    }

    pub fn genesis(&self, keypair: &Keypair) -> u64 {
        let mut timestamping_nonce = 0_u64;
        loop {
            let timestamping_work = hash_pow(
                &self.vm,
                TIMESTAMPING_SALT,
                &keypair.name_hash,
                timestamping_nonce,
            );
            if check_pow_target(&timestamping_work, TIMESTAMPING_DIFFICULTY) {
                break;
            };

            timestamping_nonce += 1;
        }

        timestamping_nonce
    }

    pub fn verify_timestamp_pow(&self, keypair: &Keypair, timestamping_nonce: u64) -> bool {
        let expected_name_hash = generate_name_hash(
            &self.vm,
            &keypair.public_key(),
            keypair.name_generation_nonce,
        );

        if let Some(expected_name_hash) = expected_name_hash {
            if keypair.name_hash != expected_name_hash {
                return false;
            }
        }

        check_pow_target(
            &hash_pow(
                &self.vm,
                TIMESTAMPING_SALT,
                &keypair.name_hash,
                timestamping_nonce,
            ),
            TIMESTAMPING_DIFFICULTY,
        )
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

fn generate_name_hash(vm: &RandomXVM, public_key: &[u8; 32], nonce: u64) -> Option<[u8; 32]> {
    let name_genreation_work = hash_pow(vm, NAME_GENERATION_SALT, public_key, nonce);
    if !check_pow_target(&name_genreation_work, NAME_GENERATION_DIFFICULTY) {
        return None;
    };

    // TODO: better type casting.
    let hash = vm
        .calculate_hash(&name_genreation_work)
        .unwrap()
        .try_into()
        .unwrap();

    Some(hash)
}

fn hash_pow(vm: &RandomXVM, salt: &str, base: &[u8; 32], nonce: u64) -> [u8; 32] {
    let mut bytes = vec![];
    bytes.extend_from_slice(salt.as_bytes());
    bytes.extend_from_slice(base);
    bytes.extend_from_slice(&nonce.to_be_bytes());

    // TODO: better type casting.
    vm.calculate_hash(&bytes).unwrap().try_into().unwrap()
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
        let timestamping_nonce = mns.genesis(&keypair);
        println!(
            "Generated the genesis for {keypair:?} in {} ms",
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
