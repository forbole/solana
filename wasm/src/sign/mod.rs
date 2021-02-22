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
    blockhash: &str,
    instructions: &[Instruction],
    authority_pubkey: &Pubkey,
    signers: &T,
) -> String {
    let recent_hash = Hash::from_str(blockhash).unwrap();
    let message = Message::new(instructions, Some(authority_pubkey));
    let tx = Transaction::new(signers, message, recent_hash);
    serialize_encode_transaction(&tx)
}

fn serialize_encode_transaction(transaction: &Transaction) -> String {
    let serialized = serialize(transaction).unwrap();
    let encoded = base64::encode(serialized);
    encoded
}
