mod key;

use base64;
use openssl::{
    error::ErrorStack,
    hash::{hash, MessageDigest},
    rsa::{Padding, Rsa},
    symm::{decrypt, encrypt, Cipher},
};
use rand::RngCore;
use serde::Serialize;

use key::{BASE62, EAPI_KEY, IV, LINUX_API_KEY, PRESET_KEY, PUBLIC_KEY};

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
    pub fn to_vec(self) -> Vec<(String, String)> {
        vec![
            ("params".to_owned(), self.params),
            ("encSecKey".to_owned(), self.enc_sec_key),
        ]
    }
}

impl EapiForm {
    pub fn to_vec(self) -> Vec<(String, String)> {
        vec![("params".to_owned(), self.params)]
    }
}

impl LinuxapiForm {
    pub fn to_vec(self) -> Vec<(String, String)> {
        vec![("eparams".to_owned(), self.eparams)]
    }
}

pub fn weapi(text: &[u8]) -> WeapiForm {
    let mut rng = rand::thread_rng();
    let mut rand_buf = [0u8; 16];
    rng.fill_bytes(&mut rand_buf);

    let sk = rand_buf
        .iter()
        .map(|i| BASE62.as_bytes()[(i % 62) as usize])
        .collect::<Vec<u8>>();

    let params = {
        let p = base64::encode(aes_128_cbc(
            text,
            PRESET_KEY.as_bytes(),
            Some(IV.as_bytes()),
        ));
        base64::encode(aes_128_cbc(p.as_bytes(), &sk, Some(IV.as_bytes())))
    };

    let enc_sec_key = {
        let reversed_sk = sk.iter().rev().map(|i| *i).collect::<Vec<u8>>();
        hex::encode(rsa(&reversed_sk, PUBLIC_KEY.as_bytes()))
    };

    WeapiForm {
        params,
        enc_sec_key,
    }
}

pub fn eapi(url: &[u8], data: &[u8]) -> EapiForm {
    let msg = format!(
        "nobody{}use{}md5forencrypt",
        String::from_utf8_lossy(url),
        String::from_utf8_lossy(data)
    );
    let digest = hex::encode(hash(MessageDigest::md5(), msg.as_bytes()).unwrap());

    let text = {
        let d = "-36cd479b6b5-";
        [url, d.as_bytes(), data, d.as_bytes(), digest.as_bytes()].concat()
    };

    let params = {
        let p = aes_128_ecb(&text, EAPI_KEY.as_bytes(), None);
        hex::encode_upper(p)
    };

    EapiForm { params }
}

#[allow(unused)]
pub fn eapi_decrypt(ct: &[u8]) -> Result<Vec<u8>, ErrorStack> {
    aes_128_ecb_decrypt(ct, EAPI_KEY.as_bytes(), None)
}

pub fn linuxapi(text: &[u8]) -> LinuxapiForm {
    let ct = aes_128_ecb(text, LINUX_API_KEY.as_bytes(), None);
    let eparams = hex::encode_upper(ct);

    LinuxapiForm { eparams }
}

fn aes_128_ecb(pt: &[u8], key: &[u8], iv: Option<&[u8]>) -> Vec<u8> {
    let cipher = Cipher::aes_128_ecb();
    let ct = encrypt(cipher, key, iv, pt).unwrap();
    ct
}

fn aes_128_ecb_decrypt(ct: &[u8], key: &[u8], iv: Option<&[u8]>) -> Result<Vec<u8>, ErrorStack> {
    let cipher = Cipher::aes_128_ecb();
    decrypt(cipher, key, iv, ct)
}

fn aes_128_cbc(pt: &[u8], key: &[u8], iv: Option<&[u8]>) -> Vec<u8> {
    let cipher = Cipher::aes_128_cbc();
    let pt = encrypt(cipher, key, iv, pt).unwrap();
    pt
}

fn rsa(pt: &[u8], key: &[u8]) -> Vec<u8> {
    let rsa = Rsa::public_key_from_pem(key).unwrap();

    let prefix = vec![0u8; 128 - pt.len()];
    let pt = [&prefix[..], pt].concat();

    let mut ct = vec![0; rsa.size() as usize];
    rsa.public_encrypt(&pt, &mut ct, Padding::NONE).unwrap();
    ct
}

#[cfg(test)]
mod tests {
    use super::key::{EAPI_KEY, IV, PRESET_KEY, PUBLIC_KEY};
    use super::{aes_128_cbc, aes_128_ecb, aes_128_ecb_decrypt, rsa, weapi};
    use crate::crypto::{eapi, eapi_decrypt, linuxapi};

    #[test]
    fn test_aes_128_ecb() {
        let pt = "plain text";
        let ct = aes_128_ecb(pt.as_bytes(), EAPI_KEY.as_bytes(), None);
        let _pt = aes_128_ecb_decrypt(&ct, EAPI_KEY.as_bytes(), None);
        assert!(_pt.is_ok());

        if let Ok(decrypted) = _pt {
            assert_eq!(&decrypted, pt.as_bytes());
        }
    }

    #[test]
    fn test_aes_cbc() {
        let pt = "plain text";
        let ct = aes_128_cbc(pt.as_bytes(), PRESET_KEY.as_bytes(), Some(IV.as_bytes()));
        assert!(hex::encode(ct).ends_with("baf0"))
    }

    #[test]
    fn test_rsa() {
        let ct = rsa(PRESET_KEY.as_bytes(), PUBLIC_KEY.as_bytes());
        assert!(hex::encode(ct).ends_with("4413"));
    }

    #[test]
    fn test_weapi() {
        weapi(r#"{"username": "alex"}"#.as_bytes());
    }

    #[test]
    fn test_eapi() {
        let ct = eapi("/url".as_bytes(), "plain text".as_bytes());
        assert!(ct.params.ends_with("C3F3"));
    }

    #[test]
    fn test_eapi_decrypt() {
        let pt = "plain text";
        let ct = aes_128_ecb(pt.as_bytes(), EAPI_KEY.as_bytes(), None);
        assert_eq!(pt.as_bytes(), &eapi_decrypt(&ct).unwrap())
    }

    #[test]
    fn test_linuxapi() {
        let ct = linuxapi(r#""plain text""#.as_bytes());
        assert!(ct.eparams.ends_with("2250"));
    }
}
