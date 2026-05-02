use std::{fmt::Display, str::FromStr};

// No `C` or `Q` because they sounds like `K`
// No `H` at the end of a syllable because it is hard to pronounce clearly.
// No `X` anywhere except last consonant in the word.

const FIRST_CONSONANTS: &[u8; 16] = b"dmbwslhfrtpnjzvk";
const SECOND_CONSONANTS: &[u8; 16] = b"zrnmtgdskblpvfjx";

const VOWELS: &[u8; 4] = b"oaiu";

const THIRD_CONSONANTS: &[u8; 16] = b"znbvsplfdrhtmkgj";
const FOURTH_CONSONANTS: &[u8; 16] = b"dksvrtlnpxbgmfzj";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name([u8; 5]);

impl Name {
    /// Creates a unique Name from the ordinal of onchain registration.
    pub fn from_ordinal(ordinal: u64) -> Self {
        Self::from(ordinal)
    }

    /// Recovers the ordinal that was used to create this Name.
    /// This is the inverse of `from_ordinal`.
    pub fn to_ordinal(&self) -> u64 {
        unpermute_ordinal(self.as_u64())
    }

    pub(crate) fn as_u64(&self) -> u64 {
        u64::from_be_bytes([
            0, 0, 0, self.0[0], self.0[1], self.0[2], self.0[3], self.0[4],
        ])
    }
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

impl From<u64> for Name {
    /// Creates a unique Name from the ordinal of onchain registration.
    fn from(ordinal: u64) -> Self {
        let b = permute_ordinal(ordinal).to_be_bytes();
        Self([b[3], b[4], b[5], b[6], b[7]])
    }
}

/// A 40-bit bijective permutation via Feistel network.
/// Guarantees no collisions across all 2^40 values.
fn permute_ordinal(x: u64) -> u64 {
    // Feistel network over 40 bits (two 20-bit halves).
    // This is provably bijective: each round is reversible,
    // and the composition of reversible functions is reversible.
    let mut left = (x >> 20) & 0xFFFFF;
    let mut right = x & 0xFFFFF;

    // Round constants (odd 20-bit values, chosen arbitrarily)
    const R: [u64; 4] = [0x9E377, 0x6C62D, 0xB5A4B, 0xD2F3E];

    for &r in &R {
        // f(right) = cheap 20-bit avalanche
        let f = right.wrapping_mul(r) ^ (right >> 7) ^ (right << 13);
        let new_right = left ^ (f & 0xFFFFF);
        left = right;
        right = new_right;
    }

    (left << 20) | right
}

/// Inverse of `permute_ordinal`. Runs the Feistel rounds in reverse.
fn unpermute_ordinal(x: u64) -> u64 {
    let mut left = (x >> 20) & 0xFFFFF;
    let mut right = x & 0xFFFFF;

    const R: [u64; 4] = [0x9E377, 0x6C62D, 0xB5A4B, 0xD2F3E];

    for &r in R.iter().rev() {
        let f = left.wrapping_mul(r) ^ (left >> 7) ^ (left << 13);
        let new_left = right ^ (f & 0xFFFFF);
        right = left;
        left = new_left;
    }

    (left << 20) | right
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
        return Err("Invalid format: must be 13 ASCII characters with dash in between");
    }

    // Use bytes directly for faster processing
    let bytes = encoded.as_bytes();

    // Find the underscore position manually for better performance
    if bytes.iter().position(|&b| b == b'-') != Some(6) {
        return Err("Dash must be at position 6");
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
    use rand::{Rng, thread_rng};

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
    fn check_names_for_ordinals() {
        for ordinal in 1..=255_u64 {
            let name = Name::from_ordinal(ordinal).to_string();
            let name2 = Name::from_ordinal(ordinal + 1).to_string();
            let name3 = Name::from_ordinal(ordinal - 1).to_string();

            println!("{name} {name2} {name3}");
        }
    }

    #[test]
    fn test_ordinal_roundtrip() {
        for ordinal in 0..100_000_u64 {
            let name = Name::from_ordinal(ordinal);
            assert_eq!(
                name.to_ordinal(),
                ordinal,
                "roundtrip failed for ordinal {ordinal}"
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
