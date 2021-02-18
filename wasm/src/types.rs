use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(skip)]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PubkeyAndPhrase{
    #[wasm_bindgen(skip)] 
    pub pubkey: String,
    #[wasm_bindgen(skip)] 
    pub phrase: String
}

#[wasm_bindgen(skip)]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PubkeyAndEncodedTransaction{
    #[wasm_bindgen(skip)]
    pub pubkey: String,
    #[wasm_bindgen(skip)] 
    pub encoded: String
}