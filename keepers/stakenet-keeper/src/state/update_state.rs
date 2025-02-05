use std::{collections::HashMap, error::Error, str::FromStr, sync::Arc};

use anchor_lang::AccountDeserialize;
use jito_tip_distribution::sdk::derive_tip_distribution_account_address;

use solana_client::{nonblocking::rpc_client::RpcClient, rpc_response::RpcVoteAccountInfo};
use solana_sdk::{
    account::Account, instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};

use stakenet_sdk::utils::{
    accounts::{
        get_all_steward_accounts, get_all_steward_validator_accounts, get_all_validator_accounts,
        get_all_validator_history_accounts, get_cluster_history_address,
        get_validator_history_address,
    },
    helpers::get_balance_with_retry,
    instructions::get_create_validator_history_instructions,
    transactions::{
        get_multiple_accounts_batched, get_vote_accounts_with_retry, submit_transactions,
    },
};
use validator_history::{constants::MIN_VOTE_EPOCHS, ClusterHistory, ValidatorHistory};

use crate::operations::keeper_operations::{KeeperCreates, KeeperOperations};

use super::{keeper_config::KeeperConfig, keeper_state::KeeperState};

pub async fn pre_create_update(
    keeper_config: &KeeperConfig,
    keeper_state: &mut KeeperState,
) -> Result<(), Box<dyn Error>> {
    let client = &keeper_config.client;
    let program_id = &keeper_config.validator_history_program_id;
    let keypair = &keeper_config.keypair;

    // Update Epoch
    match client.get_epoch_info().await {
        Ok(latest_epoch) => {
            if latest_epoch.epoch != keeper_state.epoch_info.epoch {
                keeper_state.runs_for_epoch = [0; KeeperOperations::LEN];
                keeper_state.errors_for_epoch = [0; KeeperOperations::LEN];
                keeper_state.txs_for_epoch = [0; KeeperOperations::LEN];

                keeper_state.created_accounts_for_epoch = [0; KeeperCreates::LEN];
            }

            // Always update the epoch info
            keeper_state.epoch_info = latest_epoch.clone();
        }
        Err(e) => {
            return Err(Box::new(e));
        }
    }

    // Fetch Vote Accounts
    keeper_state.vote_account_map = get_vote_account_map(client).await?;

    // Get all get vote accounts
    keeper_state.all_get_vote_account_map =
        get_all_get_vote_account_map(client, keeper_state).await?;

    // Update Cluster History
    keeper_state.cluster_history = get_cluster_history(client, program_id).await?;

    // Update Keeper Balance
    keeper_state.keeper_balance = get_balance_with_retry(client, keypair.pubkey()).await?;
    println!("keeper_balance: {:?}", keeper_state.keeper_balance);

    Ok(())
}

// Should be called after `pre_create_update`
pub async fn create_missing_accounts(
    keeper_config: &KeeperConfig,
    keeper_state: &KeeperState,
) -> Result<Vec<(KeeperCreates, usize)>, Box<dyn Error>> {
    let client = &keeper_config.client;
    let program_id = &keeper_config.validator_history_program_id;
    let keypair = &keeper_config.keypair;

    let mut created_accounts_for_epoch = vec![];

    // Create Missing Accounts
    let new_validator_history_accounts = create_missing_validator_history_accounts(
        client,
        keypair,
        program_id,
        keeper_state,
        keeper_config.tx_retry_count,
        keeper_config.tx_confirmation_seconds,
    )
    .await?;
    created_accounts_for_epoch.push((
        KeeperCreates::CreateValidatorHistory,
        new_validator_history_accounts,
    ));

    Ok(created_accounts_for_epoch)
}

pub async fn post_create_update(
    keeper_config: &KeeperConfig,
    keeper_state: &mut KeeperState,
) -> Result<(), Box<dyn Error>> {
    let client = &keeper_config.client;
    let validator_history_program_id = &keeper_config.validator_history_program_id;
    let tip_distribution_program_id = &keeper_config.tip_distribution_program_id;
    println!(
        "validator_history_program_id: {}",
        validator_history_program_id.to_string()
    );
    println!(
        "tip_distribution_program_id: {}",
        tip_distribution_program_id.to_string()
    );

    // Update Validator History Accounts
    keeper_state.validator_history_map =
        get_validator_history_map(client, validator_history_program_id).await?;

    // Get all history vote accounts
    keeper_state.all_history_vote_account_map =
        get_all_history_vote_account_map(client, keeper_state).await?;

    // Update previous tip distribution map
    keeper_state.previous_epoch_tip_distribution_map = get_tip_distribution_accounts(
        client,
        tip_distribution_program_id,
        keeper_state,
        keeper_state.epoch_info.epoch.saturating_sub(1),
    )
    .await?;

    // Update current tip distribution map
    keeper_state.current_epoch_tip_distribution_map = get_tip_distribution_accounts(
        client,
        tip_distribution_program_id,
        keeper_state,
        keeper_state.epoch_info.epoch,
    )
    .await?;

    keeper_state.all_steward_accounts = Some(
        get_all_steward_accounts(
            &keeper_config.client,
            &keeper_config.steward_program_id,
            &keeper_config.steward_config,
        )
        .await?,
    );

    keeper_state.all_steward_validator_accounts = Some(
        get_all_steward_validator_accounts(
            &keeper_config.client,
            keeper_state.all_steward_accounts.as_ref().unwrap(),
            validator_history_program_id,
        )
        .await?,
    );

    let all_get_vote_accounts: Vec<RpcVoteAccountInfo> =
        keeper_state.vote_account_map.values().cloned().collect();

    keeper_state.all_active_validator_accounts = Some(
        get_all_validator_accounts(
            &keeper_config.client,
            &all_get_vote_accounts,
            validator_history_program_id,
        )
        .await?,
    );

    Ok(())
}

async fn get_vote_account_map(
    client: &Arc<RpcClient>,
) -> Result<HashMap<Pubkey, RpcVoteAccountInfo>, Box<dyn Error>> {
    let active_vote_accounts = HashMap::from_iter(
        get_vote_accounts_with_retry(client, MIN_VOTE_EPOCHS, None)
            .await?
            .iter()
            .map(|vote_account_info| {
                (
                    Pubkey::from_str(vote_account_info.vote_pubkey.as_str())
                        .expect("Could not parse vote pubkey"),
                    vote_account_info.clone(),
                )
            }),
    );

    Ok(active_vote_accounts)
}

async fn get_cluster_history(
    client: &Arc<RpcClient>,
    program_id: &Pubkey,
) -> Result<ClusterHistory, Box<dyn Error>> {
    let cluster_history_address = get_cluster_history_address(program_id);
    println!("{:?}", program_id.to_string());
    println!("{:?}", cluster_history_address.to_string());
    let cluster_history_account = client.get_account(&cluster_history_address).await?;
    let cluster_history =
        ClusterHistory::try_deserialize(&mut cluster_history_account.data.as_slice())?;

    Ok(cluster_history)
}

async fn get_validator_history_map(
    client: &Arc<RpcClient>,
    program_id: &Pubkey,
) -> Result<HashMap<Pubkey, ValidatorHistory>, Box<dyn Error>> {
    let validator_histories = get_all_validator_history_accounts(client, *program_id).await?;

    let validator_history_map = HashMap::from_iter(
        validator_histories
            .iter()
            .map(|vote_history| (vote_history.vote_account, *vote_history)),
    );

    Ok(validator_history_map)
}

async fn get_all_history_vote_account_map(
    client: &Arc<RpcClient>,
    keeper_state: &KeeperState,
) -> Result<HashMap<Pubkey, Option<Account>>, Box<dyn Error>> {
    let validator_history_map = &keeper_state.validator_history_map;

    let all_history_vote_account_pubkeys: Vec<Pubkey> =
        validator_history_map.keys().cloned().collect();

    let all_history_vote_accounts =
        get_multiple_accounts_batched(all_history_vote_account_pubkeys.as_slice(), client).await?;

    let history_vote_accounts_map = all_history_vote_account_pubkeys
        .into_iter()
        .zip(all_history_vote_accounts)
        .collect::<HashMap<Pubkey, Option<Account>>>();

    Ok(history_vote_accounts_map)
}

async fn get_all_get_vote_account_map(
    client: &Arc<RpcClient>,
    keeper_state: &KeeperState,
) -> Result<HashMap<Pubkey, Option<Account>>, Box<dyn Error>> {
    let vote_account_map = &keeper_state.vote_account_map;

    // Convert the keys to a vector of Pubkey values
    let all_get_vote_account_pubkeys: Vec<Pubkey> = vote_account_map.keys().cloned().collect();

    let all_get_vote_accounts =
        get_multiple_accounts_batched(all_get_vote_account_pubkeys.as_slice(), client).await?;

    let get_vote_accounts_map = all_get_vote_account_pubkeys
        .into_iter()
        .zip(all_get_vote_accounts)
        .collect::<HashMap<Pubkey, Option<Account>>>();

    Ok(get_vote_accounts_map)
}

async fn get_tip_distribution_accounts(
    client: &Arc<RpcClient>,
    tip_distribution_program_id: &Pubkey,
    keeper_state: &KeeperState,
    epoch: u64,
) -> Result<HashMap<Pubkey, Option<Account>>, Box<dyn Error>> {
    let vote_accounts = keeper_state
        .all_history_vote_account_map
        .keys()
        .collect::<Vec<_>>();

    /* Filters tip distribution tuples to the addresses, then fetches accounts to see which ones exist */
    let tip_distribution_addresses = vote_accounts
        .iter()
        .map(|vote_pubkey| {
            let (pubkey, _) = derive_tip_distribution_account_address(
                tip_distribution_program_id,
                vote_pubkey,
                epoch,
            );
            pubkey
        })
        .collect::<Vec<Pubkey>>();

    let tip_distribution_accounts =
        get_multiple_accounts_batched(&tip_distribution_addresses, client).await?;

    let result = vote_accounts
        .into_iter()
        .zip(tip_distribution_accounts)
        .map(|(vote_pubkey, account)| (*vote_pubkey, account)) // Dereference vote_pubkey here
        .collect::<HashMap<Pubkey, Option<Account>>>();

    Ok(result)
}

async fn create_missing_validator_history_accounts(
    client: &Arc<RpcClient>,
    keypair: &Arc<Keypair>,
    program_id: &Pubkey,
    keeper_state: &KeeperState,
    retry_count: u16,
    confirmation_time: u64,
) -> Result<usize, Box<dyn Error>> {
    let vote_accounts = &keeper_state
        .vote_account_map
        .keys()
        .collect::<Vec<&Pubkey>>();

    let all_history_addresses = &vote_accounts
        .iter()
        .map(|vote_pubkey| get_validator_history_address(vote_pubkey, program_id))
        .collect::<Vec<Pubkey>>();

    let history_accounts = get_multiple_accounts_batched(all_history_addresses, client).await?;

    assert!(vote_accounts.len() == history_accounts.len());

    let create_transactions = vote_accounts
        .iter()
        .zip(history_accounts)
        .filter_map(|(vote_pubkey, history_account)| {
            match history_account {
                Some(_) => None,
                None => {
                    // Create accounts that don't exist
                    let ix =
                        get_create_validator_history_instructions(vote_pubkey, program_id, keypair);
                    Some(ix)
                }
            }
        })
        .collect::<Vec<Vec<Instruction>>>();

    let accounts_created = create_transactions.len();

    submit_transactions(
        client,
        create_transactions,
        keypair,
        retry_count,
        confirmation_time,
    )
    .await?;

    Ok(accounts_created)
}
