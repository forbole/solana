use crate::{jserr, sign::generate_encoded_transaction, types::PubkeyAndEncodedTransaction};
use solana_program::system_instruction;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{keypair_from_seed_phrase_and_passphrase, Keypair, Signer},
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
    let encoded = jserr!(generate_encoded_transaction(
        blockhash,
        &instructions,
        &authority_pubkey,
        &signers
    ));
    Ok(encoded)
}

#[wasm_bindgen(js_name = "createNonceAccount")]
pub fn create_nonce_account(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    lamports: u32,
) -> Result<JsValue, JsValue> {
    let authority_keypair = jserr!(keypair_from_seed_phrase_and_passphrase(phrase, passphrase));
    let authority_pubkey = authority_keypair.pubkey();
    let nonce_keypair = Keypair::new();
    let nonce_pubkey = nonce_keypair.pubkey();
    let instructions = system_instruction::create_nonce_account(
        &authority_pubkey,
        &nonce_pubkey,
        &authority_pubkey,
        lamports as u64,
    );
    let signers = [&authority_keypair, &nonce_keypair];
    let encoded = jserr!(generate_encoded_transaction(
        blockhash,
        &instructions,
        &authority_pubkey,
        &signers
    ));
    let result = PubkeyAndEncodedTransaction::new(&nonce_pubkey.to_string(), &encoded);
    Ok(jserr!(JsValue::from_serde(&result)))
}

#[wasm_bindgen(js_name = "withdrawNonce")]
pub fn withdraw_nonce(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    nonce_account: &str,
    lamports: u32,
) -> Result<String, JsValue> {
    let authority_keypair = jserr!(keypair_from_seed_phrase_and_passphrase(phrase, passphrase));
    let authority_pubkey = authority_keypair.pubkey();
    let nonce_pubkey = jserr!(Pubkey::from_str(nonce_account));
    let instructions = vec![system_instruction::withdraw_nonce_account(
        &nonce_pubkey,
        &authority_pubkey,
        &authority_pubkey,
        lamports as u64,
    )];
    let signers = [&authority_keypair];
    let encoded = jserr!(generate_encoded_transaction(
        blockhash,
        &instructions,
        &authority_pubkey,
        &signers
    ));
    Ok(encoded)
}

#[wasm_bindgen(js_name = "authorizeNonce")]
pub fn authorize_nonce(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    nonce_account: &str,
    new_authority: &str,
) -> Result<String, JsValue> {
    let authority_keypair = jserr!(keypair_from_seed_phrase_and_passphrase(phrase, passphrase));
    let authority_pubkey = authority_keypair.pubkey();
    let nonce_pubkey = jserr!(Pubkey::from_str(nonce_account));
    let new_authoriy_pubkey = jserr!(Pubkey::from_str(new_authority));
    let instructions = vec![system_instruction::authorize_nonce_account(
        &nonce_pubkey,
        &authority_pubkey,
        &new_authoriy_pubkey,
    )];
    let signers = [&authority_keypair];
    let encoded = jserr!(generate_encoded_transaction(
        blockhash,
        &instructions,
        &authority_pubkey,
        &signers
    ));
    Ok(encoded)
}

#[cfg(test)]
mod test {
    use super::*;
    use wasm_bindgen_test::*;

    static BLOCKHASH: &str = "3r1DbHt5RtsQfdDMyLaeBkoQqMcn3m4S4kDLFj4YHvae";
    static PHRASE: &str =
        "plunge bitter method anchor slogan talent draft obscure mimic hover ordinary tiny";
    static PASSPHRASE: &str = "";

    #[wasm_bindgen_test]
    fn test_transfer() {
        let to = Pubkey::new_unique().to_string();
        transfer(BLOCKHASH, PHRASE, PASSPHRASE, &to, 100).unwrap();
    }
    #[wasm_bindgen_test]
    fn test_create_nonce_account() {
        create_nonce_account(BLOCKHASH, PHRASE, PASSPHRASE, 100).unwrap();
    }
    #[wasm_bindgen_test]
    fn test_withdraw_nonce() {
        let nonce = Pubkey::new_unique().to_string();
        withdraw_nonce(BLOCKHASH, PHRASE, PASSPHRASE, &nonce, 100).unwrap();
    }
    #[wasm_bindgen_test]
    fn test_authorize_nonce() {
        let nonce = Pubkey::new_unique().to_string();
        let new_authority = Pubkey::new_unique().to_string();
        authorize_nonce(BLOCKHASH, PHRASE, PASSPHRASE, &nonce, &new_authority).unwrap();
    }
}
