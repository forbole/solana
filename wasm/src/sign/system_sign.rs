use crate::sign::{serialize_encode_transaction};
use solana_sdk::{
    hash::Hash,
    signature::{Signer, keypair_from_seed_phrase_and_passphrase},
    pubkey::Pubkey,
    message::Message,
    transaction::Transaction,
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
    let message = Message::new(&[instruction], Some(&from_keypair.pubkey()));
    let signers = [&from_keypair];
    let tx = Transaction::new(&signers, message, recent_hash);
    Ok(serialize_encode_transaction(&tx))
}


#[cfg(test)]
mod test {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_transfer(){
        let hash = "3r1DbHt5RtsQfdDMyLaeBkoQqMcn3m4S4kDLFj4YHvae";
        let phrase =
            "plunge bitter method anchor slogan talent draft obscure mimic hover ordinary tiny";
        let passphrase = "";
        let to = "FPYSXfvJ24mCk9f8bX8zgtWYKnvgf96upeSaNraEtuk9";
        transfer(hash, phrase, passphrase, to, 100).unwrap();
    }
}