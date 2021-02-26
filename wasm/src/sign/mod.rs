use crate::types::SignerConfig;
use base64;
use bincode::serialize;
use solana_sdk::{
    hash::Hash, instruction::Instruction, message::Message, pubkey::Pubkey, signers::Signers,
    transaction::Transaction,
};
use std::str::FromStr;

pub mod stake_sign;
pub mod system_sign;
pub mod token_sign;

fn generate_encoded_transaction<T: Signers>(
    config: &SignerConfig,
    instructions: &[Instruction],
    authority_pubkey: &Pubkey,
    signers: &T,
) -> Result<String, Box<dyn std::error::Error>> {
    let recent_hash = Hash::from_str(&config.blockhash().as_ref())?;
    let message = match config.nonce() {
        Some(nonce) => Message::new_with_nonce(
            instructions.to_vec(),
            Some(authority_pubkey),
            &Pubkey::from_str(&nonce)?,
            authority_pubkey,
        ),
        None => Message::new(instructions, Some(authority_pubkey)),
    };
    let mut tx = Transaction::new_unsigned(message);
    tx.try_sign(signers, recent_hash)?;
    Ok(serialize_encode_transaction(&tx)?)
}

fn serialize_encode_transaction(
    transaction: &Transaction,
) -> Result<String, Box<dyn std::error::Error>> {
    let serialized = serialize(transaction)?;
    let encoded = base64::encode(serialized);
    Ok(encoded)
}


#[cfg(test)]
mod test {
    use super::*;
    use wasm_bindgen_test::*;
    use crate::sign::system_sign::transfer;

    #[wasm_bindgen_test]
    fn test_nonce() {
        let blockhash: &str = "3r1DbHt5RtsQfdDMyLaeBkoQqMcn3m4S4kDLFj4YHvae";
        let phrase: &str =
            "plunge bitter method anchor slogan talent draft obscure mimic hover ordinary tiny";
        let passhprase: &str = "";
        let nonce = Some(String::from(Pubkey::new_unique().to_string()));
        let config = SignerConfig::new(blockhash, phrase, passhprase, nonce, None);
        let to = Pubkey::new_unique().to_string();
        transfer(&config, &to, 100).unwrap();
    }
}