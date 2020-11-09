use solana_client::{
    rpc_response::RpcVoteAccountInfo,
    rpc_response::RpcVoteAccountStatus,
};
use solana_sdk::{
    pubkey::Pubkey,
};

use std::{
    collections::HashSet,
    str::FromStr,
};

use crate::Config;

// for filter validator by stake percentage, quality and commission
fn filter_validators(
    config: &Config,
    vote: RpcVoteAccountInfo,
    quality_block_producers: &HashSet<Pubkey>,
    total_activated_stake: u64
) -> Option<RpcVoteAccountInfo> {
    let node_pubkey = Pubkey::from_str(&vote.node_pubkey).ok()?;
    let is_quality_producers = quality_block_producers.contains(&node_pubkey);
    let activated_stake_percentage: f64 = 100.0 * vote.activated_stake as f64 / total_activated_stake as f64;
    let is_stake_percentage_in_range = activated_stake_percentage <= config.stake_percentage_cap;
    let is_commission_rate_in_range = vote.commission <= config.commission_cap; 
    let is_over_min_stake_required = vote.activated_stake > 500;

    let is_fit_all_conditions = is_quality_producers 
        && is_stake_percentage_in_range
        && is_over_min_stake_required
        && is_commission_rate_in_range;
    
    if is_fit_all_conditions {
        Some(vote)
    } else {
        None
    }
}

// generate validator hashset for generate transactions step
pub fn generate_validator_list(
    config: &Config,
    vote_account_status: &RpcVoteAccountStatus,
    quality_block_producers: &HashSet<Pubkey>,
) -> HashSet<Pubkey> {
    let mut validator_list = config.validator_list.clone();
    if validator_list.len() >= config.validator_min_length{
        return validator_list;
    }
    // caculate total activated_stake in validators
     let total_activated_stake = vote_account_status
        .clone()
        .current
        .into_iter()
        .chain(vote_account_status.delinquent.clone().into_iter())
        .fold(0, |acc, vote| acc + vote.activated_stake);

    // filter producers by quality, stake percentage
    let mut quality_producers_info = vote_account_status
        .clone()
        .current
        .into_iter()
        .filter_map(|vote| {
            filter_validators(config, vote, quality_block_producers, total_activated_stake)
        })
        .collect::<Vec<_>>();

    while validator_list.len() < config.validator_min_length { 
        if quality_producers_info.len() == 0 {
            break;
        }
        let validator = quality_producers_info.pop().unwrap();
        let node_pubkey = Pubkey::from_str(&validator.node_pubkey).ok().unwrap();
        validator_list.insert(node_pubkey);
    }
    return validator_list;
}