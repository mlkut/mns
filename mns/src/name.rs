use std::{fmt::Display, str::FromStr};

// No `C` or `Q` because they sounds like `K`
// No `H` at the end of a syllable because it is hard to pronounce clearly.
// No `X` anywhere except last consonant in the word.

const FIRST_CONSONANTS: &[u8; 16] = b"dmbwslhfrtpnjzvk";
const SECOND_CONSONANTS: &[u8; 16] = b"zrnmtgdskblpvfjx";

const VOWELS: &[u8; 4] = b"oaiu";

const THIRD_CONSONANTS: &[u8; 16] = b"znbvsplfdrhtmkgj";
const FOURTH_CONSONANTS: &[u8; 16] = b"dksvrtlnpxbgmfzj";

/// 2^64 / phi (the golden ratio). Used for Fibonacci Hashing.
/// This constant is odd, which ensures the transformation is a bijection.
const PHI_64: u64 = 0x9e3779b97f4a7c15;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name([u8; 5]);

impl Name {
    /// Creates a unique Name from a 32-bit batch ID and an 8-bit offset.
    /// This mapping is bijective (one-to-one) within the 40-bit space.
    pub fn from_batch_and_offset(batch: u32, offset: u8) -> Self {
        Self(permute_40bit(batch, offset))
    }

    pub fn as_bytes(&self) -> [u8; 5] {
        self.0
    }
}

/// A bijective mixing function using Fibonacci Hashing.
fn permute_40bit(batch: u32, offset: u8) -> [u8; 5] {
    // Pack into 40 bits: [batch (32 bits) | offset (8 bits)]
    let mut x = ((batch as u64) << 8) | (offset as u64);

    // Fibonacci Hashing Mix Steps
    x = x.wrapping_mul(PHI_64);
    x ^= x >> 20; // Avalanche high-bit entropy down to the low bits
    x = x.wrapping_mul(PHI_64);

    // Mask to 40 bits and unpack
    x &= 0xFF_FFFF_FFFF;

    [
        (x >> 32) as u8,
        (x >> 24) as u8,
        (x >> 16) as u8,
        (x >> 8) as u8,
        x as u8,
    ]
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", encode(&self.0))
    }
}

impl From<[u8; 5]> for Name {
    fn from(bytes: [u8; 5]) -> Self {
        Name(bytes)
    }
}

impl From<&[u8; 5]> for Name {
    fn from(bytes: &[u8; 5]) -> Self {
        Name(*bytes)
    }
}

impl TryFrom<&[u8]> for Name {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // TODO: better error
        let prefix: [u8; 5] = value.try_into().map_err(|_| "prefix should be 5 bytes")?;

        Ok(Name(prefix))
    }
}

impl FromStr for Name {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Name(decode(s)?))
    }
}

fn encode(bytes: &[u8; 5]) -> String {
    let val = u64::from_be_bytes([
        0, 0, 0, // padding
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4],
    ]);

    let mut result = String::with_capacity(13);
    encode_word((val >> 20) & 0xFFFFF, &mut result);
    result.push('-');
    encode_word(val & 0xFFFFF, &mut result);
    result
}

fn encode_word(bits: u64, result: &mut String) {
    let chars = [
        FIRST_CONSONANTS[((bits >> 16) & 0xF) as usize] as char,
        VOWELS[((bits >> 14) & 0x3) as usize] as char,
        SECOND_CONSONANTS[((bits >> 10) & 0xF) as usize] as char,
        THIRD_CONSONANTS[((bits >> 6) & 0xF) as usize] as char,
        VOWELS[((bits >> 4) & 0x3) as usize] as char,
        FOURTH_CONSONANTS[(bits & 0xF) as usize] as char,
    ];
    chars.iter().for_each(|&c| result.push(c));
}

fn decode(encoded: &str) -> Result<[u8; 5], &'static str> {
    // Quick length check first
    if encoded.len() != 13 || !encoded.is_ascii() {
        return Err("Invalid format: must be 13 ASCII characters with underscore");
    }

    // Use bytes directly for faster processing
    let bytes = encoded.as_bytes();

    // Find the underscore position manually for better performance
    if bytes.iter().position(|&b| b == b'-') != Some(6) {
        return Err("Underscore must be at position 6");
    };

    // Decode both words in parallel
    let first_word = decode_word(&bytes[0..6])?;
    let second_word = decode_word(&bytes[7..13])?;

    // Combine and extract bytes
    let val = (first_word << 20) | second_word;
    Ok([
        (val >> 32) as u8,
        (val >> 24) as u8,
        (val >> 16) as u8,
        (val >> 8) as u8,
        val as u8,
    ])
}

fn decode_word(word: &[u8]) -> Result<u64, &'static str> {
    if word.len() != 6 {
        return Err("Word must be 6 bytes");
    }

    // Precompute all character lookups in parallel
    let c1 = decode_consonant(word, 0)? as u64;
    let v1 = decode_vowel(word[1])? as u64;
    let c2 = decode_consonant(word, 2)? as u64;
    let c3 = decode_consonant(word, 3)? as u64;
    let v2 = decode_vowel(word[4])? as u64;
    let c4 = decode_consonant(word, 5)? as u64;

    // Reconstruct the 20-bit value
    Ok((c1 << 16) | (v1 << 14) | (c2 << 10) | (c3 << 6) | (v2 << 4) | c4)
}

static VOWEL_LOOKUP: [Option<u8>; 256] = {
    let mut table = [None; 256];
    let mut i = 0;
    while i < VOWELS.len() {
        table[VOWELS[i] as usize] = Some(i as u8);
        i += 1;
    }
    table
};

fn decode_consonant(word: &[u8], idx: usize) -> Result<u8, &'static str> {
    match idx {
        0 => FIRST_CONSONANTS
            .iter()
            .position(|s| s == &word[idx])
            .map(|i| i as u8)
            .ok_or("invalid consonant character"),
        2 => SECOND_CONSONANTS
            .iter()
            .position(|s| s == &word[idx])
            .map(|i| i as u8)
            .ok_or("invalid consonant character"),
        3 => THIRD_CONSONANTS
            .iter()
            .position(|s| s == &word[idx])
            .map(|i| i as u8)
            .ok_or("invalid consonant character"),
        5 => FOURTH_CONSONANTS
            .iter()
            .position(|s| s == &word[idx])
            .map(|i| i as u8)
            .ok_or("invalid consonant character"),
        _ => Err("invalid constant order"),
    }
}

fn decode_vowel(b: u8) -> Result<u8, &'static str> {
    VOWEL_LOOKUP[b as usize].ok_or("Invalid vowel character")
}

#[cfg(test)]
mod tests {
    use rand::{Rng, RngCore, thread_rng};

    use super::*;

    #[test]
    fn encode_decode() {
        let mut rng = thread_rng();
        // The prefix is typically supposed to come from
        // the block hash where the registration happens.
        let mut hash = [0_u8; 32];
        rng.fill(&mut hash);

        for id in 0..=255_u8 {
            let name = Name::from([hash[0], hash[1], hash[2], hash[3], id]);
            let decoded: Name = name.to_string().parse().unwrap();

            assert_eq!(decoded, name);
        }
    }

    #[test]
    fn check_names_for_same_30bits_prefix() {
        let mut rng = thread_rng();
        let batch = rng.next_u32();

        for offset in 0..=255_u8 {
            let name = Name::from_batch_and_offset(batch, offset).to_string();
            let name2 = Name::from_batch_and_offset(batch + 1, offset).to_string();
            let name3 = Name::from_batch_and_offset(batch - 1, offset).to_string();

            println!("{name} {name2} {name3}");
        }
    }
    use std::collections::HashSet;

    #[test]
    fn test_global_uniqueness_and_diffusion() {
        let mut seen_names = HashSet::new();
        let total_batches_to_test = 1000;
        // let offsets_per_batch = 256; // Full u8 range

        let mut total_bit_diff = 0;
        let mut comparisons = 0;

        for batch in 0..total_batches_to_test as u32 {
            let mut previous_bits: u64 = 0;

            for offset in 0..=255_u8 {
                let name = Name::from_batch_and_offset(batch, offset);
                let current_bytes = name.as_bytes(); // Assuming Name provides access to its [u8; 5]

                // --- 1. Uniqueness Check ---
                // Convert [u8; 5] to a fixed key for the HashSet
                if !seen_names.insert(current_bytes) {
                    panic!("COLLISION DETECTED at batch {}, offset {}", batch, offset);
                }

                // --- 2. Diffusion (Randomness) Check ---
                // We check the Hamming Distance (how many bits changed)
                // between this ID and the previous one in the sequence.
                let current_bits = pack_to_u64(&current_bytes);
                if offset > 0 {
                    let diff = (current_bits ^ previous_bits).count_ones();
                    total_bit_diff += diff;
                    comparisons += 1;
                }
                previous_bits = current_bits;
            }
        }

        // --- Analysis ---
        let avg_bit_diff = total_bit_diff as f32 / comparisons as f32;
        println!("Tested {} IDs with 0 collisions.", seen_names.len());
        println!(
            "Average bits changed between adjacent IDs: {:.2} / 40",
            avg_bit_diff
        );

        // In a perfectly random distribution, changing 1 input bit should
        // flip 50% of output bits (20 bits out of 40).
        // We check if it's within a reasonable range (e.g., 15-25 bits).
        assert!(
            avg_bit_diff > 15.0 && avg_bit_diff < 25.0,
            "Diffusion is poor: adjacent IDs are too similar (Avg diff: {})",
            avg_bit_diff
        );
    }

    // Helper for the test to treat the 5 bytes as a comparable number
    fn pack_to_u64(bytes: &[u8; 5]) -> u64 {
        ((bytes[0] as u64) << 32)
            | ((bytes[1] as u64) << 24)
            | ((bytes[2] as u64) << 16)
            | ((bytes[3] as u64) << 8)
            | (bytes[4] as u64)
    }
}
