use crate::sign::{generate_transaction_with_instruction_and_hash, serialize_encode_transaction};
use solana_sdk::{
    hash::Hash,
    signature::{Signer, keypair_from_seed_phrase_and_passphrase},
    pubkey::Pubkey,
};
use solana_program::system_instruction;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "transfer")]
pub fn transfer(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    to: &str,
    lamports: u32,
) -> Result<String, JsValue> {
    let from_keypair = keypair_from_seed_phrase_and_passphrase(phrase, passphrase).unwrap();
    let from_pubkey = from_keypair.pubkey();
    let to_pubkey = Pubkey::from_str(to).unwrap();

    let instruction = system_instruction::transfer(&from_pubkey, &to_pubkey, lamports as u64);
    let recent_hash = Hash::from_str(blockhash).unwrap();
    let tx = generate_transaction_with_instruction_and_hash(&from_keypair, &[instruction], recent_hash);
    Ok(serialize_encode_transaction(&tx))
}


