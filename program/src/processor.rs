use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::state::Account;

use crate::{
    instruction::{self, SandyInstruction},
    math::calculate_swap_amount_in,
    state::SandwichState,
};

pub struct Processor;

impl Processor {
    pub fn process(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = SandyInstruction::unpack(instruction_data)?;

        match instruction {
            SandyInstruction::Initialize(args) => Self::process_initialize(accounts, args),
            SandyInstruction::SwapIn(args) => Self::process_swap_in(accounts, args),
            SandyInstruction::SwapOut => Self::process_swap_out(accounts),
        }
    }

    fn process_initialize(accounts: &[AccountInfo], args: SandwichState) -> ProgramResult {
        msg!("Instruction: Initialize");

        let accounts_iter = &mut accounts.iter();

        let payer = next_account_info(accounts_iter)?;
        let sandwich_state = next_account_info(accounts_iter)?;
        let system_program = next_account_info(accounts_iter)?;

        verify_payer(payer)?;

        if sandwich_state.data_is_empty() {
            let rent = Rent::get()?;
            let lamports = rent.minimum_balance(SandwichState::LEN);

            let (_, bump) = Pubkey::find_program_address(&[b"sandwich-state"], &crate::id());

            invoke_signed(
                &system_instruction::create_account(
                    payer.key,
                    sandwich_state.key,
                    lamports,
                    SandwichState::LEN as u64,
                    &crate::id(),
                ),
                &[
                    payer.clone(),
                    sandwich_state.clone(),
                    system_program.clone(),
                ],
                &[&[b"sandwich-state", &[bump]]],
            )?;
        }

        args.serialize(&mut *sandwich_state.data.borrow_mut())?;

        Ok(())
    }

    fn process_swap_in(accounts: &[AccountInfo], args: instruction::SwapIn) -> ProgramResult {
        msg!("Instruction: SwapIn");

        let accounts_iter = &mut accounts.iter();

        let payer = next_account_info(accounts_iter)?;
        let sandwich_state = next_account_info(accounts_iter)?;
        let user_source_token_account = next_account_info(accounts_iter)?;
        let user_destination_token_account = next_account_info(accounts_iter)?;

        let raydium_program = next_account_info(accounts_iter)?;
        let token_address = next_account_info(accounts_iter)?;
        let amm_id = next_account_info(accounts_iter)?;
        let amm_authority = next_account_info(accounts_iter)?;
        let pool_coin_token_account = next_account_info(accounts_iter)?;
        let pool_pc_token_account = next_account_info(accounts_iter)?;

        let token_program = next_account_info(accounts_iter)?;
        let associated_token_program = next_account_info(accounts_iter)?;
        let system_program = next_account_info(accounts_iter)?;

        verify_payer(payer)?;

        if user_destination_token_account.data_is_empty() {
            let ix = create_associated_token_account(
                payer.key,
                payer.key,
                token_address.key,
                token_program.key,
            );

            invoke(
                &ix,
                &[
                    payer.clone(),
                    user_destination_token_account.clone(),
                    token_address.clone(),
                    associated_token_program.clone(),
                    token_program.clone(),
                    system_program.clone(),
                ],
            )?;
        }

        let amm_market = RaydiumMarketV4::try_from_slice(&amm_id.data.borrow()[432..464])?;

        let mut reserve_a = Account::unpack(&pool_coin_token_account.data.borrow())?.amount;
        let mut reserve_b = Account::unpack(&pool_pc_token_account.data.borrow())?.amount;
        if amm_market.quote_mint != *token_address.key {
            // swap reserves
            std::mem::swap(&mut reserve_a, &mut reserve_b);
        }

        let wsol_balance = Account::unpack(&user_source_token_account.data.borrow())?.amount;

        let mut sandwich_state_data = SandwichState::try_from_slice(&sandwich_state.data.borrow())?;
        sandwich_state_data.preswap_sol_balance = wsol_balance;
        sandwich_state_data.serialize(&mut *sandwich_state.data.borrow_mut())?;

        let optimal_amount_in = calculate_swap_amount_in(
            0,
            wsol_balance,
            args.user_amount_in,
            args.user_minimum_amount_out,
            reserve_a,
            reserve_b,
        );

        let mut data: Vec<u8> = Vec::new();
        data.push(9);
        data.extend_from_slice(&optimal_amount_in.to_le_bytes());
        data.extend_from_slice(&0u64.to_le_bytes());

        let ix_accounts = vec![
            AccountMeta::new_readonly(*token_program.key, false),
            AccountMeta::new(*amm_id.key, false),
            AccountMeta::new_readonly(*amm_authority.key, false),
            AccountMeta::new(*amm_id.key, false),
            AccountMeta::new(*pool_coin_token_account.key, false),
            AccountMeta::new(*pool_pc_token_account.key, false),
            AccountMeta::new_readonly(*amm_id.key, false),
            AccountMeta::new(*amm_id.key, false),
            AccountMeta::new(*amm_id.key, false),
            AccountMeta::new(*amm_id.key, false),
            AccountMeta::new(*amm_id.key, false),
            AccountMeta::new(*amm_id.key, false),
            AccountMeta::new(*amm_id.key, false),
            AccountMeta::new_readonly(*amm_id.key, false),
            AccountMeta::new(*user_source_token_account.key, false),
            AccountMeta::new(*user_destination_token_account.key, false),
            AccountMeta::new_readonly(*payer.key, true),
        ];

        let instruction = Instruction {
            program_id: *raydium_program.key,
            accounts: ix_accounts,
            data,
        };

        invoke(&instruction, accounts)?;

        Ok(())
    }

    fn process_swap_out(accounts: &[AccountInfo]) -> ProgramResult {
        msg!("Instruction: SwapOut");

        let accounts_iter = &mut accounts.iter();

        let payer = next_account_info(accounts_iter)?;
        let sandwich_state = next_account_info(accounts_iter)?;
        let user_source_token_account = next_account_info(accounts_iter)?;
        let user_destination_token_account = next_account_info(accounts_iter)?;

        let raydium_program = next_account_info(accounts_iter)?;
        let amm_id = next_account_info(accounts_iter)?;
        let amm_authority = next_account_info(accounts_iter)?;
        let pool_coin_token_account = next_account_info(accounts_iter)?;
        let pool_pc_token_account = next_account_info(accounts_iter)?;

        let token_program = next_account_info(accounts_iter)?;
        let system_program = next_account_info(accounts_iter)?;
        let jito_tip_account = next_account_info(accounts_iter)?;

        verify_payer(payer)?;

        let sandwich_state_data = SandwichState::try_from_slice(&sandwich_state.data.borrow())?;

        let token_balance = Account::unpack(&user_source_token_account.data.borrow())?.amount;

        // swap all tokens out
        let mut data: Vec<u8> = Vec::new();
        data.push(9);
        data.extend_from_slice(&token_balance.to_le_bytes());
        data.extend_from_slice(&0u64.to_le_bytes());

        let ix_accounts = vec![
            AccountMeta::new_readonly(*token_program.key, false),
            AccountMeta::new(*amm_id.key, false),
            AccountMeta::new_readonly(*amm_authority.key, false),
            AccountMeta::new(*amm_id.key, false),
            AccountMeta::new(*pool_coin_token_account.key, false),
            AccountMeta::new(*pool_pc_token_account.key, false),
            AccountMeta::new_readonly(*amm_id.key, false),
            AccountMeta::new(*amm_id.key, false),
            AccountMeta::new(*amm_id.key, false),
            AccountMeta::new(*amm_id.key, false),
            AccountMeta::new(*amm_id.key, false),
            AccountMeta::new(*amm_id.key, false),
            AccountMeta::new(*amm_id.key, false),
            AccountMeta::new_readonly(*amm_id.key, false),
            AccountMeta::new(*user_source_token_account.key, false),
            AccountMeta::new(*user_destination_token_account.key, false),
            AccountMeta::new_readonly(*payer.key, true),
        ];

        let instruction = Instruction {
            program_id: *raydium_program.key,
            accounts: ix_accounts,
            data,
        };

        invoke(&instruction, accounts)?;

        let post_swap_balance =
            Account::unpack(&user_destination_token_account.data.borrow())?.amount;

        // check for underflow, if we are losing money fail the transaction
        let profit = post_swap_balance
            .checked_sub(sandwich_state_data.preswap_sol_balance)
            .ok_or(ProgramError::Custom(2))?;

        let tip_bps = u64::from(sandwich_state_data.tip_bps);
        let tip_amount = (profit
            .checked_mul(tip_bps)
            .ok_or(ProgramError::InvalidAccountData)?)
            / 10_000;

        let tip_ix = system_instruction::transfer(payer.key, jito_tip_account.key, tip_amount);

        invoke(
            &tip_ix,
            &[
                payer.clone(),
                jito_tip_account.clone(),
                system_program.clone(),
            ],
        )?;

        Ok(())
    }
}

fn verify_payer(payer: &AccountInfo) -> ProgramResult {
    // replace with your own payer pubkey
    if payer.key != &pubkey!("11111111111111111111111111111111") {
        return Err(ProgramError::Custom(1));
    }
    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

#[derive(BorshDeserialize)]
struct RaydiumMarketV4 {
    pub quote_mint: Pubkey,
}
