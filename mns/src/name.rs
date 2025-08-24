use std::fmt::Display;

const CONSONANTS: &[u8; 16] = b"bdfghjklmnprstvz";
const VOWELS: &[u8; 4] = b"aiou";

#[derive(Debug, Clone)]
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
        Name(bytes.clone())
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

fn encode(bytes: &[u8; 5]) -> String {
    let mut result = String::with_capacity(13); // 6 chars + dash + 6 chars = 13

    // Convert 5 bytes to 40 bits
    let val = ((bytes[0] as u64) << 32)
        | ((bytes[1] as u64) << 24)
        | ((bytes[2] as u64) << 16)
        | ((bytes[3] as u64) << 8)
        | (bytes[4] as u64);

    // First word: bits 39-20 (20 bits for CVCCVC)
    let first_word_bits = (val >> 20) & 0xFFFFF;
    encode_word(first_word_bits, &mut result);

    result.push('_');

    // Second word: bits 19-0 (20 bits for CVCCVC)
    let second_word_bits = val & 0xFFFFF;
    encode_word(second_word_bits, &mut result);

    result
}

// Helper function to encode a 20-bit value as CVCCVC pattern
fn encode_word(bits: u64, result: &mut String) {
    // CVCCVC pattern: 4+2+4+4+2+4 = 20 bits total
    let c1 = ((bits >> 16) & 0xF) as usize; // bits 16-19 (4 bits)
    let v1 = ((bits >> 14) & 0x3) as usize; // bits 14-15 (2 bits)
    let c2 = ((bits >> 10) & 0xF) as usize; // bits 10-13 (4 bits)
    let c3 = ((bits >> 6) & 0xF) as usize; // bits 6-9 (4 bits)
    let v2 = ((bits >> 4) & 0x3) as usize; // bits 4-5 (2 bits)
    let c4 = (bits & 0xF) as usize; // bits 0-3 (4 bits)

    result.push(CONSONANTS[c1] as char);
    result.push(VOWELS[v1] as char);
    result.push(CONSONANTS[c2] as char);
    result.push(CONSONANTS[c3] as char);
    result.push(VOWELS[v2] as char);
    result.push(CONSONANTS[c4] as char);
}
