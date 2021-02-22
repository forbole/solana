use crate::sign::generate_encoded_transaction;
use solana_program::system_instruction;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{keypair_from_seed_phrase_and_passphrase, Signer},
};
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
    let authority_keypair = keypair_from_seed_phrase_and_passphrase(phrase, passphrase).unwrap();
    let authority_pubkey = authority_keypair.pubkey();
    let to_pubkey = Pubkey::from_str(to).unwrap();
    let instructions = vec![system_instruction::transfer(
        &authority_pubkey,
        &to_pubkey,
        lamports as u64,
    )];
    let signers = [&authority_keypair];
    let encoded_tx =
        generate_encoded_transaction(blockhash, &instructions, &authority_pubkey, &signers);
    Ok(encoded_tx)
}

#[cfg(test)]
mod test {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_transfer() {
        let hash = "3r1DbHt5RtsQfdDMyLaeBkoQqMcn3m4S4kDLFj4YHvae";
        let phrase =
            "plunge bitter method anchor slogan talent draft obscure mimic hover ordinary tiny";
        let passphrase = "";
        let to = "FPYSXfvJ24mCk9f8bX8zgtWYKnvgf96upeSaNraEtuk9";
        transfer(hash, phrase, passphrase, to, 100).unwrap();
    }
}
