use clap::{crate_description, crate_name, crate_version, value_t, value_t_or_exit, App, Arg};
use log::*;
use solana_clap_utils::{
    input_parsers::{keypair_of, pubkey_of},
    input_validators::{is_amount, is_keypair, is_pubkey_or_keypair, is_url, is_valid_percentage},
};
use solana_client::{
    client_error, rpc_client::RpcClient, rpc_config::RpcSimulateTransactionConfig,
    rpc_request::MAX_GET_SIGNATURE_STATUSES_QUERY_ITEMS, rpc_response::RpcVoteAccountInfo,
    rpc_response::RpcVoteAccountStatus,
};
use solana_notifier::Notifier;
use solana_sdk::{
    account_utils::StateMut,
    clock::{Epoch, Slot},
    commitment_config::CommitmentConfig,
    native_token::*,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};
use solana_stake_program::stake_state::StakeState;

use std::{
    collections::{HashMap, HashSet},
    error,
    fs::File,
    iter::FromIterator,
    path::PathBuf,
    process,
    str::FromStr,
    thread::sleep,
    time::Duration,
};

mod stake_transaction;
mod validator_list;
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

fn get_config() -> Config {
    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .arg({
            let arg = Arg::with_name("config_file")
                .short("C")
                .long("config")
                .value_name("PATH")
                .takes_value(true)
                .global(true)
                .help("Configuration file to use");
            if let Some(ref config_file) = *solana_cli_config::CONFIG_FILE {
                arg.default_value(&config_file)
            } else {
                arg
            }
        })
        .arg(
            Arg::with_name("json_rpc_url")
                .long("url")
                .value_name("URL")
                .takes_value(true)
                .validator(is_url)
                .help("JSON RPC URL for the cluster")
                .conflicts_with("cluster")
        )
        .arg(
            Arg::with_name("cluster")
                .long("cluster")
                .value_name("NAME")
                .possible_values(&["mainnet-beta", "testnet"])
                .takes_value(true)
                .help("Name of the cluster to operate on")
        )
        .arg(
            Arg::with_name("validator_list_file")
                .long("validator-list")
                .value_name("FILE")
                .required(true)
                .takes_value(true)
                .conflicts_with("cluster")
                .help("File containing an YAML array of validator pubkeys eligible for staking")
        )
        .arg(
            Arg::with_name("confirm")
                .long("confirm")
                .takes_value(false)
                .help("Confirm that the stake adjustments should actually be made")
        )
        .arg(
            Arg::with_name("source_stake_address")
                .index(1)
                .value_name("ADDRESS")
                .takes_value(true)
                .required(true)
                .validator(is_pubkey_or_keypair)
                .help("The source stake account for splitting individual validator stake accounts from")
        )
        .arg(
            Arg::with_name("authorized_staker")
                .index(2)
                .value_name("KEYPAIR")
                .validator(is_keypair)
                .required(true)
                .takes_value(true)
        )
        .arg(
            Arg::with_name("quality_block_producer_percentage")
                .long("quality-block-producer-percentage")
                .value_name("PERCENTAGE")
                .takes_value(true)
                .default_value("75")
                .validator(is_valid_percentage)
                .help("Quality validators produce a block in at least this percentage of their leader slots over the previous epoch")
        )
        .arg(
            Arg::with_name("baseline_stake_amount")
                .long("baseline-stake-amount")
                .value_name("SOL")
                .takes_value(true)
                .default_value("5000")
                .validator(is_amount)
        )
        .arg(
            Arg::with_name("bonus_stake_amount")
                .long("bonus-stake-amount")
                .value_name("SOL")
                .takes_value(true)
                .default_value("15")
                .validator(is_amount)
        ).arg(
            Arg::with_name("validator_min_length")
                .long("validator-min-length")
                .value_name("LENGTH")
                .takes_value(true)
                .default_value("20")
                .validator(is_amount)
        ).arg(
            Arg::with_name("commission_cap")
                .long("commission-cap")
                .value_name("COMMISSION")
                .takes_value(true)
                .default_value("10")
                .validator(is_amount)
        ).arg(
            Arg::with_name("stake_percentage_cap")
                .long("stake-percentage-cap")
                .value_name("STAKECAP")
                .takes_value(true)
                .default_value("5")
                .validator(is_amount)
        )
        .get_matches();

    let config = if let Some(config_file) = matches.value_of("config_file") {
        solana_cli_config::Config::load(config_file).unwrap_or_default()
    } else {
        solana_cli_config::Config::default()
    };

    let source_stake_address = pubkey_of(&matches, "source_stake_address").unwrap();
    let authorized_staker = keypair_of(&matches, "authorized_staker").unwrap();
    let dry_run = !matches.is_present("confirm");
    let cluster = value_t!(matches, "cluster", String).unwrap_or_else(|_| "unknown".into());
    let quality_block_producer_percentage =
        value_t_or_exit!(matches, "quality_block_producer_percentage", usize);
    let baseline_stake_amount =
        sol_to_lamports(value_t_or_exit!(matches, "baseline_stake_amount", f64));
    let bonus_stake_amount = sol_to_lamports(value_t_or_exit!(matches, "bonus_stake_amount", f64));
    let mut validator_list_ouput_path = PathBuf::from("validators/list.yaml");
    let (json_rpc_url, validator_list) = match cluster.as_str() {
        "mainnet-beta" => (
            "http://api.mainnet-beta.solana.com".into(),
            validator_list::mainnet_beta_validators(),
        ),
        "testnet" => (
            "http://testnet.solana.com".into(),
            validator_list::testnet_validators(),
        ),
        "unknown" => {
            let validator_list_file =
                File::open(value_t_or_exit!(matches, "validator_list_file", PathBuf))
                    .unwrap_or_else(|err| {
                        error!("Unable to open validator_list: {}", err);
                        process::exit(1);
                    });
            validator_list_ouput_path = value_t_or_exit!(matches, "validator_list_file", PathBuf);

            let validator_list = serde_yaml::from_reader::<_, Vec<String>>(validator_list_file)
                .unwrap_or_else(|err| {
                    error!("Unable to read validator_list: {}", err);
                    process::exit(1);
                })
                .into_iter()
                .map(|p| {
                    Pubkey::from_str(&p).unwrap_or_else(|err| {
                        error!("Invalid validator_list pubkey '{}': {}", p, err);
                        process::exit(1);
                    })
                })
                .collect();
            (
                value_t!(matches, "json_rpc_url", String)
                    .unwrap_or_else(|_| config.json_rpc_url.clone()),
                validator_list,
            )
        }
        _ => unreachable!(),
    };
    let validator_list = validator_list.into_iter().collect::<HashSet<_>>();
    let validator_min_length = value_t_or_exit!(matches, "validator_min_length", usize);
    let commission_cap = value_t_or_exit!(matches, "commission_cap", u8);
    let stake_percentage_cap = value_t_or_exit!(matches, "stake_percentage_cap", f64);
    let config = Config {
        json_rpc_url,
        cluster,
        source_stake_address,
        authorized_staker,
        validator_list,
        dry_run,
        baseline_stake_amount,
        bonus_stake_amount,
        delinquent_grace_slot_distance: 21600, // ~24 hours worth of slots at 2.5 slots per second
        quality_block_producer_percentage,
        max_poor_block_productor_percentage: 100,
        address_labels: config.address_labels,
        validator_list_ouput_path,
        validator_min_length,
        commission_cap,
        stake_percentage_cap,
    };

    info!("RPC URL: {}", config.json_rpc_url);
    config
}

fn get_stake_account(
    rpc_client: &RpcClient,
    address: &Pubkey,
) -> Result<(u64, StakeState), String> {
    let account = rpc_client.get_account(address).map_err(|e| {
        format!(
            "Failed to fetch stake account {}: {}",
            address,
            e.to_string()
        )
    })?;

    if account.owner != solana_stake_program::id() {
        return Err(format!(
            "not a stake account (owned by {}): {}",
            account.owner, address
        ));
    }

    account
        .state()
        .map_err(|e| {
            format!(
                "Failed to decode stake account at {}: {}",
                address,
                e.to_string()
            )
        })
        .map(|stake_state| (account.lamports, stake_state))
}

/// Split validators into quality/poor lists based on their block production over the given `epoch`
fn classify_block_producers(
    rpc_client: &RpcClient,
    config: &Config,
    epoch: Epoch,
) -> Result<(HashSet<Pubkey>, HashSet<Pubkey>), Box<dyn error::Error>> {
    let epoch_schedule = rpc_client.get_epoch_schedule()?;
    let first_slot_in_epoch = epoch_schedule.get_first_slot_in_epoch(epoch);
    let last_slot_in_epoch = epoch_schedule.get_last_slot_in_epoch(epoch);

    let minimum_ledger_slot = rpc_client.minimum_ledger_slot()?;
    if minimum_ledger_slot >= last_slot_in_epoch {
        return Err(format!(
            "Minimum ledger slot is newer than the last epoch: {} > {}",
            minimum_ledger_slot, last_slot_in_epoch
        )
        .into());
    }

    let first_slot = if minimum_ledger_slot > first_slot_in_epoch {
        minimum_ledger_slot
    } else {
        first_slot_in_epoch
    };

    let confirmed_blocks = rpc_client.get_confirmed_blocks(first_slot, Some(last_slot_in_epoch))?;
    let confirmed_blocks: HashSet<Slot> = HashSet::from_iter(confirmed_blocks.into_iter());

    let mut poor_block_producers = HashSet::new();
    let mut quality_block_producers = HashSet::new();

    let leader_schedule = rpc_client.get_leader_schedule(Some(first_slot))?.unwrap();
    for (validator_identity, relative_slots) in leader_schedule {
        let mut validator_blocks = 0;
        let mut validator_slots = 0;
        for relative_slot in relative_slots {
            let slot = first_slot_in_epoch + relative_slot as Slot;
            if slot >= first_slot {
                validator_slots += 1;
                if confirmed_blocks.contains(&slot) {
                    validator_blocks += 1;
                }
            }
        }
        trace!(
            "Validator {} produced {} blocks in {} slots",
            validator_identity,
            validator_blocks,
            validator_slots
        );
        if validator_slots > 0 {
            let validator_identity = Pubkey::from_str(&validator_identity)?;
            if validator_blocks * 100 / validator_slots >= config.quality_block_producer_percentage
            {
                quality_block_producers.insert(validator_identity);
            } else {
                poor_block_producers.insert(validator_identity);
            }
        }
    }

    info!("quality_block_producers: {}", quality_block_producers.len());
    trace!("quality_block_producers: {:?}", quality_block_producers);
    info!("poor_block_producers: {}", poor_block_producers.len());
    trace!("poor_block_producers: {:?}", poor_block_producers);
    Ok((quality_block_producers, poor_block_producers))
}

fn validate_source_stake_account(
    rpc_client: &RpcClient,
    config: &Config,
) -> Result<u64, Box<dyn error::Error>> {
    // check source stake account
    let (source_stake_balance, source_stake_state) =
        get_stake_account(&rpc_client, &config.source_stake_address)?;

    info!(
        "stake account balance: {} SOL",
        lamports_to_sol(source_stake_balance)
    );
    match &source_stake_state {
        StakeState::Initialized(_) | StakeState::Stake(_, _) => source_stake_state
            .authorized()
            .map_or(Ok(source_stake_balance), |authorized| {
                if authorized.staker != config.authorized_staker.pubkey() {
                    Err(format!(
                        "The authorized staker for the source stake account is not {}",
                        config.authorized_staker.pubkey()
                    )
                    .into())
                } else {
                    Ok(source_stake_balance)
                }
            }),
        _ => Err(format!(
            "Source stake account is not in the initialized state: {:?}",
            source_stake_state
        )
        .into()),
    }
}

struct ConfirmedTransaction {
    success: bool,
    signature: Signature,
    memo: String,
}

/// Simulate a list of transactions and filter out the ones that will fail
fn simulate_transactions(
    rpc_client: &RpcClient,
    candidate_transactions: Vec<(Transaction, String)>,
) -> client_error::Result<Vec<(Transaction, String)>> {
    let (blockhash, _fee_calculator) = rpc_client.get_recent_blockhash()?;

    info!(
        "Simulating {} transactions with blockhash {}",
        candidate_transactions.len(),
        blockhash
    );
    let mut simulated_transactions = vec![];
    for (mut transaction, memo) in candidate_transactions {
        transaction.message.recent_blockhash = blockhash;

        let sim_result = rpc_client.simulate_transaction_with_config(
            &transaction,
            RpcSimulateTransactionConfig {
                sig_verify: false,
                ..RpcSimulateTransactionConfig::default()
            },
        )?;
        if sim_result.value.err.is_some() {
            trace!(
                "filtering out transaction due to simulation failure: {:?}: {}",
                sim_result,
                memo
            );
        } else {
            simulated_transactions.push((transaction, memo))
        }
    }
    info!(
        "Successfully simulating {} transactions",
        simulated_transactions.len()
    );
    Ok(simulated_transactions)
}

fn transact(
    rpc_client: &RpcClient,
    dry_run: bool,
    transactions: Vec<(Transaction, String)>,
    authorized_staker: &Keypair,
) -> Result<Vec<ConfirmedTransaction>, Box<dyn error::Error>> {
    let authorized_staker_balance = rpc_client.get_balance(&authorized_staker.pubkey())?;
    info!(
        "Authorized staker balance: {} SOL",
        lamports_to_sol(authorized_staker_balance)
    );

    let (blockhash, fee_calculator, last_valid_slot) = rpc_client
        .get_recent_blockhash_with_commitment(CommitmentConfig::max())?
        .value;
    info!("{} transactions to send", transactions.len());

    let required_fee = transactions.iter().fold(0, |fee, (transaction, _)| {
        fee + fee_calculator.calculate_fee(&transaction.message)
    });
    info!("Required fee: {} SOL", lamports_to_sol(required_fee));
    if required_fee > authorized_staker_balance {
        return Err("Authorized staker has insufficient funds".into());
    }

    let mut pending_transactions = HashMap::new();
    for (mut transaction, memo) in transactions.into_iter() {
        transaction.sign(&[authorized_staker], blockhash);

        pending_transactions.insert(transaction.signatures[0], memo);
        if !dry_run {
            rpc_client.send_transaction(&transaction)?;
        }
    }

    let mut finalized_transactions = vec![];
    loop {
        if pending_transactions.is_empty() {
            break;
        }

        let slot = rpc_client.get_slot_with_commitment(CommitmentConfig::max())?;
        info!(
            "Current slot={}, last_valid_slot={} (slots remaining: {}) ",
            slot,
            last_valid_slot,
            last_valid_slot.saturating_sub(slot)
        );

        if slot > last_valid_slot {
            error!(
                "Blockhash {} expired with {} pending transactions",
                blockhash,
                pending_transactions.len()
            );

            for (signature, memo) in pending_transactions.into_iter() {
                finalized_transactions.push(ConfirmedTransaction {
                    success: false,
                    signature,
                    memo,
                });
            }
            break;
        }

        let pending_signatures = pending_transactions.keys().cloned().collect::<Vec<_>>();
        let mut statuses = vec![];
        for pending_signatures_chunk in
            pending_signatures.chunks(MAX_GET_SIGNATURE_STATUSES_QUERY_ITEMS - 1)
        {
            trace!(
                "checking {} pending_signatures",
                pending_signatures_chunk.len()
            );
            statuses.extend(
                rpc_client
                    .get_signature_statuses(&pending_signatures_chunk)?
                    .value
                    .into_iter(),
            )
        }
        assert_eq!(statuses.len(), pending_signatures.len());

        for (signature, status) in pending_signatures.into_iter().zip(statuses.into_iter()) {
            info!("{}: status={:?}", signature, status);
            let completed = if dry_run {
                Some(true)
            } else if let Some(status) = &status {
                if status.confirmations.is_none() || status.err.is_some() {
                    Some(status.err.is_none())
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(success) = completed {
                warn!("{}: completed.  success={}", signature, success);
                let memo = pending_transactions.remove(&signature).unwrap();
                finalized_transactions.push(ConfirmedTransaction {
                    success,
                    signature,
                    memo,
                });
            }
        }
        sleep(Duration::from_secs(5));
    }

    Ok(finalized_transactions)
}

fn process_confirmations(
    mut confirmations: Vec<ConfirmedTransaction>,
    notifier: Option<&Notifier>,
) -> bool {
    let mut ok = true;

    confirmations.sort_by(|a, b| a.memo.cmp(&b.memo));
    for ConfirmedTransaction {
        success,
        signature,
        memo,
    } in confirmations
    {
        if success {
            info!("OK:   {}: {}", signature, memo);
            if let Some(notifier) = notifier {
                notifier.send(&memo)
            }
        } else {
            error!("FAIL: {}: {}", signature, memo);
            ok = false
        }
    }
    ok
}

// for filter validator by stake percentage, quality
fn filter_validator_status(
    config: &Config,
    vote: RpcVoteAccountInfo,
    quality_block_producers: &HashSet<Pubkey>,
    total_activated_stake: u64
) -> Option<RpcVoteAccountInfo> {
    let node_pubkey = Pubkey::from_str(&vote.node_pubkey).ok()?;
    let is_quality_producers = quality_block_producers.contains(&node_pubkey);
    let activated_stake_percentage: f64 = 100.0 * vote.activated_stake as f64 / total_activated_stake as f64;
    let is_percentage_in_range = activated_stake_percentage <= config.stake_percentage_cap;
    let is_over_min_stake_required = vote.activated_stake > 500;
    let is_fit_all_conditions = is_quality_producers && is_percentage_in_range && is_over_min_stake_required;
    if is_fit_all_conditions {
        Some(vote)
    } else {
        None
    }
}

// generate validator hashset for create transactions step
fn generate_validator_list(
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
            filter_validator_status(config, vote, quality_block_producers, total_activated_stake)
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
    ) = stake_transaction::create_stake_transactions(
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
