use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SignerConfig {
    blockhash: String,
    phrase: String,
    passphrase: String,
    nonce: Option<String>,
    seed: Option<String>,
}

#[wasm_bindgen(skip)]
impl SignerConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(
        blockhash: &str,
        phrase: &str,
        passphrase: &str,
        nonce: Option<String>,
        seed: Option<String>,
    ) -> SignerConfig {
        SignerConfig {
            blockhash: blockhash.to_string(),
            phrase: phrase.to_string(),
            passphrase: passphrase.to_string(),
            nonce: nonce,
            seed: seed,
        }
    }
    pub fn phrase(&self) -> String {
        self.phrase.clone()
    }
    pub fn passphrase(&self) -> String {
        self.passphrase.clone()
    }
    pub fn blockhash(&self) -> String {
        self.blockhash.clone()
    }
    pub fn nonce(&self) -> Option<String> {
        self.nonce.clone()
    }
    pub fn seed(&self) -> Option<String> {
        self.seed.clone()
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PubkeyAndPhrase {
    pubkey: String,
    phrase: String,
}

#[wasm_bindgen]
impl PubkeyAndPhrase {
    #[wasm_bindgen(constructor)]
    pub fn new(pubkey: &str, phrase: &str) -> PubkeyAndPhrase {
        PubkeyAndPhrase {
            pubkey: pubkey.to_string(),
            phrase: phrase.to_string(),
        }
    }
    #[wasm_bindgen(getter)]
    pub fn pubkey(&self) -> String {
        self.pubkey.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn phrase(&self) -> String {
        self.phrase.clone()
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PubkeyAndEncodedTransaction {
    pubkey: String,
    encoded: String,
}

#[wasm_bindgen]
impl PubkeyAndEncodedTransaction {
    #[wasm_bindgen(constructor)]
    pub fn new(pubkey: &str, encoded: &str) -> PubkeyAndEncodedTransaction {
        PubkeyAndEncodedTransaction {
            pubkey: pubkey.to_string(),
            encoded: encoded.to_string(),
        }
    }
    #[wasm_bindgen(getter)]
    pub fn pubkey(&self) -> String {
        self.pubkey.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn encoded(&self) -> String {
        self.encoded.clone()
    }
}
