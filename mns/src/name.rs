use core::iter::Iterator;
use std::{fmt::Display, str::FromStr};

// No `C` or `Q` because they sounds like `K`
// No `H` at the end of a syllable because it is hard to pronounce clearly.
// No `H` at the beginning of the second syllable to avoid accidents like `sh`,
//      leaving `sz` to be pronounced like `sh`.
// No `X` anywhere except last consonant in the word.
// No `W` because it is pronounced `V` by many.
// `P` couldn't be avoided, despite being confused with `B`.

const FIRST_CONSONANTS: &[u8; 16] = b"dmbyslhfrtpnjzvk";
const SECOND_CONSONANTS: &[u8; 16] = b"zrnmtgdskblpvfjy";

const VOWELS: &[u8; 4] = b"oaeu";

const THIRD_CONSONANTS: &[u8; 16] = b"znbvsplfdrytmkgj";
const FOURTH_CONSONANTS: &[u8; 16] = b"dksvrtlnpxbgmfzj";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(u64);

impl Name {
    /// Creates a unique Name from the ordinal of onchain registration.
    pub fn from_ordinal(ordinal: u64) -> Self {
        Self(permute_ordinal(ordinal))
    }

    /// Recovers the ordinal that was used to create this Name.
    /// This is the inverse of `from_ordinal`.
    pub fn to_ordinal(&self) -> u64 {
        unpermute_ordinal(self.as_u64())
    }

    pub(crate) fn encode(&self) -> String {
        let val = self.as_u64();

        let mut result = String::with_capacity(13);
        encode_word((val >> 20) & 0xFFFFF, &mut result);
        result.push('-');
        encode_word(val & 0xFFFFF, &mut result);
        result
    }

    pub(crate) fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.encode())
    }
}

impl FromStr for Name {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Name(decode(s)?))
    }
}

const MASK_20: u64 = 0xFFFFF;

/// The Feistel round function.
/// Must be identical in both directions.
#[inline(always)]
fn round_f(val: u64, r: u64) -> u64 {
    // A simple non-linear mixer for 20 bits
    (val.wrapping_mul(r) ^ (val >> 7) ^ (val << 13)) & MASK_20
}

/// A 40-bit bijective permutation.
/// Guarantees no collisions across the domain [0, 2^40).
pub fn permute_ordinal(x: u64) -> u64 {
    // Ensure input is within 40-bit range
    let mut left = (x >> 20) & MASK_20;
    let mut right = x & MASK_20;

    // 4 rounds of Feistel
    const R: [u64; 4] = [0x9E377, 0x6C62D, 0xB5A4B, 0xD2F3E];

    for &r in &R {
        let next_right = left ^ round_f(right, r);
        left = right;
        right = next_right;
    }

    (left << 20) | right
}

/// Inverse of `permute_ordinal`.
pub fn unpermute_ordinal(x: u64) -> u64 {
    let mut left = (x >> 20) & MASK_20;
    let mut right = x & MASK_20;

    const R: [u64; 4] = [0x9E377, 0x6C62D, 0xB5A4B, 0xD2F3E];

    // Run rounds in reverse order
    for &r in R.iter().rev() {
        // In un-permuting, we calculate the 'f' of the current left
        // to retrieve the previous right.
        let prev_left = right ^ round_f(left, r);
        right = left;
        left = prev_left;
    }

    (left << 20) | right
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

fn decode(encoded: &str) -> Result<u64, &'static str> {
    // Quick length check first
    if encoded.len() != 13 || !encoded.is_ascii() {
        return Err("Invalid format: must be 13 ASCII characters with dash in between");
    }

    // Use bytes directly for faster processing
    let bytes = encoded.as_bytes();

    // Find the dash position manually for better performance
    if bytes.iter().position(|&b| b == b'-') != Some(6) {
        return Err("Dash must be at position 6");
    };

    // Decode both words in parallel
    let first_word = decode_word(&bytes[0..6])?;
    let second_word = decode_word(&bytes[7..13])?;

    // Combine and extract bytes
    let val = (first_word << 20) | second_word;

    Ok(val)
}

fn decode_word(word: &[u8]) -> Result<u64, &'static str> {
    if word.len() != 6 {
        return Err("Word must be 6 bytes");
    }

    // Precompute all character lookups in parallel
    let c1 = decode_consonant(0, word[0])? as u64;
    let v1 = decode_vowel(word[1])? as u64;
    let c2 = decode_consonant(1, word[2])? as u64;
    let c3 = decode_consonant(2, word[3])? as u64;
    let v2 = decode_vowel(word[4])? as u64;
    let c4 = decode_consonant(3, word[5])? as u64;

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

static CONSONANT_LOOKUP: [[Option<u8>; 256]; 4] = {
    let mut tables = [[None; 256]; 4];
    let lists = [
        FIRST_CONSONANTS,
        SECOND_CONSONANTS,
        THIRD_CONSONANTS,
        FOURTH_CONSONANTS,
    ];

    let mut i = 0;
    while i < lists.len() {
        let mut j = 0;
        while j < lists[i].len() {
            tables[i][lists[i][j] as usize] = Some(j as u8);
            j += 1;
        }
        i += 1;
    }

    tables
};

fn decode_consonant(table_idx: usize, ch: u8) -> Result<u8, &'static str> {
    CONSONANT_LOOKUP[table_idx][ch as usize].ok_or("invalid consonant character")
}

fn decode_vowel(b: u8) -> Result<u8, &'static str> {
    VOWEL_LOOKUP[b as usize].ok_or("Invalid vowel character")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn check_names_for_ordinals() {
        for ordinal in 1..=255_u64 {
            let name = Name::from_ordinal(ordinal).to_string();
            let name2 = Name::from_ordinal(ordinal + 1).to_string();
            let name3 = Name::from_ordinal(ordinal - 1).to_string();

            println!("{name} {name2} {name3}");
        }
    }

    #[test]
    fn test_name_regression_golden_values() {
        // These values are derived from the current Feistel constants
        // and the consonant/vowel bit-packing logic.
        let cases = [
            (0, "dozzod-dozzod"),            // Ordinal 0
            (1, "tanfuj-hudzuf"),            // Ordinal 1
            (42, "jeyvas-hayses"),           // Ordinal 42
            (1_000_000, "jeyzax-zogjep"),    // Large ordinal
            (0xFFFFFFFFFF, "moddog-vodnof"), // Max 40-bit value
        ];

        for (ordinal, expected_name) in cases {
            let actual_name = Name::from_ordinal(ordinal).to_string();
            assert_eq!(
                actual_name, expected_name,
                "Regression failure: ordinal {} produced {} but expected {}",
                ordinal, actual_name, expected_name
            );
        }
    }

    #[test]
    fn test_global_uniqueness_and_diffusion() {
        use rayon::prelude::*;

        let count: u64 = 10_000_000;

        // --- Parallel diffusion + collect names ---
        // Each chunk computes its own bit-diff sum; boundary diffs between
        // chunks are handled separately below.
        let chunk_size: u64 = 100_000;
        let num_chunks = count.div_ceil(chunk_size);

        // Collect the first name of each chunk so we can compute cross-boundary diffs.
        let chunk_heads: Vec<u64> = (0..num_chunks)
            .map(|c| Name::from_ordinal(c * chunk_size).as_u64())
            .collect();

        let results: Vec<(Vec<u64>, u64)> = (0..num_chunks)
            .into_par_iter()
            .map(|chunk| {
                let start = chunk * chunk_size;
                let end = (start + chunk_size).min(count);

                let mut values = Vec::with_capacity((end - start) as usize);
                let mut bit_diff: u64 = 0;
                let mut prev = Name::from_ordinal(start).as_u64();
                values.push(prev);

                for ordinal in (start + 1)..end {
                    let v = Name::from_ordinal(ordinal).as_u64();
                    bit_diff += (v ^ prev).count_ones() as u64;
                    prev = v;
                    values.push(v);
                }
                (values, bit_diff)
            })
            .collect();

        let mut total_bit_diff: u64 = results.iter().map(|(_, d)| d).sum();
        let all_values: Vec<Vec<u64>> = results.into_iter().map(|(v, _)| v).collect();

        // Add the cross-boundary diffs
        for i in 1..chunk_heads.len() {
            // last value of chunk i-1 is chunk_heads[i] - 1's name;
            // easier: just use the last element of each chunk vec.
            let last_of_prev = *all_values[i as usize - 1].last().unwrap();
            total_bit_diff += (chunk_heads[i] ^ last_of_prev).count_ones() as u64;
        }

        // --- Uniqueness: sort + dedup is faster than HashSet for large N ---
        let mut flat: Vec<u64> = all_values.into_iter().flatten().collect();
        flat.par_sort_unstable();
        let original_len = flat.len();
        flat.dedup();
        if flat.len() != original_len {
            // Find the duplicate for a useful panic message
            let dupes: Vec<u64> = flat
                .windows(2)
                .filter(|w| w[0] == w[1])
                .map(|w| w[0])
                .collect();
            panic!(
                "COLLISION DETECTED — duplicate name values: {:?}",
                &dupes[..dupes.len().min(5)]
            );
        }

        assert_eq!(flat.len(), count as usize);
        println!("Tested {} IDs with 0 collisions.", flat.len());

        let comparisons = count - 1;
        let avg_bit_diff = total_bit_diff as f32 / comparisons as f32;
        println!(
            "Average bits changed between adjacent IDs: {:.2} / 40",
            avg_bit_diff
        );

        assert!(
            (avg_bit_diff - 20.0).abs() < 0.10,
            "Diffusion is poor: adjacent IDs are too similar (Avg diff: {avg_bit_diff})"
        );
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Tests that any 5-byte sequence can be encoded to a string
        /// and decoded back to the exact same 5 bytes.
        #[test]
        fn test_name_string_roundtrip(ordinal in 0..0xFFFFFFFFFFu64) {
            let name = Name::from_ordinal(ordinal);
            let encoded = name.to_string();
            let decoded: Name = encoded.parse().unwrap();
            assert_eq!(name, decoded, "String roundtrip failed for ordinal: {}, encoded as: {}", ordinal, encoded);
        }

        /// Tests that any ordinal (0 to 2^40-1) can be converted to
        /// a Name and back to the same ordinal.
        #[test]
        fn test_ordinal_permutation_roundtrip(ordinal in 0..0xFFFFFFFFFFu64) {
            let name = Name::from_ordinal(ordinal);
            let recovered = name.to_ordinal();
            assert_eq!(ordinal, recovered, "Ordinal roundtrip failed for: {}", ordinal);
        }
    }
}
