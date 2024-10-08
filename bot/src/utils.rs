use crate::{decoder::raydium_amm::RAYDIUM_AMM_PROGRAM_ID, relayer::Packet};
use anyhow::Result;
use borsh::BorshDeserialize;
use rand::seq::SliceRandom;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    packet::{Packet as SolanaPacket, PACKET_DATA_SIZE},
    pubkey,
    pubkey::Pubkey,
    transaction::VersionedTransaction,
};

use std::{cmp::min, str::FromStr, sync::Arc};

pub fn versioned_tx_from_packet(p: &Packet) -> Option<VersionedTransaction> {
    let mut data = [0; PACKET_DATA_SIZE];
    let copy_len = min(data.len(), p.data.len());
    data[..copy_len].copy_from_slice(&p.data[..copy_len]);
    let mut packet = SolanaPacket::new(data, Default::default());
    if let Some(meta) = &p.meta {
        packet.meta_mut().size = meta.size as usize;
    }
    packet.deserialize_slice(..).ok()
}

pub fn get_random_tip_account() -> Pubkey {
    let tip_accounts = [
        "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
        "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
        "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
        "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
        "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
        "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
        "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
        "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
    ];

    Pubkey::from_str(tip_accounts.choose(&mut rand::thread_rng()).unwrap()).unwrap()
}

pub struct PoolDetails {
    pub token_address: Pubkey,
    pub amm_id: Pubkey,
    pub pool_coin_token_account: Pubkey,
    pub pool_pc_token_account: Pubkey,
}

pub async fn get_pool_details(
    rpc_client: Arc<RpcClient>,
    amm_id: Pubkey,
) -> Result<PoolDetails> {
    let amm_market_data = rpc_client.get_account_data(&amm_id).await?;
    let amm_market_account = RaydiumMarketV4::try_from_slice(&amm_market_data)?;

    let serum_market = amm_market_account.market_id;

    let pool_coin_token_account = Pubkey::find_program_address(
        &[
            &RAYDIUM_AMM_PROGRAM_ID.to_bytes(),
            &serum_market.to_bytes(),
            b"coin_vault_associated_seed",
        ],
        &RAYDIUM_AMM_PROGRAM_ID,
    )
    .0;

    let pool_pc_token_account = Pubkey::find_program_address(
        &[
            &RAYDIUM_AMM_PROGRAM_ID.to_bytes(),
            &serum_market.to_bytes(),
            b"pc_vault_associated_seed",
        ],
        &RAYDIUM_AMM_PROGRAM_ID,
    )
    .0;

    let reversed = amm_market_account.quote_mint.to_string()
        == "So11111111111111111111111111111111111111112";

    let token_address = if reversed {
        amm_market_account.base_mint
    } else {
        amm_market_account.quote_mint
    };

    Ok(PoolDetails {
        token_address,
        amm_id,
        pool_coin_token_account,
        pool_pc_token_account,
    })
}

pub fn to_base_58(transaction: &VersionedTransaction) -> String {
    let serialized = bincode::serialize(&transaction).unwrap();
    bs58::encode(serialized).into_string()
}

pub fn get_associated_token_address(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            owner.as_ref(),
            &pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").to_bytes(),
            mint.as_ref(),
        ],
        &pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"),
    )
    .0
}

#[derive(BorshDeserialize)]
struct RaydiumMarketV4 {
    pub _status: u64,
    pub _nonce: u64,
    pub _max_order: u64,
    pub _depth: u64,
    pub _base_decimal: u64,
    pub _quote_decimal: u64,
    pub _state: u64,
    pub _reset_flag: u64,
    pub _min_size: u64,
    pub _vol_max_cut_ratio: u64,
    pub _amount_wave_ratio: u64,
    pub _base_lot_size: u64,
    pub _quote_lot_size: u64,
    pub _min_price_multiplier: u64,
    pub _max_price_multiplier: u64,
    pub _system_decimal_value: u64,
    pub _min_separate_numerator: u64,
    pub _min_separate_denominator: u64,
    pub _trade_fee_numerator: u64,
    pub _trade_fee_denominator: u64,
    pub _pnl_numerator: u64,
    pub _pnl_denominator: u64,
    pub _swap_fee_numerator: u64,
    pub _swap_fee_denominator: u64,
    pub _base_need_take_pnl: u64,
    pub _quote_need_take_pnl: u64,
    pub _quote_total_pnl: u64,
    pub _base_total_pnl: u64,
    pub _pool_open_time: u64,
    pub _punish_pc_amount: u64,
    pub _punish_coin_amount: u64,
    pub _orderbook_to_init_time: u64,
    pub _swap_base_in_amount: u128,
    pub _swap_quote_out_amount: u128,
    pub _swap_base2_quote_fee: u64,
    pub _swap_quote_in_amount: u128,
    pub _swap_base_out_amount: u128,
    pub _swap_quote2_base_fee: u64,
    // AMM vault
    pub _base_vault: Pubkey,
    pub _quote_vault: Pubkey,
    // Mint
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub _lp_mint: Pubkey,
    // Market
    pub _open_orders: Pubkey,
    pub market_id: Pubkey,
    pub _market_program_id: Pubkey,
    pub _target_orders: Pubkey,
    pub _withdraw_queue: Pubkey,
    pub _lp_vault: Pubkey,
    pub _owner: Pubkey,
    // True circulating supply without lock up
    pub _lp_reserve: u64,
    pub _padding: [u64; 3],
}
