use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use crate::state::PoolState;

pub fn handler(
    ctx: Context<InitializePool>, 
) -> Result<()> {

    let pool_state = &mut ctx.accounts.pool_state;
    pool_state.total_amount_minted = 0; 
    pool_state.token_0_mint = ctx.accounts.mint0.key();
    pool_state.token_1_mint = ctx.accounts.mint1.key();
    msg!("Pool state initialized, token0: {}, token1: {}", pool_state.token_0_mint, pool_state.token_1_mint);
    Ok(())
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    // pool for token_x -> token_y 
    pub mint0: Account<'info, Mint>,
    pub mint1: Account<'info, Mint>,

    #[account(
        init, 
        payer=payer, 
        space=8 + PoolState::INIT_SPACE,
        seeds=[b"pool_state", mint0.key().as_ref(), mint1.key().as_ref()], 
        bump,
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    // authority so 1 acc pass in can derive all other pdas 
    /// CHECK: this is the authority for the pool
    #[account(seeds=[b"authority", pool_state.key().as_ref()], bump)]
    pub pool_authority: AccountInfo<'info>,

    // account to hold token X
    #[account(
        init, 
        payer=payer, 
        seeds=[b"vault0", pool_state.key().as_ref()], 
        bump,
        token::mint = mint0,
        token::authority = pool_authority
    )]
    pub vault0: Box<Account<'info, TokenAccount>>, 
    // account to hold token Y
    #[account(
        init, 
        payer=payer, 
        seeds=[b"vault1", pool_state.key().as_ref()],
        bump,
        token::mint = mint1,
        token::authority = pool_authority
    )]
    pub vault1: Box<Account<'info, TokenAccount>>, 

    // pool mint : used to track relative contribution amount of LPs
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

    // accounts required to init a new mint
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

