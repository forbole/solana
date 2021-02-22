use crate::{sign::generate_encoded_transaction, types::PubkeyAndEncodedTransaction};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{keypair_from_seed_phrase_and_passphrase, Keypair, Signer},
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
    let authority_keypair = keypair_from_seed_phrase_and_passphrase(phrase, passphrase).unwrap();
    let authority_pubkey = authority_keypair.pubkey();
    let stake_keypair = Keypair::new();
    let authorized = Authorized {
        staker: authority_pubkey,
        withdrawer: authority_pubkey,
    };
    let lockup = Lockup::default();
    let instructions = stake_instruction::create_account(
        &authority_pubkey,
        &stake_keypair.pubkey(),
        &authorized,
        &lockup,
        lamports as u64,
    );
    let signers = [&authority_keypair, &stake_keypair];
    let encoded =
        generate_encoded_transaction(blockhash, &instructions, &authority_pubkey, &signers);
    let result = PubkeyAndEncodedTransaction {
        pubkey: stake_keypair.pubkey().to_string(),
        encoded: encoded,
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
    let authority_keypair = keypair_from_seed_phrase_and_passphrase(phrase, passphrase).unwrap();
    let authority_pubkey = authority_keypair.pubkey();
    let stake_pubkey = Pubkey::from_str(stake_account).unwrap();
    let validator_pubkey = Pubkey::from_str(validator).unwrap();
    let instructions = vec![stake_instruction::delegate_stake(
        &stake_pubkey,
        &authority_pubkey,
        &validator_pubkey,
    )];
    let signers = [&authority_keypair];
    let encoded =
        generate_encoded_transaction(blockhash, &instructions, &authority_pubkey, &signers);
    Ok(encoded)
}

#[wasm_bindgen(js_name = "deactivateStake")]
pub fn deactivate_stake(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    stake_account: &str,
) -> Result<String, JsValue> {
    let authority_keypair = keypair_from_seed_phrase_and_passphrase(phrase, passphrase).unwrap();
    let authority_pubkey = authority_keypair.pubkey();
    let stake_pubkey = Pubkey::from_str(stake_account).unwrap();
    let instructions = vec![stake_instruction::deactivate_stake(
        &stake_pubkey,
        &authority_pubkey,
    )];
    let signers = [&authority_keypair];
    let encoded =
        generate_encoded_transaction(blockhash, &instructions, &authority_pubkey, &signers);
    Ok(encoded)
}

#[wasm_bindgen(js_name = "withdrawStake")]
pub fn withdraw_stake(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    stake_account: &str,
    lamports: u64,
) -> Result<String, JsValue> {
    let authority_keypair = keypair_from_seed_phrase_and_passphrase(phrase, passphrase).unwrap();
    let authority_pubkey = authority_keypair.pubkey();
    let stake_pubkey = Pubkey::from_str(stake_account).unwrap();
    let instructions = vec![stake_instruction::withdraw(
        &stake_pubkey,
        &authority_pubkey,
        &authority_pubkey,
        lamports as u64,
        None,
    )];
    let signers = [&authority_keypair];
    let encoded =
        generate_encoded_transaction(blockhash, &instructions, &authority_pubkey, &signers);
    Ok(encoded)
}

#[wasm_bindgen(js_name = "mergeStake")]
pub fn merge_stake(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    source: &str,
    destination: &str,
) -> Result<String, JsValue> {
    let authority_keypair = keypair_from_seed_phrase_and_passphrase(phrase, passphrase).unwrap();
    let authority_pubkey = authority_keypair.pubkey();
    let source_pubkey = Pubkey::from_str(source).unwrap();
    let destination_pubkey = Pubkey::from_str(destination).unwrap();
    let instructions =
        stake_instruction::merge(&destination_pubkey, &source_pubkey, &authority_pubkey);
    let signers = [&authority_keypair];
    let encoded =
        generate_encoded_transaction(blockhash, &instructions, &authority_pubkey, &signers);
    Ok(encoded)
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

    #[wasm_bindgen_test]
    fn test_deactivate_stake() {
        let hash = "3r1DbHt5RtsQfdDMyLaeBkoQqMcn3m4S4kDLFj4YHvae";
        let phrase =
            "plunge bitter method anchor slogan talent draft obscure mimic hover ordinary tiny";
        let passphrase = "";
        let stake_account = Pubkey::new_unique().to_string();
        deactivate_stake(hash, phrase, passphrase, &stake_account).unwrap();
    }

    #[wasm_bindgen_test]
    fn test_withdraw_stake() {
        let hash = "3r1DbHt5RtsQfdDMyLaeBkoQqMcn3m4S4kDLFj4YHvae";
        let phrase =
            "plunge bitter method anchor slogan talent draft obscure mimic hover ordinary tiny";
        let passphrase = "";
        let stake_account = Pubkey::new_unique().to_string();
        withdraw_stake(hash, phrase, passphrase, &stake_account, 100).unwrap();
    }

    #[wasm_bindgen_test]
    fn test_merge_stake() {
        let hash = "3r1DbHt5RtsQfdDMyLaeBkoQqMcn3m4S4kDLFj4YHvae";
        let phrase =
            "plunge bitter method anchor slogan talent draft obscure mimic hover ordinary tiny";
        let passphrase = "";
        let source = Pubkey::new_unique().to_string();
        let destination = Pubkey::new_unique().to_string();
        merge_stake(hash, phrase, passphrase, &source, &destination).unwrap();
    }
}
