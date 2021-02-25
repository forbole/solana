use crate::{jserr, sign::generate_encoded_transaction, types::PubkeyAndEncodedTransaction};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{keypair_from_seed_phrase_and_passphrase, Keypair, Signer},
};
use solana_stake_program::{
    stake_instruction,
    stake_state::{Authorized, Lockup, StakeAuthorize},
};
use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub enum StakeAuthorizeInput {
    Staker,
    Withdrawer,
}

impl StakeAuthorizeInput {
    fn into(&self) -> StakeAuthorize {
        match self {
            StakeAuthorizeInput::Staker => StakeAuthorize::Staker,
            StakeAuthorizeInput::Withdrawer => StakeAuthorize::Withdrawer,
        }
    }
}

#[wasm_bindgen(js_name = "createStakeAccount")]
pub fn create_stake_account(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    lamports: u32,
) -> Result<JsValue, JsValue> {
    let authority_keypair = jserr!(keypair_from_seed_phrase_and_passphrase(phrase, passphrase));
    let authority_pubkey = authority_keypair.pubkey();
    let stake_keypair = Keypair::new();
    let stake_pubkey = stake_keypair.pubkey();
    let authorized = Authorized {
        staker: authority_pubkey,
        withdrawer: authority_pubkey,
    };
    let lockup = Lockup::default();
    let instructions = stake_instruction::create_account(
        &authority_pubkey,
        &stake_pubkey,
        &authorized,
        &lockup,
        lamports as u64,
    );
    let signers = [&authority_keypair, &stake_keypair];
    let encoded = jserr!(generate_encoded_transaction(
        blockhash,
        &instructions,
        &authority_pubkey,
        &signers
    ));
    let result = PubkeyAndEncodedTransaction::new(&stake_pubkey.to_string(), &encoded);
    Ok(jserr!(JsValue::from_serde(&result)))
}

#[wasm_bindgen(js_name = "delegateStake")]
pub fn delegate_stake(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    stake_account: &str,
    validator: &str,
) -> Result<String, JsValue> {
    let authority_keypair = jserr!(keypair_from_seed_phrase_and_passphrase(phrase, passphrase));
    let authority_pubkey = authority_keypair.pubkey();
    let stake_pubkey = jserr!(Pubkey::from_str(stake_account));
    let validator_pubkey = jserr!(Pubkey::from_str(validator));
    let instructions = vec![stake_instruction::delegate_stake(
        &stake_pubkey,
        &authority_pubkey,
        &validator_pubkey,
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

#[wasm_bindgen(js_name = "deactivateStake")]
pub fn deactivate_stake(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    stake_account: &str,
) -> Result<String, JsValue> {
    let authority_keypair = jserr!(keypair_from_seed_phrase_and_passphrase(phrase, passphrase));
    let authority_pubkey = authority_keypair.pubkey();
    let stake_pubkey = Pubkey::from_str(stake_account).unwrap();
    let instructions = vec![stake_instruction::deactivate_stake(
        &stake_pubkey,
        &authority_pubkey,
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

#[wasm_bindgen(js_name = "withdrawStake")]
pub fn withdraw_stake(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    stake_account: &str,
    lamports: u32,
) -> Result<String, JsValue> {
    let authority_keypair = jserr!(keypair_from_seed_phrase_and_passphrase(phrase, passphrase));
    let authority_pubkey = authority_keypair.pubkey();
    let stake_pubkey = jserr!(Pubkey::from_str(stake_account));
    let instructions = vec![stake_instruction::withdraw(
        &stake_pubkey,
        &authority_pubkey,
        &authority_pubkey,
        lamports as u64,
        None,
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

#[wasm_bindgen(js_name = "mergeStake")]
pub fn merge_stake(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    source: &str,
    destination: &str,
) -> Result<String, JsValue> {
    let authority_keypair = jserr!(keypair_from_seed_phrase_and_passphrase(phrase, passphrase));
    let authority_pubkey = authority_keypair.pubkey();
    let source_pubkey = jserr!(Pubkey::from_str(source));
    let destination_pubkey = jserr!(Pubkey::from_str(destination));
    let instructions =
        stake_instruction::merge(&destination_pubkey, &source_pubkey, &authority_pubkey);
    let signers = [&authority_keypair];
    let encoded = jserr!(generate_encoded_transaction(
        blockhash,
        &instructions,
        &authority_pubkey,
        &signers
    ));
    Ok(encoded)
}

#[wasm_bindgen(js_name = "splitStake")]
pub fn split_stake(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    source: &str,
    lamports: u32,
) -> Result<JsValue, JsValue> {
    let authority_keypair = jserr!(keypair_from_seed_phrase_and_passphrase(phrase, passphrase));
    let authority_pubkey = authority_keypair.pubkey();
    let source_pubkey = jserr!(Pubkey::from_str(source));
    let split_keypair = Keypair::new();
    let split_pubkey = split_keypair.pubkey();
    let instructions = stake_instruction::split(
        &source_pubkey,
        &authority_pubkey,
        lamports as u64,
        &split_pubkey,
    );
    let signers = [&authority_keypair, &split_keypair];
    let encoded = jserr!(generate_encoded_transaction(
        blockhash,
        &instructions,
        &authority_pubkey,
        &signers
    ));
    let result = PubkeyAndEncodedTransaction::new(&split_pubkey.to_string(), &encoded);
    Ok(jserr!(JsValue::from_serde(&result)))
}

#[wasm_bindgen(js_name = "authorizeStake")]
pub fn authorize_stake(
    blockhash: &str,
    phrase: &str,
    passphrase: &str,
    source: &str,
    new_authority: &str,
    authorize_type: StakeAuthorizeInput,
) -> Result<String, JsValue> {
    let authority_keypair = jserr!(keypair_from_seed_phrase_and_passphrase(phrase, passphrase));
    let authority_pubkey = authority_keypair.pubkey();
    let source_pubkey = jserr!(Pubkey::from_str(source));
    let new_authoriy_pubkey = jserr!(Pubkey::from_str(new_authority));
    let stake_authorize = StakeAuthorizeInput::into(&authorize_type);
    let instructions = vec![stake_instruction::authorize(
        &source_pubkey,
        &authority_pubkey,
        &new_authoriy_pubkey,
        stake_authorize,
        None,
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
    fn test_create_stake_account() {
        create_stake_account(BLOCKHASH, PHRASE, PASSPHRASE, 100).unwrap();
    }

    #[wasm_bindgen_test]
    fn test_delegate_stake() {
        let stake_account = Pubkey::new_unique().to_string();
        let validator = Pubkey::new_unique().to_string();
        delegate_stake(BLOCKHASH, PHRASE, PASSPHRASE, &stake_account, &validator).unwrap();
    }

    #[wasm_bindgen_test]
    fn test_deactivate_stake() {
        let stake_account = Pubkey::new_unique().to_string();
        deactivate_stake(BLOCKHASH, PHRASE, PASSPHRASE, &stake_account).unwrap();
    }

    #[wasm_bindgen_test]
    fn test_withdraw_stake() {
        let stake_account = Pubkey::new_unique().to_string();
        withdraw_stake(BLOCKHASH, PHRASE, PASSPHRASE, &stake_account, 100).unwrap();
    }

    #[wasm_bindgen_test]
    fn test_merge_stake() {
        let source = Pubkey::new_unique().to_string();
        let destination = Pubkey::new_unique().to_string();
        merge_stake(BLOCKHASH, PHRASE, PASSPHRASE, &source, &destination).unwrap();
    }
    #[wasm_bindgen_test]
    fn test_split_stake() {
        let source = Pubkey::new_unique().to_string();
        split_stake(BLOCKHASH, PHRASE, PASSPHRASE, &source, 100).unwrap();
    }
    #[wasm_bindgen_test]
    fn test_authorize_stake() {
        let source = Pubkey::new_unique().to_string();
        let new_authority = Pubkey::new_unique().to_string();
        let mut authorize_type = StakeAuthorizeInput::Staker;
        authorize_stake(
            BLOCKHASH,
            PHRASE,
            PASSPHRASE,
            &source,
            &new_authority,
            authorize_type,
        )
        .unwrap();
        authorize_type = StakeAuthorizeInput::Withdrawer;
        authorize_stake(
            BLOCKHASH,
            PHRASE,
            PASSPHRASE,
            &source,
            &new_authority,
            authorize_type,
        )
        .unwrap();
    }
}
