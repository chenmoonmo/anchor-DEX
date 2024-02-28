use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("BMP3XsejH17RjqpBTRLN364ZBSWiGBJKQiE73ikUEEBJ");

#[program]
pub mod anchor_dex {
    use super::*;

    pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
        init_pool::handler(ctx)
    }
    pub fn add_liquidity(
        ctx: Context<LiquidityOperation>,
        amount_liq0: u64,
        amount_liq1: u64,
    ) -> Result<()> {
        liquidity::add_liquidity(ctx, amount_liq0, amount_liq1)
    }
    pub fn remove_liquidity(ctx: Context<LiquidityOperation>, burn_amount: u64) -> Result<()> {
        liquidity::remove_liquidity(ctx, burn_amount)
    }
    // pub fn swap()
}
