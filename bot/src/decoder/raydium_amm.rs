use borsh::BorshDeserialize;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;

use super::{InstructionDecoder, SwapBaseIn, SwapInstruction};

pub const RAYDIUM_AMM_PROGRAM_ID: Pubkey = pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");

pub struct RaydiumAmm;

impl InstructionDecoder for RaydiumAmm {
    fn decode_instruction(
        &self,
        data: &[u8],
        account_keys: &[Pubkey],
        accounts: &[u8],
    ) -> Option<SwapInstruction> {
        if data.first() != Some(&9) {
            return None;
        }

        let data = &data[1..];
        let instruction: SwapBaseIn = match SwapBaseIn::try_from_slice(data) {
            Ok(base_in) => base_in,
            Err(_) => return None,
        };

        Some(SwapInstruction {
            amount_in: instruction.amount_in,
            minimum_amount_out: instruction.minimum_amount_out,
            amm_id: account_keys[accounts[1] as usize],
        })
    }
}
