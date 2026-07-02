use alloy::primitives::B256;
use alloy::signers::local::PrivateKeySigner as AlloySigner;
use k256::SecretKey;
use keyring::Entry;
use rand::rngs::OsRng;

const SERVICE_NAME: &str = "mns";
const KEYRING_USER: &str = "private-key";

fn keyring_entry() -> Result<keyring::Entry, Box<dyn std::error::Error>> {
    Ok(Entry::new(SERVICE_NAME, KEYRING_USER)?)
}

fn secret_to_signer(secret: &SecretKey) -> Result<AlloySigner, Box<dyn std::error::Error>> {
    let b256 = B256::from_slice(&secret.to_bytes()[..]);
    Ok(AlloySigner::from_bytes(&b256)?)
}

pub fn get_or_create_key() -> Result<AlloySigner, Box<dyn std::error::Error>> {
    let entry = keyring_entry()?;
    match entry.get_password() {
        Ok(hex_key) => {
            let bytes = hex::decode(hex_key.trim())?;
            let secret = SecretKey::from_slice(&bytes)?;
            secret_to_signer(&secret)
        }
        Err(_) => {
            let secret = SecretKey::random(&mut OsRng);
            let signer = secret_to_signer(&secret)?;
            let hex = hex::encode(&secret.to_bytes()[..]);
            entry.set_password(&hex)?;
            Ok(signer)
        }
    }
}

pub fn get_key() -> Result<AlloySigner, Box<dyn std::error::Error>> {
    let entry = keyring_entry()?;
    let hex_key = entry.get_password()?;
    let bytes = hex::decode(hex_key.trim())?;
    let secret = SecretKey::from_slice(&bytes)?;
    secret_to_signer(&secret)
}
