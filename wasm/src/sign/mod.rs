use solana_sdk::{
    transaction::Transaction,
};
use bincode::serialize;
use base64;

pub mod system_sign;
pub mod stake_sign;
pub mod token_sign;


fn serialize_encode_transaction(transaction: &Transaction) -> String {
    let serialized = serialize(transaction).unwrap();
    let encoded = base64::encode(serialized);

    encoded
}