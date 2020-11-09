use log::*;
use solana_cli_output::display::format_labeled_address;
use solana_client::{rpc_client::RpcClient, rpc_response::RpcVoteAccountInfo};
use solana_metrics::datapoint_info;
use solana_sdk::{
    epoch_info::EpochInfo, message::Message, native_token::*, pubkey::Pubkey, signature::Signer,
    transaction::Transaction,
};
use solana_stake_program::stake_instruction;

use crate::utils::get_stake_account;
use crate::Config;
use std::{collections::HashSet, str::FromStr};
struct AccountStatus {
    is_exist: bool,
    is_deactivating: bool,
    is_undelegated: bool,
}

// check account delegation status via rpc
fn check_account_status(
    rpc_client: &RpcClient,
    epoch_info: &EpochInfo,
    stake_address: &Pubkey,
    config: &Config
) -> AccountStatus {
    let mut status = AccountStatus {
        is_exist: false,
        is_deactivating: false,
        is_undelegated: true,
    };
    let stake_amount = config.baseline_stake_amount;
    if let Ok((balance, stake_state)) = get_stake_account(&rpc_client, &stake_address) {
        status.is_exist = true;
        if balance != stake_amount {
            info!(
                "Unexpected balance in stake account {}: {}, expected {}",
                stake_address, balance, stake_amount
            );
        }
        if let Some(delegation) = stake_state.delegation() {
            status.is_undelegated = false;
            // epoch the stake was deactivating
            status.is_deactivating = delegation.deactivation_epoch == epoch_info.epoch;
            if !status.is_deactivating {
                let cool_down = 0;
                status.is_undelegated = delegation.deactivation_epoch + cool_down < epoch_info.epoch;
            }
        }
    }
    return status;
}
#[derive(Debug)]
enum AccountAction {
    None,
    Create,
    Delegate,
    Deactivate,
    Withdraw,
}

// set the account action for delegation process, and check if the validator is delinquent or not
fn get_accounts_action(
    root_slot: &u64,
    epoch_info: &EpochInfo,
    config: &Config,
    node_pubkey: &Pubkey,
    validator_is_qualified: bool,
    source_stake_lamports_required: &mut u64,
    baseline_status: AccountStatus
) -> (AccountAction, bool) {
    let formatted_node_pubkey =
        format_labeled_address(&node_pubkey.to_string(), &config.address_labels);
    let mut baseline_action = AccountAction::None;
    let mut is_long_term_unqualified = false;
    // Validator is considered delinquent if its root slot is less than delinquent_grace_slot_distance( 21600 ) slots behind the current
    // slot.  This is very generous.
    if *root_slot
        < epoch_info
            .absolute_slot
            .saturating_sub(config.delinquent_grace_slot_distance) || 
            !validator_is_qualified
    {
        if baseline_status.is_exist && baseline_status.is_undelegated {
            info!(
                "Need to withdraw baseline stake account from validator {}",
                formatted_node_pubkey
            );
            baseline_action = AccountAction::Withdraw;
            is_long_term_unqualified = true;
        } else if baseline_status.is_exist && !baseline_status.is_deactivating {
            info!(
                "Need to deactivate baseline stake account from validator {}",
                formatted_node_pubkey
            );
            baseline_action = AccountAction::Deactivate;
        } else if !baseline_status.is_exist {
            is_long_term_unqualified = true;
        }
    } else {
        // the action of baseline
        if !baseline_status.is_exist {
            info!(
                "Need to create baseline stake account for validator {}",
                formatted_node_pubkey
            );
            *source_stake_lamports_required += config.baseline_stake_amount;
            baseline_action = AccountAction::Create;
        } else if baseline_status.is_undelegated {
            info!(
                "Need to delegate baseline stake account to validator {}",
                formatted_node_pubkey
            );
            baseline_action = AccountAction::Delegate;
        }
    }
    return (baseline_action, is_long_term_unqualified);
}

// create transactions list to create and delegate accounts
pub fn generate_stake_transactions(
    vote_account_info: &Vec<RpcVoteAccountInfo>,
    config: &Config,
    rpc_client: &RpcClient,
    quality_block_producers: HashSet<Pubkey>,
    too_many_poor_block_producers: bool,
    epoch_info: &EpochInfo,
) -> (
    Vec<(Transaction, String)>,
    Vec<(Transaction, String)>,
    Vec<String>,
    u64,
) {
    let mut validator_list: Vec<String> = vec![];
    let mut source_stake_lamports_required = 0;
    let mut create_stake_transactions = vec![];
    let mut delegate_stake_transactions = vec![];
    for RpcVoteAccountInfo {
        vote_pubkey,
        node_pubkey,
        root_slot,
        ..
    } in vote_account_info
    {
        let formatted_node_pubkey = format_labeled_address(&node_pubkey, &config.address_labels);
        let node_pubkey = Pubkey::from_str(&node_pubkey).unwrap();
        let baseline_seed = &vote_pubkey.to_string()[..32];
        let vote_pubkey = Pubkey::from_str(&vote_pubkey).unwrap();
        let validator_is_qualified =
            !too_many_poor_block_producers && quality_block_producers.contains(&node_pubkey);

        let baseline_stake_address = Pubkey::create_with_seed(
            &config.authorized_staker.pubkey(),
            baseline_seed,
            &solana_stake_program::id(),
        )
        .unwrap();

        // Check baseline status
        let baseline_status = check_account_status(
            &rpc_client,
            &epoch_info,
            &baseline_stake_address,
            &config,
        );

        // Determine the action of baseline and bonus accounts
        let (mut baseline_action, is_long_term_unqualified) = get_accounts_action(
            &root_slot,
            &epoch_info,
            &config,
            &node_pubkey,
            validator_is_qualified,
            &mut source_stake_lamports_required,
            baseline_status,
        );

        datapoint_info!(
            "validator-status",
            ("cluster", config.cluster, String),
            ("id", node_pubkey.to_string(), String),
            ("slot", epoch_info.absolute_slot, i64),
            ("ok", !is_long_term_unqualified, bool)
        );

        // Create transaction to create account by actions
        if let AccountAction::Create = baseline_action {
            create_stake_transactions.push((
                Transaction::new_unsigned(Message::new(
                    &stake_instruction::split_with_seed(
                        &config.source_stake_address,
                        &config.authorized_staker.pubkey(),
                        config.baseline_stake_amount,
                        &baseline_stake_address,
                        &config.authorized_staker.pubkey(),
                        baseline_seed,
                    ),
                    Some(&config.authorized_staker.pubkey()),
                )),
                format!(
                    "Creating baseline stake account for validator {} ({})",
                    formatted_node_pubkey, baseline_stake_address
                ),
            ));
            baseline_action = AccountAction::Delegate;
        }

        // Delegation transactions by actions
        if let AccountAction::None = baseline_action {
        } else if let AccountAction::Withdraw = baseline_action {
            delegate_stake_transactions.push((
                Transaction::new_unsigned(Message::new(
                    &[stake_instruction::withdraw(
                        &baseline_stake_address,
                        &config.authorized_staker.pubkey(),
                        &config.source_stake_address,
                        config.baseline_stake_amount,
                        None,
                    )],
                    Some(&config.authorized_staker.pubkey()),
                )),
                format!(
                    "🏖️ `{}` is delinquent. Removed ◎{} baseline stake",
                    formatted_node_pubkey,
                    lamports_to_sol(config.baseline_stake_amount),
                ),
            ));
        } else if let AccountAction::Deactivate = baseline_action {
            delegate_stake_transactions.push((
                Transaction::new_unsigned(Message::new(
                    &[stake_instruction::deactivate_stake(
                        &baseline_stake_address,
                        &config.authorized_staker.pubkey(),
                    )],
                    Some(&config.authorized_staker.pubkey()),
                )),
                format!(
                    "🏖️ `{}` is delinquent. Deactivated ◎{} baseline stake",
                    formatted_node_pubkey,
                    lamports_to_sol(config.baseline_stake_amount),
                ),
            ));
        } else {
            delegate_stake_transactions.push((
                Transaction::new_unsigned(Message::new(
                    &[stake_instruction::delegate_stake(
                        &baseline_stake_address,
                        &config.authorized_staker.pubkey(),
                        &vote_pubkey,
                    )],
                    Some(&config.authorized_staker.pubkey()),
                )),
                format!(
                    "🥩 `{}` is current. Added ◎{} baseline stake",
                    formatted_node_pubkey,
                    lamports_to_sol(config.baseline_stake_amount),
                ),
            ));
        }

        if !is_long_term_unqualified {
            // remove delinquent validator from list
            validator_list.push(node_pubkey.to_string());
        }
    }
    return (
        create_stake_transactions,
        delegate_stake_transactions,
        validator_list,
        source_stake_lamports_required,
    );
}
