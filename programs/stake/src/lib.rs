#![cfg_attr(RUSTC_WITH_SPECIALIZATION, feature(specialization))]

#[cfg(not(target_arch = "wasm32"))]
use solana_sdk::genesis_config::GenesisConfig;

pub mod config;
pub mod stake_instruction;
pub mod stake_state;

solana_sdk::declare_id!("Stake11111111111111111111111111111111111111");

#[cfg(not(target_arch = "wasm32"))]
pub fn add_genesis_accounts(genesis_config: &mut GenesisConfig) -> u64 {
    config::add_genesis_account(genesis_config)
}

#[cfg(not(target_arch = "wasm32"))]
#[macro_use]
extern crate solana_frozen_abi_macro;
