//! Use PoW then hash.

use ed25519_dalek::SigningKey;
use rand::{RngCore, thread_rng};
use sha2::{Digest, Sha256};
use std::time::Instant;

/// Check if hash meets PoW target (has required leading zero bits)
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

fn hash_pow(public_key: &[u8; 32], nonce: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(nonce.to_le_bytes());
    hasher.update(&public_key);
    hasher.finalize()[..].try_into().unwrap()
}

/// Complete user workflow: generate keypair, find nonce, create ID
fn user_create_id(target_difficulty: u8) -> (SigningKey, [u8; 5], u64, u64) {
    println!(
        "User creating ID with {} leading zero bits PoW...",
        target_difficulty
    );
    let start_total = Instant::now();

    // Step 1: Generate keypair (fast)
    let mut rng = thread_rng();
    let mut seed = [0u8; 32];
    rng.fill_bytes(&mut seed);
    let signing_key = SigningKey::from_bytes(&seed);
    let public_key = signing_key.verifying_key();
    let public_key = public_key.as_bytes();

    println!("  Finding PoW nonce...");
    let start_pow = Instant::now();
    let mut nonce = 0u64;

    let id;

    loop {
        if let Some(_id) = calculate_id(public_key, nonce, target_difficulty) {
            id = _id;

            break;
        }

        nonce += 1;

        if nonce % 50000 == 0 {
            let elapsed = start_pow.elapsed().as_secs();
            println!("    PoW attempt {}, elapsed: {}s", nonce, elapsed);
        }
    }

    println!("  Generated ID: {}", hex::encode(id));

    let total_time = start_total.elapsed().as_millis() as u64;

    println!("  Total user time: {total_time}ms");

    (signing_key, id, nonce, total_time)
}

fn calculate_id(public_key: &[u8; 32], nonce: u64, target_difficulty: u8) -> Option<[u8; 5]> {
    let pow_hash = hash_pow(public_key, nonce);
    if !check_pow_target(&pow_hash, target_difficulty) {
        return None;
    };

    let mut hasher = Sha256::new();
    hasher.update("mns.alt");
    hasher.update(pow_hash);
    let hash = &hasher.finalize()[..];

    let mut id = [0u8; 5];
    id.copy_from_slice(&hash[0..5]);

    Some(id)
}

/// Verifier workflow: given public key, ID, and nonce, verify the claim
fn verify_id(
    public_key: &[u8; 32],
    nonce: u64,
    claimed_id: [u8; 5],
    pow_difficulty: u8,
) -> (bool, u64) {
    println!(
        "Verifying ID {} with nonce {}...",
        hex::encode(claimed_id),
        nonce
    );
    let start = Instant::now();

    let valid = calculate_id(public_key, nonce, pow_difficulty)
        .map(|calculate_id| calculate_id == claimed_id)
        .unwrap_or_default();

    let duration = start.elapsed().as_millis() as u64;

    if valid {
        println!("  ID verification SUCCEEDED IN {duration} ms");
        return (true, duration);
    }

    println!("  ID verification FAILED");

    (false, duration)
}

/// Attacker workflow: find different public key that generates same ID and valid PoW
fn attacker_find_collision(
    target_id: [u8; 5],
    target_difficulty: u8,
    max_attempts: u64,
) -> Option<(u64, u64)> {
    println!(
        "Attacker searching for collision with ID: {}",
        hex::encode(target_id)
    );
    println!(
        "Required: same 5-byte ID + valid {}-bit PoW",
        target_difficulty
    );
    let start = Instant::now();

    let mut rng = thread_rng();

    for attempt in 0..max_attempts {
        if attempt % 10 == 0 && attempt > 0 {
            let elapsed = start.elapsed().as_secs();
            println!("  Attempt {}: {}s elapsed", attempt, elapsed);
        }

        // Generate different keypair
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);
        let signing_key = SigningKey::from_bytes(&seed);
        let public_key = signing_key.verifying_key();
        let public_key = public_key.as_bytes();

        // Now try to find a valid PoW nonce
        let mut nonce = 0u64;
        let max_nonce_attempts = 1000000; // Limit nonce search

        for _ in 0..max_nonce_attempts {
            if let Some(computed_id) = calculate_id(public_key, nonce, target_difficulty) {
                if computed_id == target_id {
                    let duration = start.elapsed().as_millis() as u64;
                    println!("    FULL COLLISION FOUND!");
                    println!("    Matching ID: {}", hex::encode(computed_id));
                    println!("    Valid nonce: {}", nonce);
                    println!(
                        "    Time taken: {}ms after {} attempts",
                        duration,
                        attempt + 1
                    );

                    return Some((duration, attempt + 1));
                } else {
                    // println!(
                    //     "  Found valid PoW but not the same ID {}: {}s elapsed, {computed_id:?} != {target_id:?}",
                    //     attempt,
                    //     start.elapsed().as_secs()
                    // );
                }
            };

            nonce += 1;
        }

        // Check if ID matches target
    }

    let duration = start.elapsed().as_millis() as u64;
    println!(
        "  Attack failed after {} attempts in {}ms",
        max_attempts, duration
    );
    None
}

fn main() {
    println!("M.alt ID System Security Analysis");
    println!("================================\n");

    // Test with different PoW difficulties
    for pow_difficulty in [30] {
        println!(
            "=== PoW Difficulty: {} leading zero bits ===\n",
            pow_difficulty
        );

        // 1. USER CREATES ID
        let (signing_key, user_id, nonce, user_time) = user_create_id(pow_difficulty);
        let public_key = signing_key.verifying_key();

        println!();

        // 2. VERIFIER CHECKS ID
        let (is_valid, verification_time) =
            verify_id(public_key.as_bytes(), nonce, user_id, pow_difficulty);

        if !is_valid {
            println!("ERROR: Verification failed for legitimate user!");
            continue;
        }

        println!();

        // 3. ATTACKER ATTEMPTS COLLISION
        println!("--- Attacker Analysis ---");
        let max_attacker_attempts = 2_000_000; // Limited due to Argon2 cost
        let collision_result =
            attacker_find_collision(user_id, pow_difficulty, max_attacker_attempts);

        // 4. ANALYSIS
        println!("\n--- Performance Analysis ---");
        println!("User creation time: {}minutes", user_time / 60);
        println!("Verification time: {}ms", verification_time);
        println!(
            "Verification cost: Same as user's Argon2 time (~{}ms)",
            verification_time
        );

        if let Some((attack_time, attack_attempts)) = collision_result {
            println!(
                "Attacker succeeded in {}ms after {} attempts",
                attack_time, attack_attempts
            );
            println!(
                "Average time per attack attempt: {}ms",
                attack_time / attack_attempts
            );
        } else {
            println!("Attacker failed after {} attempts", max_attacker_attempts);

            // Calculate theoretical security
            let expected_id_attempts = 2u64.pow(40 / 2);
            let expected_pow_attempts = 2u64.pow(pow_difficulty as u32);

            println!(
                "Expected attempts for ID collision: sqrt(~2^40) = {}",
                expected_id_attempts
            );
            println!(
                "Expected time for ID collision: sqrt(~2^40) * argon hash time= {} hours",
                expected_id_attempts * verification_time / 1000 / 60 / 60
            );
            println!(
                "Expected attempts for PoW: ~2^{} = {}",
                pow_difficulty, expected_pow_attempts
            );
            println!("Each attempt requires ~{}ms of Argon2", verification_time);

            let total_expected_time = 2u64.pow(40) * verification_time / 1000 / 60 / 60 / 24 / 365;
            println!("Estimated time to break: ~{} years", total_expected_time);
        }

        println!("\n{}\n", "=".repeat(60));
    }
}

