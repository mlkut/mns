use std::{fmt::Display, str::FromStr};

const CONSONANTS: &[u8; 16] = b"bdfghjklmnprstvz";
const VOWELS: &[u8; 4] = b"aiou";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name([u8; 5]);

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
    result.push('_');
    encode_word(val & 0xFFFFF, &mut result);
    result
}

fn encode_word(bits: u64, result: &mut String) {
    // Use a single unsafe block for maximum performance (if safe)
    unsafe {
        result.as_mut_vec().extend_from_slice(&[
            CONSONANTS[((bits >> 16) & 0xF) as usize],
            VOWELS[((bits >> 14) & 0x3) as usize],
            CONSONANTS[((bits >> 10) & 0xF) as usize],
            CONSONANTS[((bits >> 6) & 0xF) as usize],
            VOWELS[((bits >> 4) & 0x3) as usize],
            CONSONANTS[(bits & 0xF) as usize],
        ]);
    }
}

fn decode(encoded: &str) -> Result<[u8; 5], &'static str> {
    // Quick length check first
    if encoded.len() != 13 || !encoded.is_ascii() {
        return Err("Invalid format: must be 13 ASCII characters with underscore");
    }

    // Use bytes directly for faster processing
    let bytes = encoded.as_bytes();

    // Find the underscore position manually for better performance
    if bytes.iter().position(|&b| b == b'_') != Some(6) {
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
    let c1 = decode_consonant(word[0])? as u64;
    let v1 = decode_vowel(word[1])? as u64;
    let c2 = decode_consonant(word[2])? as u64;
    let c3 = decode_consonant(word[3])? as u64;
    let v2 = decode_vowel(word[4])? as u64;
    let c4 = decode_consonant(word[5])? as u64;

    // Reconstruct the 20-bit value
    Ok((c1 << 16) | (v1 << 14) | (c2 << 10) | (c3 << 6) | (v2 << 4) | c4)
}

// Precompute lookup tables for faster decoding
static CONSONANT_LOOKUP: [Option<u8>; 256] = {
    let mut table = [None; 256];
    let mut i = 0;
    while i < CONSONANTS.len() {
        table[CONSONANTS[i] as usize] = Some(i as u8);
        i += 1;
    }
    table
};

static VOWEL_LOOKUP: [Option<u8>; 256] = {
    let mut table = [None; 256];
    let mut i = 0;
    while i < VOWELS.len() {
        table[VOWELS[i] as usize] = Some(i as u8);
        i += 1;
    }
    table
};

fn decode_consonant(b: u8) -> Result<u8, &'static str> {
    CONSONANT_LOOKUP[b as usize].ok_or("Invalid consonant character")
}

fn decode_vowel(b: u8) -> Result<u8, &'static str> {
    VOWEL_LOOKUP[b as usize].ok_or("Invalid vowel character")
}

#[cfg(test)]
mod tests {
    use rand::{RngCore, thread_rng};

    use super::*;

    #[test]
    fn encode_decode() {
        let mut rng = thread_rng();

        let mut bytes = [0u8; 5];
        rng.fill_bytes(&mut bytes);

        let name = Name::from(bytes);

        let encoded = name.to_string();
        let decoded: Name = encoded.parse().unwrap();

        assert_eq!(decoded, name);
    }
}
