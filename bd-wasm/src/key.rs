use bip39::{Language, Mnemonic, MnemonicType};
use wasm_bindgen::prelude::*;

use ed25519_dalek::{Keypair, PublicKey, SecretKey, SECRET_KEY_LENGTH};
use hmac::Hmac;
use pbkdf2::pbkdf2;
use serde::{Deserialize, Serialize};
use sha2::Sha512;
use unicode_normalization::UnicodeNormalization;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    public_key: Vec<u8>,
    secret_key: Vec<u8>,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PairAndPhrase {
    pair: Pair,
    phrase: String,
}

#[wasm_bindgen(js_name = "generateKey")]
pub fn generate_key(passphrase: &str) -> Result<JsValue, JsValue> {
    let word_count = 12;
    let mnemonic_type = MnemonicType::for_word_count(word_count).unwrap();
    let language = Language::English;
    let phrase = Mnemonic::new(mnemonic_type, language).into_phrase();

    let pair = generate_pairs(&phrase, passphrase);
    let result = PairAndPhrase { pair, phrase };
    Ok(JsValue::from_serde(&result).unwrap())
}

#[wasm_bindgen(js_name = "recoverKey")]
pub fn recover_key(phrase: &str, passphrase: &str) -> Result<JsValue, JsValue> {
    let pairs = generate_pairs(phrase, passphrase);
    Ok(JsValue::from_serde(&pairs).unwrap())
}

pub fn generate_pairs(phrase: &str, password: &str) -> Pair {
    let seed = generate_seed(&phrase, &password);
    let secret = SecretKey::from_bytes(&seed).unwrap();
    let public = PublicKey::from(&secret);
    let keypair = Keypair { secret, public };
    let secret_key = keypair.to_bytes().to_vec();
    let public_key = keypair.public.as_bytes().to_vec();
    
    Pair {
        public_key,
        secret_key,
    }
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
