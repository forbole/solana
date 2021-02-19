#![cfg_attr(RUSTC_WITH_SPECIALIZATION, feature(specialization))]
#![allow(clippy::integer_arithmetic)]

pub mod authorized_voters;
pub mod vote_instruction;
pub mod vote_state;
pub mod vote_transaction;

#[cfg(not(target_arch = "wasm32"))]
#[macro_use]
extern crate solana_metrics;

#[cfg(not(target_arch = "wasm32"))]
#[macro_use]
extern crate solana_frozen_abi_macro;

solana_sdk::declare_id!("Vote111111111111111111111111111111111111111");
