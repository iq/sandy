use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SandwichState {
    pub preswap_sol_balance: u64,
    pub tip_bps: u16,
}

impl SandwichState {
  pub const LEN : usize = 8 + 2;
}