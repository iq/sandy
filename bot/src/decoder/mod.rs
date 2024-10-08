use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;

pub mod banana;
pub mod raydium_amm;

#[derive(Debug)]
pub struct SwapInstruction {
    pub amount_in: u64,
    pub minimum_amount_out: u64,

    pub amm_id: Pubkey,
}

#[derive(BorshDeserialize)]
struct SwapBaseIn {
    pub amount_in: u64,
    pub minimum_amount_out: u64,
}

pub trait InstructionDecoder {
    fn decode_instruction(
        &self,
        data: &[u8],
        account_keys: &[Pubkey],
        accounts: &[u8],
    ) -> Option<SwapInstruction>;
}

pub fn get_instruction_decoder(program_id: &Pubkey) -> Option<Box<dyn InstructionDecoder + Send>> {
    match *program_id {
        raydium_amm::RAYDIUM_AMM_PROGRAM_ID => Some(Box::new(raydium_amm::RaydiumAmm)),
        banana::BANANA_PROGRAM_ID => Some(Box::new(banana::BananaGun)),
        _ => None,
    }
}
