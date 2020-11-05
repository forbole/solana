use clap::{crate_description, crate_name, crate_version, value_t, value_t_or_exit, App, Arg};
use solana_clap_utils::{
    input_parsers::{keypair_of, pubkey_of},
    input_validators::{is_amount, is_keypair, is_pubkey_or_keypair, is_url, is_valid_percentage},
};
use log::*;
use solana_sdk::{
    native_token::*,
    pubkey::Pubkey,
};
use std::{
    collections::HashSet,
    fs::File,
    path::PathBuf,
    process,
    str::FromStr
};
use crate::validator_list;
use crate::Config;
pub fn get_config() -> Config {
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
                .default_value("validator.list")
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
    let validator_list_ouput_path = value_t_or_exit!(matches, "validator_list_file", PathBuf);
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
                        info!("Unable to open validator_list: {}, create empty file", err);
                        return File::create(&validator_list_ouput_path).unwrap();
                    });

            let validator_list = serde_yaml::from_reader::<_, Vec<String>>(validator_list_file)
                .unwrap_or_else(|err| {
                    info!("Unable to read validator_list: {}, create empty vector.", err);
                    return vec![];
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