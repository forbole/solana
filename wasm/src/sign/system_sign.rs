use crate::{jserr, sign::generate_encoded_transaction};
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
    let authority_keypair = jserr!(keypair_from_seed_phrase_and_passphrase(phrase, passphrase));
    let authority_pubkey = authority_keypair.pubkey();
    let to_pubkey = jserr!(Pubkey::from_str(to));
    let instructions = vec![system_instruction::transfer(
        &authority_pubkey,
        &to_pubkey,
        lamports as u64,
    )];
    let signers = [&authority_keypair];
    let encoded_tx = jserr!(generate_encoded_transaction(
        blockhash,
        &instructions,
        &authority_pubkey,
        &signers
    ));
    Ok(encoded_tx)
}

#[cfg(test)]
mod test {
    use super::*;
    use wasm_bindgen_test::*;

    static BLOCKHASH : &str = "3r1DbHt5RtsQfdDMyLaeBkoQqMcn3m4S4kDLFj4YHvae";
    static PHRASE : &str = "plunge bitter method anchor slogan talent draft obscure mimic hover ordinary tiny";
    static PASSPHRASE : &str = "";

    #[wasm_bindgen_test]
    fn test_transfer() {
        let to = Pubkey::new_unique().to_string();
        transfer(BLOCKHASH, PHRASE, PASSPHRASE, &to, 100).unwrap();
    }
}
