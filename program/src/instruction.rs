use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

use crate::state::SandwichState;

#[derive(BorshDeserialize)]
pub struct SwapIn {
    pub user_amount_in: u64,
    pub user_minimum_amount_out: u64,
}

pub enum SandyInstruction {
    Initialize(SandwichState),
    SwapIn(SwapIn),
    SwapOut,
}

impl SandyInstruction {
    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = data
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match tag {
            0 => Self::Initialize(SandwichState::try_from_slice(rest)?),
            1 => Self::SwapIn(SwapIn::try_from_slice(rest)?),
            2 => Self::SwapOut,
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}
