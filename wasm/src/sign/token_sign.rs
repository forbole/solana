use crate::{sign::serialize_encode_transaction, types::PubkeyAndEncodedTransaction};
use solana_program::{program_pack::Pack, system_instruction};
use solana_sdk::{
    hash::Hash,
    message::Message,
    signature::{keypair_from_seed_phrase_and_passphrase, Keypair, Signer},
    transaction::Transaction,
};
use spl_token::{instruction as spl_token_instruction, state::Mint};
use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "getTokenLength")]
pub fn get_token_size() -> usize {
    Mint::LEN
}

#[wasm_bindgen(js_name = "createToken")]
pub fn create_token(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    decimals: u8,
    minimum_balance_for_rent_exemption: i32,
    enable_freeze: bool,
) -> Result<JsValue, JsValue> {
    let from_keypair = keypair_from_seed_phrase_and_passphrase(phrase, passphrase).unwrap();
    let from_pubkey = from_keypair.pubkey();
    let token_keypair = Keypair::new();
    let token_pubkey = token_keypair.pubkey();
    let freeze_authority_pubkey = if enable_freeze {
        Some(from_pubkey)
    } else {
        None
    };

    let instructions = vec![
        system_instruction::create_account(
            &from_pubkey,
            &token_pubkey,
            minimum_balance_for_rent_exemption as u64,
            Mint::LEN as u64,
            &spl_token::id(),
        ),
        spl_token_instruction::initialize_mint(
            &spl_token::id(),
            &token_pubkey,
            &from_pubkey,
            freeze_authority_pubkey.as_ref(),
            decimals,
        )
        .unwrap(),
    ];
    let recent_hash = Hash::from_str(blockhash).unwrap();
    let signers = [&from_keypair, &from_keypair, &token_keypair];
    let message = Message::new(&instructions, Some(&from_keypair.pubkey()));
    let tx = Transaction::new(&signers, message, recent_hash);

    let result = PubkeyAndEncodedTransaction {
        pubkey: token_pubkey.to_string(),
        encoded: serialize_encode_transaction(&tx),
    };
    Ok(JsValue::from_serde(&result).unwrap())
}

#[cfg(test)]
mod test {
    use super::*;
    use wasm_bindgen_test::*;
    #[wasm_bindgen_test]
    fn test_create_token() {
        let hash = "3r1DbHt5RtsQfdDMyLaeBkoQqMcn3m4S4kDLFj4YHvae";
        let phrase =
            "plunge bitter method anchor slogan talent draft obscure mimic hover ordinary tiny";
        let passphrase = "";
        create_token(hash, phrase, passphrase, 9, 100, false).unwrap();
    }
}
