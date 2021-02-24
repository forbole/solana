use crate::{jserr, sign::generate_encoded_transaction, types::PubkeyAndEncodedTransaction};
use solana_program::{program_pack::Pack, rent::Rent, system_instruction};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{keypair_from_seed_phrase_and_passphrase, Keypair, Signer},
};
use spl_token::{
    instruction as spl_token_instruction,
    state::{Account, Mint},
};
use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "createToken")]
pub fn create_token(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    decimals: u8,
    enable_freeze: bool,
) -> Result<JsValue, JsValue> {
    let authority_keypair = jserr!(keypair_from_seed_phrase_and_passphrase(phrase, passphrase));
    let authority_pubkey = authority_keypair.pubkey();
    let token_keypair = Keypair::new();
    let token_pubkey = token_keypair.pubkey();
    let freeze_authority_pubkey = if enable_freeze {
        Some(authority_pubkey)
    } else {
        None
    };
    let instructions = vec![
        system_instruction::create_account(
            &authority_pubkey,
            &token_pubkey,
            Rent::default().minimum_balance(Mint::LEN),
            Mint::LEN as u64,
            &spl_token::id(),
        ),
        jserr!(spl_token_instruction::initialize_mint(
            &spl_token::id(),
            &token_pubkey,
            &authority_pubkey,
            freeze_authority_pubkey.as_ref(),
            decimals,
        )),
    ];
    let signers = [&authority_keypair, &token_keypair];
    let encoded = jserr!(generate_encoded_transaction(
        blockhash,
        &instructions,
        &authority_pubkey,
        &signers
    ));
    let result = PubkeyAndEncodedTransaction {
        pubkey: token_pubkey.to_string(),
        encoded: encoded,
    };
    Ok(jserr!(JsValue::from_serde(&result)))
}

#[wasm_bindgen(js_name = "mintToken")]
pub fn mint_token(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    token: &str,
    recipient: &str,
    amount: u32,
    decimals: u8,
) -> Result<String, JsValue> {
    let authority_keypair = jserr!(keypair_from_seed_phrase_and_passphrase(phrase, passphrase));
    let authority_pubkey = authority_keypair.pubkey();
    let token_pubkey = jserr!(Pubkey::from_str(token));
    let recipient_pubkey = jserr!(Pubkey::from_str(recipient));
    let instructions = vec![jserr!(spl_token_instruction::mint_to_checked(
        &spl_token::id(),
        &token_pubkey,
        &recipient_pubkey,
        &authority_pubkey,
        &[],
        amount as u64,
        decimals,
    ))];
    let signers = [&authority_keypair];
    let encoded = jserr!(generate_encoded_transaction(
        blockhash,
        &instructions,
        &authority_pubkey,
        &signers
    ));
    Ok(encoded)
}

#[wasm_bindgen(js_name = "burnToken")]
pub fn burn_token(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    mint: &str,
    token_account: &str,
    amount: u32,
    decimals: u8,
) -> Result<String, JsValue> {
    let authority_keypair = jserr!(keypair_from_seed_phrase_and_passphrase(phrase, passphrase));
    let authority_pubkey = authority_keypair.pubkey();
    let token_account_pubkey = jserr!(Pubkey::from_str(token_account));
    let mint_pubkey = jserr!(Pubkey::from_str(mint));
    let instructions = vec![jserr!(spl_token_instruction::burn_checked(
        &spl_token::id(),
        &token_account_pubkey,
        &mint_pubkey,
        &authority_pubkey,
        &[],
        amount as u64,
        decimals,
    ))];
    let signers = [&authority_keypair];
    let encoded = jserr!(generate_encoded_transaction(
        blockhash,
        &instructions,
        &authority_pubkey,
        &signers
    ));
    Ok(encoded)
}

#[wasm_bindgen(js_name = "createTokenAccount")]
pub fn create_token_account(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    mint: &str,
) -> Result<JsValue, JsValue> {
    let authority_keypair = jserr!(keypair_from_seed_phrase_and_passphrase(phrase, passphrase));
    let authority_pubkey = authority_keypair.pubkey();
    let mint_pubkey = jserr!(Pubkey::from_str(mint));
    let account_keypair = Keypair::new();
    let account_pubkey = account_keypair.pubkey();
    let instructions = vec![
        system_instruction::create_account(
            &authority_pubkey,
            &account_pubkey,
            Rent::default().minimum_balance(Account::LEN),
            Account::LEN as u64,
            &spl_token::id(),
        ),
        jserr!(spl_token_instruction::initialize_account(
            &spl_token::id(),
            &account_pubkey,
            &mint_pubkey,
            &authority_pubkey,
        )),
    ];
    let signers = [&authority_keypair, &account_keypair];
    let encoded = jserr!(generate_encoded_transaction(
        blockhash,
        &instructions,
        &authority_pubkey,
        &signers
    ));
    let result = PubkeyAndEncodedTransaction {
        pubkey: account_pubkey.to_string(),
        encoded: encoded,
    };
    Ok(jserr!(JsValue::from_serde(&result)))
}

#[wasm_bindgen(js_name = "transferToken")]
pub fn transfer_token(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    mint: &str,
    source: &str,
    destination: &str,
    amount: u32,
    decimals: u8,
) -> Result<String, JsValue> {
    let authority_keypair = jserr!(keypair_from_seed_phrase_and_passphrase(phrase, passphrase));
    let authority_pubkey = authority_keypair.pubkey();
    let source_pubkey = jserr!(Pubkey::from_str(source));
    let mint_pubkey = jserr!(Pubkey::from_str(mint));
    let destination_pubkey = jserr!(Pubkey::from_str(destination));
    let instructions = vec![jserr!(spl_token_instruction::transfer_checked(
        &spl_token::id(),
        &source_pubkey,
        &mint_pubkey,
        &destination_pubkey,
        &authority_pubkey,
        &[],
        amount as u64,
        decimals,
    ))];
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

    static BLOCKHASH : &str = "3r1DbHt5RtsQfdDMyLaeBkoQqMcn3m4S4kDLFj4YHvae";
    static PHRASE : &str = "plunge bitter method anchor slogan talent draft obscure mimic hover ordinary tiny";
    static PASSPHRASE : &str = "";

    #[wasm_bindgen_test]
    fn test_create_token() {
        create_token(BLOCKHASH, PHRASE, PASSPHRASE, 9, false).unwrap();
    }
    #[wasm_bindgen_test]
    fn test_mint_token() {
        let token = Pubkey::new_unique().to_string();
        let account = Pubkey::new_unique().to_string();
        mint_token(BLOCKHASH, PHRASE, PASSPHRASE, &token, &account, 100, 6).unwrap();
    }
    #[wasm_bindgen_test]
    fn test_burn_token() {
        let token = Pubkey::new_unique().to_string();
        let account = Pubkey::new_unique().to_string();
        burn_token(BLOCKHASH, PHRASE, PASSPHRASE, &token, &account, 100, 6).unwrap();
    }
    #[wasm_bindgen_test]
    fn test_create_token_account() {
        let token = Pubkey::new_unique().to_string();
        create_token_account(BLOCKHASH, PHRASE, PASSPHRASE, &token).unwrap();
    }
    #[wasm_bindgen_test]
    fn test_transfer_token() {
        let source = Pubkey::new_unique().to_string();
        let token = Pubkey::new_unique().to_string();
        let destination = Pubkey::new_unique().to_string();
        transfer_token(
            BLOCKHASH,
            PHRASE,
            PASSPHRASE,
            &token,
            &source,
            &destination,
            100,
            6,
        )
        .unwrap();
    }
}
