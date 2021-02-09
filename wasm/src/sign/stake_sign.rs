use crate::{
    sign::{serialize_encode_transaction},
    types::PubkeyAndEncodedTransaction
};
use solana_stake_program::{
    stake_instruction,
    stake_state::{Authorized, Lockup}
};
use solana_sdk::{
    hash::Hash,
    signature::{Signer, keypair_from_seed_phrase_and_passphrase},
    pubkey::Pubkey,
    transaction::Transaction,
    message::Message
};
use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "createStakeAccount")]
pub fn create_stake_account(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    lamports: u32
) -> Result<JsValue, JsValue> {
    let from_keypair = keypair_from_seed_phrase_and_passphrase(phrase, passphrase).unwrap();
    let from_pubkey = from_keypair.pubkey();
    let stake_pubkey = Pubkey::new_unique();

    let authorized = Authorized{
        staker: from_pubkey,
        withdrawer: from_pubkey,
    };

    let lockup = Lockup::default();
    let instructions = stake_instruction::create_account(
        &from_pubkey,
        &stake_pubkey,
        &authorized,
        &lockup,
        lamports as u64
    );
    let recent_hash = Hash::from_str(blockhash).unwrap();
    let message = Message::new(&instructions, Some(&from_pubkey));
    let tx = Transaction::new(&[&from_keypair], message, recent_hash);
    let result = PubkeyAndEncodedTransaction{
        pubkey: stake_pubkey.to_string(),
        encoded: serialize_encode_transaction(&tx)
    };

    Ok( JsValue::from_serde(&result).unwrap())
}

// #[wasm_bindgen(js_name = "delegateStake")]
// pub fn delegate_stake(
//     blockhash: &str,
//     phrase: &str,
//     passphrase: &str,
//     lamports: u32
// ) -> 