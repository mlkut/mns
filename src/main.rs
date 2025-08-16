//! Using Argon2 to limit generating keys that hashes to an ID

use argon2::{Algorithm, Argon2, Params, Version};
use ed25519_dalek::SigningKey;
use rand::{RngCore, thread_rng};
use sha2::{Digest, Sha256};
use std::time::Instant;

/// Default Argon2 configuration - moderate difficulty for ID generation
fn get_default_argon2() -> Argon2<'static> {
    // Minimal parameters
    let params = Params::new(
        32,   // memory cost (32 KB)
        3,    // time cost (3 iterations)
        1,    // parallelism (1 thread)
        None, // output length (None = default 32 bytes)
    )
    .unwrap();

    // Argon2::new(Algorithm::default(), Version::default(), params)
    Argon2::default()
}

/// Generate Argon2 hash of public key (this becomes the basis for ID)
fn argon2_hash_pubkey(public_key: &[u8]) -> [u8; 32] {
    let argon2 = get_default_argon2();
    let salt = b"malakut_m.alt_id_system_v1_salt_32b"; // Exactly 32 bytes for salt

    let mut output = [0u8; 32];
    argon2
        .hash_password_into(public_key, salt, &mut output)
        .expect("Argon2 hash failed");

    output
}

/// Extract ID from Argon2 hash (first 5 bytes)
fn extract_id(argon2_hash: &[u8; 32]) -> [u8; 5] {
    let mut id = [0u8; 5];
    id.copy_from_slice(&argon2_hash[0..5]);
    id
}

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

/// Complete user workflow: generate keypair, find nonce, create ID
fn user_create_id(pow_difficulty: u8) -> (SigningKey, [u8; 5], u64, u64) {
    println!(
        "User creating ID with {} leading zero bits PoW...",
        pow_difficulty
    );
    let start_total = Instant::now();

    // Step 1: Generate keypair (fast)
    let mut rng = thread_rng();
    let mut seed = [0u8; 32];
    rng.fill_bytes(&mut seed);
    let signing_key = SigningKey::from_bytes(&seed);
    let public_key = signing_key.verifying_key();

    // Step 2: Generate Argon2 hash of public key (expensive, but done once)
    println!("  Computing Argon2 hash of public key...");
    let start_argon2 = Instant::now();
    let argon2_hash = argon2_hash_pubkey(public_key.as_bytes());
    let argon2_time = start_argon2.elapsed().as_millis() as u64;
    println!("  Argon2 hash completed in {}ms", argon2_time);

    // Step 3: Extract ID from Argon2 hash
    let id = extract_id(&argon2_hash);
    println!("  Generated ID: {}", hex::encode(id));

    // Step 4: Find nonce for PoW (hash(nonce || argon2_hash) < target)
    println!("  Finding PoW nonce...");
    let start_pow = Instant::now();
    let mut nonce = 0u64;

    loop {
        // Compute hash(nonce || argon2_hash)
        let mut hasher = Sha256::new();
        hasher.update(nonce.to_le_bytes());
        hasher.update(&argon2_hash);
        let pow_hash = hasher.finalize();

        if check_pow_target(&pow_hash, pow_difficulty) {
            let pow_time = start_pow.elapsed().as_millis() as u64;
            let total_time = start_total.elapsed().as_millis() as u64;

            println!("  PoW nonce found: {} in {}ms", nonce, pow_time);
            println!(
                "  Total user time: {}ms (Argon2: {}ms, PoW: {}ms)",
                total_time, argon2_time, pow_time
            );

            return (signing_key, id, nonce, total_time);
        }

        nonce += 1;

        if nonce % 50000 == 0 {
            let elapsed = start_pow.elapsed().as_secs();
            println!("    PoW attempt {}, elapsed: {}s", nonce, elapsed);
        }
    }
}

/// Verifier workflow: given public key, ID, and nonce, verify the claim
fn verify_id(
    public_key: &[u8],
    claimed_id: [u8; 5],
    nonce: u64,
    pow_difficulty: u8,
) -> (bool, u64) {
    println!(
        "Verifying ID {} with nonce {}...",
        hex::encode(claimed_id),
        nonce
    );
    let start = Instant::now();

    // Step 1: Compute Argon2 hash of public key (expensive!)
    let argon2_hash = argon2_hash_pubkey(public_key);

    // Step 2: Check if ID matches first 5 bytes of Argon2 hash
    let computed_id = extract_id(&argon2_hash);
    if computed_id != claimed_id {
        let duration = start.elapsed().as_millis() as u64;
        println!(
            "  ID verification FAILED: computed {}, claimed {}",
            hex::encode(computed_id),
            hex::encode(claimed_id)
        );
        return (false, duration);
    }

    // Step 3: Verify PoW: hash(nonce || argon2_hash) meets target
    let mut hasher = Sha256::new();
    hasher.update(nonce.to_le_bytes());
    hasher.update(&argon2_hash);
    let pow_hash = hasher.finalize();

    let pow_valid = check_pow_target(&pow_hash, pow_difficulty);
    let duration = start.elapsed().as_millis() as u64;

    if pow_valid {
        println!("  Verification SUCCESS in {}ms", duration);
    } else {
        println!("  PoW verification FAILED in {}ms", duration);
    }

    (pow_valid, duration)
}

/// Attacker workflow: find different public key that generates same ID and valid PoW
fn attacker_find_collision(
    target_id: [u8; 5],
    pow_difficulty: u8,
    max_attempts: u64,
) -> Option<(u64, u64)> {
    println!(
        "Attacker searching for collision with ID: {}",
        hex::encode(target_id)
    );
    println!(
        "Required: same 5-byte ID + valid {}-bit PoW",
        pow_difficulty
    );
    let start = Instant::now();

    let mut rng = thread_rng();
    let mut valid_ids_found = 0u64;
    let mut valid_pows_found = 0u64;

    for attempt in 0..max_attempts {
        if attempt % 10 == 0 && attempt > 0 {
            let elapsed = start.elapsed().as_secs();
            println!(
                "  Attempt {}: {}s elapsed, {} matching IDs, {} valid PoWs",
                attempt, elapsed, valid_ids_found, valid_pows_found
            );
        }

        // Generate different keypair
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);
        let signing_key = SigningKey::from_bytes(&seed);
        let public_key = signing_key.verifying_key();

        // Compute expensive Argon2 hash
        let argon2_hash = argon2_hash_pubkey(public_key.as_bytes());
        let computed_id = extract_id(&argon2_hash);

        // Check if ID matches target
        if computed_id == target_id {
            valid_ids_found += 1;
            println!("    MATCHING ID FOUND! Attempt {}", attempt);

            // Now try to find a valid PoW nonce
            let mut nonce = 0u64;
            let max_nonce_attempts = 1000000; // Limit nonce search

            for _ in 0..max_nonce_attempts {
                let mut hasher = Sha256::new();
                hasher.update(nonce.to_le_bytes());
                hasher.update(&argon2_hash);
                let pow_hash = hasher.finalize();

                if check_pow_target(&pow_hash, pow_difficulty) {
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
                }

                nonce += 1;
            }

            println!(
                "    Found matching ID but no valid PoW within {} nonce attempts",
                max_nonce_attempts
            );
            valid_pows_found += 1;
        }
    }

    let duration = start.elapsed().as_millis() as u64;
    println!(
        "  Attack failed after {} attempts in {}ms",
        max_attempts, duration
    );
    println!(
        "  Found {} matching IDs, {} with valid PoW",
        valid_ids_found, valid_pows_found
    );
    None
}

fn main() {
    println!("M.alt ID System Security Analysis");
    println!("================================\n");

    // Test with different PoW difficulties
    for pow_difficulty in [24] {
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
            verify_id(public_key.as_bytes(), user_id, nonce, pow_difficulty);

        if !is_valid {
            println!("ERROR: Verification failed for legitimate user!");
            continue;
        }

        println!();

        // 3. ATTACKER ATTEMPTS COLLISION
        println!("--- Attacker Analysis ---");
        let max_attacker_attempts = 50; // Limited due to Argon2 cost
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

