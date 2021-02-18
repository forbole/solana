use crate::{sign::serialize_encode_transaction, types::PubkeyAndEncodedTransaction};
use solana_sdk::{
    hash::Hash,
    message::Message,
    pubkey::Pubkey,
    signature::{keypair_from_seed_phrase_and_passphrase, Keypair, Signer},
    transaction::Transaction,
};
use solana_stake_program::{
    stake_instruction,
    stake_state::{Authorized, Lockup},
};
use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "createStakeAccount")]
pub fn create_stake_account(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    lamports: u32,
) -> Result<JsValue, JsValue> {
    let from_keypair = keypair_from_seed_phrase_and_passphrase(phrase, passphrase).unwrap();
    let from_pubkey = from_keypair.pubkey();
    let stake_keypair = Keypair::new();
    let authorized = Authorized {
        staker: from_pubkey,
        withdrawer: from_pubkey,
    };

    let lockup = Lockup::default();
    let instructions = stake_instruction::create_account(
        &from_pubkey,
        &stake_keypair.pubkey(),
        &authorized,
        &lockup,
        lamports as u64,
    );
    let recent_hash = Hash::from_str(blockhash).unwrap();
    let message = Message::new(&instructions, Some(&from_keypair.pubkey()));
    let signers = [&from_keypair, &from_keypair, &stake_keypair];
    let tx = Transaction::new(&signers, message, recent_hash);

    let result = PubkeyAndEncodedTransaction{
        pubkey: stake_keypair.pubkey().to_string(),
        encoded: serialize_encode_transaction(&tx)
    };
    Ok(JsValue::from_serde(&result).unwrap())
}

#[wasm_bindgen(js_name = "delegateStake")]
pub fn delegate_stake(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    stake_account: &str,
    validator: &str,
) -> Result<String, JsValue> {
    let from_keypair = keypair_from_seed_phrase_and_passphrase(phrase, passphrase).unwrap();
    let from_pubkey = from_keypair.pubkey();

    let stake_pubkey = Pubkey::from_str(stake_account).unwrap();
    let validator_pubkey = Pubkey::from_str(validator).unwrap();

    let instruction =
        stake_instruction::delegate_stake(&stake_pubkey, &from_pubkey, &validator_pubkey);
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
    fn test_create_stake_account() {
        let hash = "3r1DbHt5RtsQfdDMyLaeBkoQqMcn3m4S4kDLFj4YHvae";
        let phrase =
            "plunge bitter method anchor slogan talent draft obscure mimic hover ordinary tiny";
        let passphrase = "";
        create_stake_account(hash, phrase, passphrase, 100).unwrap();
    }

    #[wasm_bindgen_test]
    fn test_delegate_stake() {
        let hash = "3r1DbHt5RtsQfdDMyLaeBkoQqMcn3m4S4kDLFj4YHvae";
        let phrase =
            "plunge bitter method anchor slogan talent draft obscure mimic hover ordinary tiny";
        let passphrase = "";
        let stake_account = Pubkey::new_unique().to_string();
        let validator = Pubkey::new_unique().to_string();
        delegate_stake(hash, phrase, passphrase, &stake_account, &validator).unwrap();
    }
}
