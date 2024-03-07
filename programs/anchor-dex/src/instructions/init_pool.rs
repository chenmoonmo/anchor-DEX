use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::PoolState;

pub fn handler(
    ctx: Context<InitializePool>, 
) -> Result<()> {

    let pool_state = &mut ctx.accounts.pool_state;
    pool_state.total_amount_minted = 0; 
    msg!("Pool state initialized");
    Ok(())
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    // token0
    pub mint0: Account<'info, Mint>,
    // token1
    pub mint1: Account<'info, Mint>,
    // 池子信息
    #[account(
        init, 
        payer=payer, 
        space=PoolState::init_size(),
        seeds=[b"pool_state", mint0.key().as_ref(), mint1.key().as_ref()], 
        bump,
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    // 持有其他账户权限的账户
    /// CHECK: this is the authority for the pool
    #[account(seeds=[b"authority", pool_state.key().as_ref()], bump)]
    pub pool_authority: UncheckedAccount<'info>,

    // 持有token0的账户
    #[account(
        init, 
        payer=payer, 
        seeds=[b"vault0", pool_state.key().as_ref()], 
        bump,
        token::mint = mint0,
        token::authority = pool_authority
    )]
    pub vault0: Box<Account<'info, TokenAccount>>, 
    // 持有token1的账户
    #[account(
        init, 
        payer=payer, 
        seeds=[b"vault1", pool_state.key().as_ref()],
        bump,
        token::mint = mint1,
        token::authority = pool_authority
    )]
    pub vault1: Box<Account<'info, TokenAccount>>, 

    // LP token
    #[account(
        init, 
        payer=payer,
        seeds=[b"pool_mint", pool_state.key().as_ref()], 
        bump, 
        mint::decimals = 9,
        mint::authority = pool_authority
    )] 
    pub pool_mint: Box<Account<'info, Mint>>, 

    #[account(mut)]
    pub payer: Signer<'info>,

    // 创建 token 需要的参数
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

