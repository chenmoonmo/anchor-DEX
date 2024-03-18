use anchor_lang::prelude::*;

// a pool without fee
#[account]
pub struct PoolState {
    pub mint0: Pubkey,
    pub mint1: Pubkey,
    pub total_amount_minted: u64,
}

impl PoolState {
    // total_amount_minted: u64 needs 8 bytes

    pub fn init_size() -> usize {
        let total_amount_minted_size: usize = 8;
        let mint0_size: usize = 32;
        let mint1_size: usize = 32;

        let total_size: usize = (total_amount_minted_size + mint0_size + mint1_size) * 2;

        return total_size;
    }
}
