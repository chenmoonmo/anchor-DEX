use anchor_lang::prelude::*;

// a pool without fee
#[account]
#[derive(InitSpace)]
pub struct PoolState {
    /// Address of token 0 mint
    pub token_0_mint: Pubkey,
    /// Address of token 1 mint
    pub token_1_mint: Pubkey,
    /// total amount of pool mint
    pub total_amount_minted: u64,
}
