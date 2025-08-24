use std::{fmt::Display, str::FromStr};

const CONSONANTS: &[u8; 16] = b"bdfghjklmnprstvz";
const VOWELS: &[u8; 4] = b"aiou";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name([u8; 6]); // Changed from 5 to 6 bytes

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", encode(&self.0))
    }
}

impl From<[u8; 6]> for Name {
    fn from(bytes: [u8; 6]) -> Self {
        Name(bytes)
    }
}

impl From<&[u8; 6]> for Name {
    fn from(bytes: &[u8; 6]) -> Self {
        Name(bytes.clone())
    }
}

impl TryFrom<&[u8]> for Name {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let prefix: [u8; 6] = value.try_into().map_err(|_| "prefix should be 6 bytes")?;
        Ok(Name(prefix))
    }
}

impl FromStr for Name {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Name(decode(s)?))
    }
}

// Encode 6 bytes as 3 CVCVC words (16 bits each)
fn encode(bytes: &[u8; 6]) -> String {
    let val = u64::from_be_bytes([
        0, 0, // padding
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
    ]);

    let mut result = String::with_capacity(17); // 5 + 1 + 5 + 1 + 5 = 17 chars
    encode_cvcvc_word((val >> 32) & 0xFFFF, &mut result);
    result.push('_');
    encode_cvcvc_word((val >> 16) & 0xFFFF, &mut result);
    result.push('_');
    encode_cvcvc_word(val & 0xFFFF, &mut result);
    result
}

// Helper function to encode a 16-bit value as CVCVC pattern
fn encode_cvcvc_word(bits: u64, result: &mut String) {
    // CVCVC pattern: 4+2+4+2+4 = 16 bits total
    let c1 = ((bits >> 12) & 0xF) as usize; // bits 12-15 (4 bits)
    let v1 = ((bits >> 10) & 0x3) as usize; // bits 10-11 (2 bits)
    let c2 = ((bits >> 6) & 0xF) as usize; // bits 6-9 (4 bits)
    let v2 = ((bits >> 4) & 0x3) as usize; // bits 4-5 (2 bits)
    let c3 = (bits & 0xF) as usize; // bits 0-3 (4 bits)

    unsafe {
        result.as_mut_vec().extend_from_slice(&[
            CONSONANTS[c1],
            VOWELS[v1],
            CONSONANTS[c2],
            VOWELS[v2],
            CONSONANTS[c3],
        ]);
    }
}

fn decode(encoded: &str) -> Result<[u8; 6], &'static str> {
    // Check format: 5 + 1 + 5 + 1 + 5 = 17 characters
    if encoded.len() != 17 || !encoded.is_ascii() {
        return Err("Invalid format: must be 17 ASCII characters with two underscores");
    }

    let bytes = encoded.as_bytes();

    // Find underscore positions
    let underscores: Vec<usize> = bytes
        .iter()
        .enumerate()
        .filter(|&(_, &b)| b == b'_')
        .map(|(i, _)| i)
        .collect();

    if underscores != [5, 11] {
        return Err("Underscores must be at positions 5 and 11");
    }

    // Decode all three words
    let word1 = decode_cvcvc_word(&bytes[0..5])?;
    let word2 = decode_cvcvc_word(&bytes[6..11])?;
    let word3 = decode_cvcvc_word(&bytes[12..17])?;

    // Combine the three 16-bit values into a 48-bit value
    let val = (word1 << 32) | (word2 << 16) | word3;

    // Extract 6 bytes from the 48-bit value
    Ok([
        (val >> 40) as u8,
        (val >> 32) as u8,
        (val >> 24) as u8,
        (val >> 16) as u8,
        (val >> 8) as u8,
        val as u8,
    ])
}

fn decode_cvcvc_word(word: &[u8]) -> Result<u64, &'static str> {
    if word.len() != 5 {
        return Err("CVCVC word must be 5 bytes");
    }

    // CVCVC pattern: positions 0,2,4 are consonants, positions 1,3 are vowels
    let c1 = decode_consonant(word[0])? as u64;
    let v1 = decode_vowel(word[1])? as u64;
    let c2 = decode_consonant(word[2])? as u64;
    let v2 = decode_vowel(word[3])? as u64;
    let c3 = decode_consonant(word[4])? as u64;

    // Reconstruct the 16-bit value: c1(4) + v1(2) + c2(4) + v2(2) + c3(4)
    Ok((c1 << 12) | (v1 << 10) | (c2 << 6) | (v2 << 4) | c3)
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

        let mut bytes = [0u8; 6];
        rng.fill_bytes(&mut bytes);

        let name = Name::from(bytes);
        println!("{name}.mns.alt");

        let encoded = name.to_string();
        let decoded: Name = encoded.parse().unwrap();

        assert_eq!(decoded, name);
    }
}
