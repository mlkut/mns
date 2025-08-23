use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Name([u8; 5]);

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let input = base32::encode(base32::Alphabet::Z, &self.0);
        let first_part = &input[..4];
        let second_part = &input[4..];

        write!(f, "{}_{}", first_part, second_part)
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
