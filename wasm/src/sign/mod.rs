use solana_sdk::{
    hash::Hash,
    message::Message,
    signature::{Keypair, Signer},
    transaction::Transaction,
    instruction::Instruction,
};
use bincode::serialize;
use base64;

pub mod system_sign;
pub mod stake_sign;

pub fn generate_transaction_with_instruction_and_hash(from_keypair: &Keypair, instructions: &[Instruction], recent_hash: Hash) -> Transaction {
    let from_pubkey = from_keypair.pubkey();
    let message = Message::new(instructions, Some(&from_pubkey));
    let tx = Transaction::new(&[from_keypair], message, recent_hash);
    tx
}

fn serialize_encode_transaction(transaction: &Transaction) -> String {
    let serialized = serialize(transaction).unwrap();
    let encoded = base64::encode(serialized);

    encoded
}