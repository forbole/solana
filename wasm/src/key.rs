use crate::{jserr, types::PubkeyAndPhrase};
use solana_sdk::signature::{Signer, keypair_from_seed_phrase_and_passphrase};
use bip39::{Language, Mnemonic, MnemonicType};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "generateKey")]
pub fn generate_key(passphrase: Option<String>) -> Result<JsValue, JsValue> {
    let passphrase = match passphrase {
        Some(passphrase) => passphrase,
        None => "".to_string(),
    };
    let word_count = 12;
    let mnemonic_type = jserr!(MnemonicType::for_word_count(word_count));
    let language = Language::English;
    let phrase = Mnemonic::new(mnemonic_type, language).into_phrase();
    let keypair = jserr!(keypair_from_seed_phrase_and_passphrase(&phrase, &passphrase));
    let pubkey_and_passphrase = PubkeyAndPhrase::new(&keypair.pubkey().to_string(), &phrase);
    Ok(jserr!(JsValue::from_serde(&pubkey_and_passphrase)))
}
