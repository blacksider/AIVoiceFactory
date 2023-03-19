#![allow(unused)]

use aes::Aes256;
use base64::{Engine as _, engine::general_purpose};
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use rand::seq::SliceRandom;

static KEY: &str = "aivoice_factory20230319@macarron";

type AesCBC = Cbc<Aes256, Pkcs7>;

/// generate random bytes iv for encoding
fn generate_random_iv(size: usize) -> String {
    let mut rng = &mut rand::thread_rng();
    String::from_utf8(
        KEY.as_bytes()
            .choose_multiple(&mut rng, size)
            .cloned()
            .collect()
    ).unwrap()
}

/// encrypt data by key, return a base64 encoded string
fn encrypt(key: &str, data: &str) -> String {
    let iv_str = generate_random_iv(16);
    let iv = iv_str.as_bytes();
    let cipher = AesCBC::new_from_slices(key.as_bytes(), iv).unwrap();
    let ciphertext = cipher.encrypt_vec(data.as_bytes());
    let mut buffer = bytebuffer::ByteBuffer::from_bytes(iv);
    buffer.write_bytes(&ciphertext);
    general_purpose::STANDARD.encode(buffer.as_bytes().to_vec())
}

/// decrypt data by key, data should be a base64 encoded string, return decrypted string
fn decrypt(key: &str, data: &str) -> String {
    let bytes = general_purpose::STANDARD
        .decode(data).unwrap();
    let cipher = AesCBC::new_from_slices(key.as_bytes(), &bytes[0..16]).unwrap();
    String::from_utf8(cipher.decrypt_vec(&bytes[16..]).unwrap()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let encrypted = encrypt(KEY, "some text");
        assert_eq!(decrypt(KEY, &encrypted), "some text");
    }
}
