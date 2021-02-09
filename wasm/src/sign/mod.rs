use solana_sdk::{
    hash::Hash,
    signers::Signers,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use bincode::serialize;
use base64;

pub mod system_sign;
pub mod stake_sign;


fn serialize_encode_transaction(transaction: &Transaction) -> String {
    let serialized = serialize(transaction).unwrap();
    let encoded = base64::encode(serialized);

    encoded
}