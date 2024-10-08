use std::sync::Arc;

use decoder::get_instruction_decoder;
use log::{error, info, warn};
use relayer::{forward_incoming_transactions, PendingTransaction};
use reqwest::Client;
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey, pubkey::Pubkey, signature::Keypair, signer::EncodableKey};
use tokio::sync::mpsc::channel;
use transaction::TransactionBuilder;
use utils::{get_pool_details, to_base_58, versioned_tx_from_packet};

mod decoder;
mod relayer;
mod transaction;
mod utils;

// replace with your deployed program id
pub const SANDWICH_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_nanos()
        .init();

    //clear screen
    print!("\x1B[2J\x1B[1;1H");

    println!();

    let rpc_client = Arc::new(RpcClient::new(
        "https://api.mainnet-beta.solana.com".to_string(),
    ));

    let keypair = Arc::new(Keypair::read_from_file("../payer.json").unwrap());

    let (pending_transactions_tx, mut pending_transactions_rx) = channel::<PendingTransaction>(100);

    tokio::spawn(forward_incoming_transactions(pending_transactions_tx));

    while let Some(pending_transactions) = pending_transactions_rx.recv().await {
        for pending_transaction in pending_transactions.transactions {
            let rpc_client = rpc_client.clone();
            let keypair = keypair.clone();

            tokio::spawn(async move {
                let pending_transaction = match versioned_tx_from_packet(&pending_transaction) {
                    Some(pending_transaction) => pending_transaction,
                    None => {
                        error!("Failed to deserialize transaction");
                        return;
                    }
                };

                info!(
                    "Received pending transaction {:?}",
                    pending_transaction.signatures
                );

                let account_keys = pending_transaction.message.static_account_keys();

                for instruction in pending_transaction.message.instructions() {
                    let program_id = account_keys[instruction.program_id_index as usize];

                    let Some(decoder) = get_instruction_decoder(&program_id) else {
                        warn!("No decoder found for program id: {:?}", program_id);
                        continue;
                    };

                    let Some(user_swap_instruction) = decoder.decode_instruction(
                        &instruction.data,
                        pending_transaction.message.static_account_keys(),
                        &instruction.accounts,
                    ) else {
                        warn!("No swap instruction found in transaction");
                        continue;
                    };

                    info!("Found swap instruction: {:?}", user_swap_instruction);

                    let pool_details =
                        match get_pool_details(rpc_client.clone(), user_swap_instruction.amm_id)
                            .await
                        {
                            Ok(pool_details) => pool_details,
                            Err(e) => {
                                error!("Failed to get pool details: {:?}", e);
                                continue;
                            }
                        };

                    info!(
                        "Building bundle for token: {:?}",
                        pool_details.token_address
                    );

                    let latest_blockhash = *pending_transaction.message.recent_blockhash();

                    let transaction_builder = TransactionBuilder::new(
                        user_swap_instruction.amount_in,
                        user_swap_instruction.minimum_amount_out,
                        keypair.clone(),
                        pool_details,
                        latest_blockhash,
                    );

                    let front_transaction = transaction_builder.front_transaction();
                    let back_transaction = transaction_builder.back_transaction();

                    let bundle = [front_transaction, pending_transaction, back_transaction]
                        .iter()
                        .map(to_base_58)
                        .collect::<Vec<String>>();

                    let res = Client::new()
                        .post("https://ny.mainnet.block-engine.jito.wtf/api/v1/bundles")
                        .json(&json!({"jsonrpc": "2.0", "id": 0, "method": "sendBundle", "params": [bundle]}))
                        .send()
                        .await;

                    match res {
                        Ok(res) => {
                            let status = res.status();
                            if status.is_success() {
                                let body = res.json::<serde_json::Value>().await.unwrap();
                                info!("Sent bundle with ID: {:?}", body.get("result"));
                            } else {
                                error!(
                                    "Failed to send bundle: {:?}",
                                    res.json::<serde_json::Value>().await
                                );
                            }
                        }
                        Err(e) => {
                            error!("Error when sending bundle: {:?}", e);
                        }
                    }

                    break;
                }
            });
        }
    }
}
