use crate::errors::*;
use sodiumoxide::crypto::secretbox::{self, Key, Nonce};
use std::iter;

pub fn key_trunc_pad(mut key: &[u8], len: usize, pad: u8) -> Vec<u8> {
    if key.len() > len {
        key = &key[..len];
    }

    let mut key = key.to_vec();
    key.extend(iter::repeat(pad).take(len - key.len()));
    key
}

pub fn sodium_secretbox_open(encrypted: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if encrypted.len() <= secretbox::NONCEBYTES {
        bail!("Encrypted message is too short");
    }

    let key = Key::from_slice(key)
        .ok_or_else(|| format_err!("Key has wrong length"))?;
    let nonce = Nonce::from_slice(&encrypted[..secretbox::NONCEBYTES])
        .ok_or_else(|| format_err!("Nonce has wrong length"))?;
    let ciphertext = &encrypted[secretbox::NONCEBYTES..];
    let plain = secretbox::open(ciphertext, &nonce, &key)
        .map_err(|_| format_err!("Failed to decrypt secretbox"))?;
    Ok(plain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_equal() {
        let key = key_trunc_pad(&[1, 2, 3, 4, 5], 5, 0);
        assert_eq!(key, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_key_trunc() {
        let key = key_trunc_pad(&[1, 2, 3, 4, 5, 6, 7, 8, 9], 5, 0);
        assert_eq!(key, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_key_pad() {
        let key = key_trunc_pad(&[1, 2, 3], 5, 0);
        assert_eq!(key, &[1, 2, 3, 0, 0]);
    }
}
