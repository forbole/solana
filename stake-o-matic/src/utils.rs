use log::*;
use solana_client::{
    client_error, rpc_client::RpcClient, rpc_config::RpcSimulateTransactionConfig,
    rpc_request::MAX_GET_SIGNATURE_STATUSES_QUERY_ITEMS
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
    iter::FromIterator,
    str::FromStr,
    thread::sleep,
    time::Duration,
};

use crate::Config;

pub fn get_stake_account(
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
pub fn classify_block_producers(
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

pub fn validate_source_stake_account(
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

pub struct ConfirmedTransaction {
    success: bool,
    signature: Signature,
    memo: String,
}

/// Simulate a list of transactions and filter out the ones that will fail
pub fn simulate_transactions(
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

pub fn transact(
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

pub fn process_confirmations(
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