use bip39::{Language, Mnemonic, MnemonicType};
use wasm_bindgen::prelude::*;
use ed25519_dalek::{SECRET_KEY_LENGTH};
use hmac::Hmac;
use pbkdf2::pbkdf2;
use sha2::Sha512;
use unicode_normalization::UnicodeNormalization;

#[wasm_bindgen(js_name = "generateKey")]
pub fn generate_key() -> Result<String, JsValue> {
    let word_count = 12;
    let mnemonic_type = MnemonicType::for_word_count(word_count).unwrap();
    let language = Language::English;
    let phrase = Mnemonic::new(mnemonic_type, language).into_phrase();
    Ok(phrase)
}

pub fn generate_seed(phrase: &str, password: &str) -> Vec<u8> {
    const PBKDF2_ROUNDS: u32 = 2048;
    const PBKDF2_BYTES: usize = 64;

    let salt = format!("mnemonic{}", password);
    let normalized_salt = salt.nfkd().to_string();
    let mut seed = [0u8; PBKDF2_BYTES];

    pbkdf2::<Hmac<Sha512>>(
        phrase.as_bytes(),
        normalized_salt.as_bytes(),
        PBKDF2_ROUNDS,
        &mut seed,
    );

    seed[..SECRET_KEY_LENGTH].to_vec()
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use hex_literal::hex;
    #[test]
    fn generate_seed_test() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let password = "nullius　à　nym.zone ¹teſts² English";
        let expected_seed_hex = hex!("61f3aa13adcf5f4b8661fc062501d67eca3a53fc0ed129076ad7a22983b6b5ed0e84e47b24cff23b7fca57e127f62f28c1584ed487872d4bfbc773257bdbc434");
        let seed = generate_seed(phrase, password);
        assert_eq!(
            seed[..SECRET_KEY_LENGTH],
            expected_seed_hex[..SECRET_KEY_LENGTH]
        );
    }
}
