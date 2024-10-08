use solana_sdk::hash::Hash;
use solana_sdk::pubkey;
use solana_sdk::transaction::Transaction;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::VersionedTransaction,
};
use std::sync::Arc;

use crate::utils::get_random_tip_account;
use crate::{
    utils::{get_associated_token_address, PoolDetails},
    SANDWICH_PROGRAM_ID,
};

pub struct TransactionBuilder {
    pub user_amount_in: u64,
    pub user_minimum_amount_out: u64,

    pub keypair: Arc<Keypair>,
    pub pool_details: PoolDetails,

    pub user_source_token_account: Pubkey,
    pub user_destination_token_account: Pubkey,

    pub sandwich_state: Pubkey,
    pub recent_blockhash: Hash,
}

impl TransactionBuilder {
    pub fn new(
        user_amount_in: u64,
        user_minimum_amount_out: u64,
        keypair: Arc<Keypair>,
        pool_details: PoolDetails,
        recent_blockhash: Hash,
    ) -> Self {
        let user_source_token_account = get_associated_token_address(
            &keypair.pubkey(),
            &pubkey!("So11111111111111111111111111111111111111112"),
        );
        let user_destination_token_account =
            get_associated_token_address(&keypair.pubkey(), &pool_details.token_address);

        let sandwich_state =
            Pubkey::find_program_address(&[b"sandwich-state"], &SANDWICH_PROGRAM_ID).0;

        Self {
            user_amount_in,
            user_minimum_amount_out,

            keypair,
            pool_details,

            user_source_token_account,
            user_destination_token_account,

            sandwich_state,
            recent_blockhash,
        }
    }

    pub fn front_transaction(&self) -> VersionedTransaction {
        let swap_accounts = vec![
            AccountMeta::new(self.keypair.pubkey(), true),
            AccountMeta::new(self.sandwich_state, false),
            AccountMeta::new(self.user_source_token_account, false),
            AccountMeta::new(self.user_destination_token_account, false),
            AccountMeta::new_readonly(
                pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"),
                false,
            ),
            AccountMeta::new_readonly(self.pool_details.token_address, false),
            AccountMeta::new(self.pool_details.amm_id, false),
            AccountMeta::new_readonly(
                pubkey!("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"),
                false,
            ),
            AccountMeta::new(self.pool_details.pool_coin_token_account, false),
            AccountMeta::new(self.pool_details.pool_pc_token_account, false),
            AccountMeta::new_readonly(
                pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
                false,
            ),
            AccountMeta::new_readonly(
                pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"),
                false,
            ),
            AccountMeta::new_readonly(pubkey!("11111111111111111111111111111111"), false),
        ];

        let mut data: Vec<u8> = Vec::new();
        data.push(1);
        data.extend_from_slice(&self.user_amount_in.to_le_bytes());
        data.extend_from_slice(&self.user_minimum_amount_out.to_le_bytes());

        self.build_transaction(Instruction {
            program_id: SANDWICH_PROGRAM_ID,
            accounts: swap_accounts,
            data,
        })
    }

    pub fn back_transaction(&self) -> VersionedTransaction {
        let swap_accounts = vec![
            AccountMeta::new(self.keypair.pubkey(), true),
            AccountMeta::new(self.sandwich_state, false),
            AccountMeta::new(self.user_destination_token_account, false),
            AccountMeta::new(self.user_source_token_account, false),
            AccountMeta::new_readonly(
                pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"),
                false,
            ),
            AccountMeta::new(self.pool_details.amm_id, false),
            AccountMeta::new_readonly(
                pubkey!("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"),
                false,
            ),
            AccountMeta::new(self.pool_details.pool_coin_token_account, false),
            AccountMeta::new(self.pool_details.pool_pc_token_account, false),
            AccountMeta::new_readonly(
                pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
                false,
            ),
            AccountMeta::new_readonly(pubkey!("11111111111111111111111111111111"), false),
            AccountMeta::new(get_random_tip_account(), false),
        ];

        let data: Vec<u8> = vec![2];

        self.build_transaction(Instruction {
            program_id: SANDWICH_PROGRAM_ID,
            accounts: swap_accounts,
            data,
        })
    }

    fn build_transaction(&self, swap_instruction: Instruction) -> VersionedTransaction {
        VersionedTransaction::from(Transaction::new_signed_with_payer(
            &[swap_instruction],
            Some(&self.keypair.pubkey()),
            &[&self.keypair],
            self.recent_blockhash,
        ))
    }
}
