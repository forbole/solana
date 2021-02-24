use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

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
    #[wasm_bindgen(setter)]
    pub fn set_pubkey(&mut self, pubkey: String) {
        self.pubkey = pubkey;
    }
    #[wasm_bindgen(getter)]
    pub fn phrase(&self) -> String {
        self.phrase.clone()
    }
    #[wasm_bindgen(setter)]
    pub fn set_phrase(&mut self, phrase: String) {
        self.phrase = phrase;
    }
}

#[wasm_bindgen(skip)]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PubkeyAndEncodedTransaction {
    #[wasm_bindgen(skip)]
    pub pubkey: String,
    #[wasm_bindgen(skip)]
    pub encoded: String,
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
    #[wasm_bindgen(setter)]
    pub fn set_pubkey(&mut self, pubkey: String) {
        self.pubkey = pubkey;
    }
    #[wasm_bindgen(getter)]
    pub fn encoded(&self) -> String {
        self.encoded.clone()
    }
    #[wasm_bindgen(setter)]
    pub fn set_encoded(&mut self, phrase: String) {
        self.encoded = phrase;
    }
}