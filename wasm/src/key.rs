use crate::{
    jserr,
    types::{PubkeyAndPhrase, SignerConfig},
};
use bip39::{Language, Mnemonic, MnemonicType};
use solana_sdk::signature::{keypair_from_seed_phrase_and_passphrase, Signer};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "generateKey")]
pub fn generate_key(passphrase: Option<String>) -> Result<JsValue, JsValue> {
    let passphrase = match passphrase {
        Some(passphrase) => passphrase,
        None => "".to_string(),
    };
    let word_count = 24;
    let mnemonic_type = jserr!(MnemonicType::for_word_count(word_count));
    let language = Language::English;
    let phrase = Mnemonic::new(mnemonic_type, language).into_phrase();
    let keypair = jserr!(keypair_from_seed_phrase_and_passphrase(
        &phrase,
        &passphrase
    ));
    let pubkey_and_passphrase = PubkeyAndPhrase::new(&keypair.pubkey().to_string(), &phrase);
    Ok(jserr!(JsValue::from_serde(&pubkey_and_passphrase)))
}

#[wasm_bindgen(js_name = "getPubkeyFromConfig")]
pub fn get_pubkey_from_config(config: &SignerConfig) -> Result<String, JsValue> {
    let keypair = jserr!(keypair_from_seed_phrase_and_passphrase(
        &config.phrase().as_ref(),
        &config.passphrase().as_ref()
    ));
    Ok(keypair.pubkey().to_string())
}

#[cfg(test)]
mod test{
    use super::*;
    use wasm_bindgen_test::*;

    static BLOCKHASH: &str = "3r1DbHt5RtsQfdDMyLaeBkoQqMcn3m4S4kDLFj4YHvae";
    static PHRASE: &str =
        "plunge bitter method anchor slogan talent draft obscure mimic hover ordinary tiny";
    static PASSPHRASE: &str = "";

    #[wasm_bindgen_test]
    fn test_get_pubkey_from_phrase() {
        let config = SignerConfig::new(BLOCKHASH, PHRASE, PASSPHRASE, None);
        let pubkey = get_pubkey_from_config(&config).unwrap();
        assert_eq!(&pubkey, "6xKtnsnabAsPXRbA6sd7GYQBSb4HFbuiEebJwkL1ecrz");
        
    }
}
