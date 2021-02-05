use crate::key::generate_seed;
use solana_sdk::{
    hash::Hash,
    message::Message,
    signature::{Keypair, Signer},
    transaction::Transaction,
    pubkey::Pubkey,
};
use solana_program::system_instruction;
use ed25519_dalek::{SecretKey, PublicKey, Keypair as DalekKeypair};
use bincode::serialize;
use base64;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "transfer")]
pub fn transfer(
    phrase: &str,
    passphrase: &str,
    to: &str,
    lamports: u32,
    blockhash: &str,
) -> Result<String, JsValue> {
    let seed = generate_seed(phrase, passphrase);
    let secret = SecretKey::from_bytes(&seed).unwrap();
    let public = PublicKey::from(&secret);
    let bytes = DalekKeypair{secret, public}.to_bytes();
    let from_keypair = Keypair::from_bytes(&bytes).unwrap();

    let from_pubkey = from_keypair.pubkey();
    let to_pubkey = Pubkey::from_str(to).unwrap();

    let instruction = system_instruction::transfer(&from_pubkey, &to_pubkey, lamports as u64);
    let message = Message::new(&[instruction], Some(&from_pubkey));
    let recent_hash = Hash::from_str(blockhash).unwrap();
    let tx = Transaction::new(&[&from_keypair], message, recent_hash);
    Ok(serialize_encode_transaction(&tx))
}

fn serialize_encode_transaction(transaction: &Transaction) -> String {
    let serialized = serialize(transaction).unwrap();
    let encoded = base64::encode(serialized);

    encoded
}
