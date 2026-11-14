mod key;

use aes::Aes128;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use cbc::Encryptor;
use cipher::{BlockDecryptMut, BlockEncryptMut, KeyInit, KeyIvInit, block_padding::Pkcs7};
use md5::{Digest, Md5};
use rand::RngExt;
use rsa::{BigUint, RsaPublicKey, pkcs8::DecodePublicKey, traits::PublicKeyParts};
use serde::Serialize;

use key::{BASE62, EAPI_KEY, IV, LINUX_API_KEY, PRESET_KEY, PUBLIC_KEY};

/// Pure-Rust crypto failure (no OpenSSL in the dependency graph).
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct CryptoError(String);

impl CryptoError {
    fn message(error: impl std::fmt::Display) -> Self {
        Self(error.to_string())
    }
}

#[derive(Serialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Crypto {
    Weapi,
    Eapi,
    #[allow(unused)]
    Linuxapi,
}

pub struct WeapiForm {
    params: String,
    enc_sec_key: String,
}

pub struct EapiForm {
    params: String,
}
pub struct LinuxapiForm {
    eparams: String,
}

impl WeapiForm {
    pub fn into_vec(self) -> Vec<(String, String)> {
        vec![
            ("params".to_owned(), self.params),
            ("encSecKey".to_owned(), self.enc_sec_key),
        ]
    }
}

impl EapiForm {
    pub fn into_vec(self) -> Vec<(String, String)> {
        vec![("params".to_owned(), self.params)]
    }
}

impl LinuxapiForm {
    pub fn into_vec(self) -> Vec<(String, String)> {
        vec![("eparams".to_owned(), self.eparams)]
    }
}

pub fn weapi(text: &[u8]) -> Result<WeapiForm, CryptoError> {
    let mut rng = rand::rng();
    let mut rand_buf = [0u8; 16];
    rng.fill(&mut rand_buf);

    let sk = rand_buf
        .iter()
        .map(|i| BASE62.as_bytes()[(i % 62) as usize])
        .collect::<Vec<u8>>();

    let params = {
        let p = STANDARD.encode(aes_128_cbc(text, PRESET_KEY.as_bytes(), IV.as_bytes())?);
        STANDARD.encode(aes_128_cbc(p.as_bytes(), &sk, IV.as_bytes())?)
    };

    let enc_sec_key = {
        let reversed_sk = sk.iter().rev().copied().collect::<Vec<u8>>();
        hex::encode(rsa(&reversed_sk, PUBLIC_KEY.as_bytes())?)
    };

    Ok(WeapiForm {
        params,
        enc_sec_key,
    })
}

pub fn eapi(url: &[u8], data: &[u8]) -> Result<EapiForm, CryptoError> {
    let msg = format!(
        "nobody{}use{}md5forencrypt",
        String::from_utf8_lossy(url),
        String::from_utf8_lossy(data)
    );
    let digest = hex::encode(Md5::digest(msg.as_bytes()));

    let text = {
        let d = "-36cd479b6b5-";
        [url, d.as_bytes(), data, d.as_bytes(), digest.as_bytes()].concat()
    };

    let params = hex::encode_upper(aes_128_ecb(&text, EAPI_KEY.as_bytes())?);

    Ok(EapiForm { params })
}

#[allow(unused)]
pub fn eapi_decrypt(ct: &[u8]) -> Result<Vec<u8>, CryptoError> {
    aes_128_ecb_decrypt(ct, EAPI_KEY.as_bytes())
}

pub fn linuxapi(text: &[u8]) -> Result<LinuxapiForm, CryptoError> {
    let ct = aes_128_ecb(text, LINUX_API_KEY.as_bytes())?;
    Ok(LinuxapiForm {
        eparams: hex::encode_upper(ct),
    })
}

fn aes_128_ecb(pt: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes128::new_from_slice(key).map_err(CryptoError::message)?;
    Ok(cipher.encrypt_padded_vec_mut::<Pkcs7>(pt))
}

fn aes_128_ecb_decrypt(ct: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes128::new_from_slice(key).map_err(CryptoError::message)?;
    cipher
        .decrypt_padded_vec_mut::<Pkcs7>(ct)
        .map_err(CryptoError::message)
}

fn aes_128_cbc(pt: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let cipher = Encryptor::<Aes128>::new_from_slices(key, iv).map_err(CryptoError::message)?;
    Ok(cipher.encrypt_padded_vec_mut::<Pkcs7>(pt))
}

/// Raw RSA (no PKCS#1 padding): the protocol left-pads the plaintext itself.
fn rsa(pt: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let pem = std::str::from_utf8(key).map_err(CryptoError::message)?;
    let public =
        RsaPublicKey::from_public_key_pem(&normalize_pem(pem)?).map_err(CryptoError::message)?;
    let size = public.size();

    if pt.len() > size {
        return Err(CryptoError(
            "RSA plaintext is larger than the modulus".to_owned(),
        ));
    }
    let mut padded = vec![0u8; size - pt.len()];
    padded.extend_from_slice(pt);

    let message = BigUint::from_bytes_be(&padded);
    let encrypted = message.modpow(public.e(), public.n());
    let mut bytes = encrypted.to_bytes_be();
    if bytes.len() < size {
        let mut prefix = vec![0u8; size - bytes.len()];
        prefix.append(&mut bytes);
        bytes = prefix;
    }
    Ok(bytes)
}

/// Rewrap a PEM body at 64 columns; some parsers reject long single lines.
fn normalize_pem(pem: &str) -> Result<String, CryptoError> {
    let body: String = pem
        .lines()
        .filter(|line| !line.trim().starts_with("-----"))
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect();
    if body.is_empty() {
        return Err(CryptoError("empty public key".to_owned()));
    }
    let mut normalized = String::from("-----BEGIN PUBLIC KEY-----\n");
    for chunk in body.as_bytes().chunks(64) {
        normalized.push_str(std::str::from_utf8(chunk).map_err(CryptoError::message)?);
        normalized.push('\n');
    }
    normalized.push_str("-----END PUBLIC KEY-----\n");
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::key::{EAPI_KEY, IV, PRESET_KEY, PUBLIC_KEY};
    use super::{aes_128_cbc, aes_128_ecb, aes_128_ecb_decrypt, rsa, weapi};
    use crate::crypto::{eapi, eapi_decrypt, linuxapi};

    #[test]
    fn test_aes_128_ecb() {
        let pt = "plain text";
        let ct = aes_128_ecb(pt.as_bytes(), EAPI_KEY.as_bytes()).expect("valid test cipher");
        let _pt = aes_128_ecb_decrypt(&ct, EAPI_KEY.as_bytes());
        assert!(_pt.is_ok());

        if let Ok(decrypted) = _pt {
            assert_eq!(&decrypted, pt.as_bytes());
        }
    }

    #[test]
    fn test_aes_cbc() {
        let pt = "plain text";
        let ct = aes_128_cbc(pt.as_bytes(), PRESET_KEY.as_bytes(), IV.as_bytes())
            .expect("valid test cipher");
        assert!(hex::encode(ct).ends_with("baf0"))
    }

    #[test]
    fn test_rsa() {
        let ct = rsa(PRESET_KEY.as_bytes(), PUBLIC_KEY.as_bytes()).expect("valid public key");
        assert!(hex::encode(ct).ends_with("4413"))
    }

    #[test]
    fn test_weapi() {
        weapi(r#"{"username": "alex"}"#.as_bytes()).expect("valid weapi payload");
    }

    #[test]
    fn test_eapi() {
        let ct = eapi("/url".as_bytes(), "plain text".as_bytes()).expect("valid eapi payload");
        assert!(ct.params.ends_with("C3F3"));
    }

    #[test]
    fn test_eapi_decrypt() {
        let pt = "plain text";
        let ct = aes_128_ecb(pt.as_bytes(), EAPI_KEY.as_bytes()).expect("valid test cipher");
        assert_eq!(pt.as_bytes(), &eapi_decrypt(&ct).unwrap())
    }

    #[test]
    fn test_linuxapi() {
        let ct = linuxapi(r#""plain text""#.as_bytes()).expect("valid linuxapi payload");
        assert!(ct.eparams.ends_with("2250"));
    }
}
