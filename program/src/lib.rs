use processor::Processor;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

mod instruction;
mod math;
mod processor;
mod state;

entrypoint!(process_instruction);
// replace with your program id
solana_program::declare_id!("11111111111111111111111111111111");

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Processor::process(program_id, accounts, instruction_data)
}
