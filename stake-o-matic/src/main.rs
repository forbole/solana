use log::*;
use solana_client::{
    rpc_client::RpcClient
};
use solana_notifier::Notifier;
use solana_sdk::{
    native_token::*,
    pubkey::Pubkey,
    signature::{Keypair}
};

use std::{
    collections::{HashMap, HashSet},
    error,
    fs::File,
    path::PathBuf,
    process,
    str::FromStr,
};

mod stake_transaction;
mod validator_list;
mod args;
mod utils;
mod validator_filter;

use stake_transaction::generate_stake_transactions;
use args::get_config;
use utils::*;
use validator_filter::generate_validator_list;

#[derive(Debug)]
pub struct Config {
    json_rpc_url: String,
    cluster: String,
    source_stake_address: Pubkey,
    authorized_staker: Keypair,

    /// Only validators with an identity pubkey in this validator_list will be staked
    validator_list: HashSet<Pubkey>,

    dry_run: bool,

    /// Amount of lamports to stake any validator in the validator_list that is not delinquent
    baseline_stake_amount: u64,

    /// Amount of additional lamports to stake quality block producers in the validator_list
    bonus_stake_amount: u64,

    /// Quality validators produce a block at least this percentage of their leader slots over the
    /// previous epoch
    quality_block_producer_percentage: usize,

    /// A delinquent validator gets this number of slots of grace (from the current slot) before it
    /// will be fully destaked.  The grace period is intended to account for unexpected bugs that
    /// cause a validator to go down
    delinquent_grace_slot_distance: u64,

    /// Don't ever unstake more than this percentage of the cluster at one time
    max_poor_block_productor_percentage: usize,

    address_labels: HashMap<String, String>,

    // new validator list output path
    validator_list_ouput_path: PathBuf,

    // validator list length at least this size
    validator_min_length: usize,

    // the cap of commision for filtering new validators
    commission_cap: u8,

    // the cap of activated stake percentage for filtering new validators
    stake_percentage_cap: f64,
}

#[allow(clippy::cognitive_complexity)] // Yeah I know...
fn main() -> Result<(), Box<dyn error::Error>> {
    solana_logger::setup_with_default("solana=info");
    let config = get_config();
    let notifier = Notifier::default();
    let rpc_client = RpcClient::new(config.json_rpc_url.clone());

    let source_stake_balance = validate_source_stake_account(&rpc_client, &config)?;

    let epoch_info = rpc_client.get_epoch_info()?;
    let last_epoch = epoch_info.epoch - 1;

    info!("Epoch info: {:?}", epoch_info);

    let (quality_block_producers, poor_block_producers) =
        classify_block_producers(&rpc_client, &config, last_epoch)?;
    let too_many_poor_block_producers = false;

    // Fetch vote account status for all the validator_listed validators
    let vote_account_status = rpc_client.get_vote_accounts()?;

    let validator_list = generate_validator_list(&config, &vote_account_status, &quality_block_producers);
    
    let vote_account_info = vote_account_status
        .current
        .into_iter()
        .chain(vote_account_status.delinquent.into_iter())
        .filter_map(|vai| {
            let node_pubkey = Pubkey::from_str(&vai.node_pubkey).ok()?;
            if validator_list.contains(&node_pubkey) {
                Some(vai)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // create stake transactions
    let (
        create_stake_transactions,
        delegate_stake_transactions,
        validator_list,
        source_stake_lamports_required,
    ) = generate_stake_transactions(
        &vote_account_info,
        &config,
        &rpc_client,
        quality_block_producers,
        too_many_poor_block_producers,
        &epoch_info,
    );

    // confirm create stake transactions
    if create_stake_transactions.is_empty() {
        info!("All stake accounts exist");
    } else {
        info!(
            "{} SOL is required to create {} stake accounts",
            lamports_to_sol(source_stake_lamports_required),
            create_stake_transactions.len()
        );
        if source_stake_balance < source_stake_lamports_required {
            error!(
                "Source stake account has insufficient balance: {} SOL, but {} SOL is required",
                lamports_to_sol(source_stake_balance),
                lamports_to_sol(source_stake_lamports_required)
            );
            process::exit(1);
        }

        let create_stake_transactions =
            simulate_transactions(&rpc_client, create_stake_transactions)?;
        let confirmations = transact(
            &rpc_client,
            config.dry_run,
            create_stake_transactions,
            &config.authorized_staker,
        )?;

        if !process_confirmations(confirmations, None) {
            error!("Failed to create one or more stake accounts.  Unable to continue");
            process::exit(1);
        }
    }

    // confirm delegate stake transactions
    let delegate_stake_transactions =
        simulate_transactions(&rpc_client, delegate_stake_transactions)?;
    let confirmations = transact(
        &rpc_client,
        config.dry_run,
        delegate_stake_transactions,
        &config.authorized_staker,
    )?;

    if too_many_poor_block_producers {
        let message = format!(
            "Note: Something is wrong, more than {}% of validators classified \
                       as poor block producers in epoch {}.  Bonus stake frozen",
            config.max_poor_block_productor_percentage, last_epoch,
        );
        warn!("{}", message);
        if !config.dry_run {
            notifier.send(&message);
        }
    }
    if !process_confirmations(
        confirmations,
        if config.dry_run {
            None
        } else {
            Some(&notifier)
        },
    ) {
        process::exit(1);
    }
    if !config.dry_run {
        // update new validator_list
        let buffer = File::create(config.validator_list_ouput_path).unwrap();
        serde_yaml::to_writer(buffer, &validator_list).unwrap();
    }
    Ok(())
}
