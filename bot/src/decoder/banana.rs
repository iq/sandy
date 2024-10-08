use borsh::BorshDeserialize;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;

use super::{raydium_amm::RAYDIUM_AMM_PROGRAM_ID, InstructionDecoder, SwapBaseIn, SwapInstruction};

pub const BANANA_PROGRAM_ID: Pubkey = pubkey!("BANANAjs7FJiPQqJTGFzkZJndT9o7UmKiYYGaJz6frGu");

pub struct BananaGun;

impl InstructionDecoder for BananaGun {
    fn decode_instruction(
        &self,
        data: &[u8],
        account_keys: &[Pubkey],
        accounts: &[u8],
    ) -> Option<SwapInstruction> {
        if account_keys[accounts[3] as usize] != RAYDIUM_AMM_PROGRAM_ID {
            return None;
        }

        let data = &data[data.len() - 16..];

        let instruction: SwapBaseIn = match SwapBaseIn::try_from_slice(data) {
            Ok(base_in) => base_in,
            Err(_) => return None,
        };

        Some(SwapInstruction {
            amount_in: instruction.amount_in,
            minimum_amount_out: instruction.minimum_amount_out,
            amm_id: account_keys[accounts[7] as usize],
        })
    }
}
