use anchor_lang::prelude::*;
use anchor_spl::{
    token,
    token::{ Mint, MintTo, Token, TokenAccount, Transfer},
};

use crate::error::ErrorCode;
use crate::state::PoolState;

fn sqrt(x: u128) -> f64 {
    let mut z = (x + 1) / 2;
    let mut y = x;
    while z < y {
        y = z;
        z = (x / z + z) / 2;
    }
    y as f64
}

pub fn add_liquidity(
    ctx: Context<LiquidityOperation>,
    amount_liq0: u64, // amount of token0
    amount_liq1: u64, // amount of token1
) -> Result<()> {
    let user_balance0 = ctx.accounts.user0.amount;
    let user_balance1 = ctx.accounts.user1.amount;

    // ensure enough balance
    require!(amount_liq0 <= user_balance0, ErrorCode::NotEnoughBalance);
    require!(amount_liq1 <= user_balance1, ErrorCode::NotEnoughBalance);

    let vault_balance0 = ctx.accounts.vault0.amount;
    let vault_balance1 = ctx.accounts.vault1.amount;
    let pool_state = &mut ctx.accounts.pool_state;

    let deposit0 = amount_liq0;
    let deposit1;
    let amount_to_mint;

    // initial deposit
    msg!("vaults: {} {}", vault_balance0, vault_balance1);
    msg!("init deposits: {} {}", amount_liq0, amount_liq1);

    if pool_state.total_amount_minted == 0 {
        deposit1 = amount_liq1;
        // lp = sqrt(deposit0 * deposit1)
        amount_to_mint =sqrt((deposit0 as u128).checked_mul(deposit1 as u128).unwrap()).floor() as u64;

        msg!("pmint: {}", amount_to_mint);
    } else {
        // require equal amount deposit based on pool exchange rate
        let exchange10 = vault_balance1.checked_div(vault_balance0).unwrap();
        let amount_deposit_1 = amount_liq0.checked_mul(exchange10).unwrap();

        require!(amount_deposit_1 <= amount_liq1, ErrorCode::NotEnoughBalance);
        deposit1 = amount_deposit_1;

        amount_to_mint = deposit1
            .checked_div(vault_balance1)
            .unwrap()
            .checked_mul(pool_state.total_amount_minted)
            .unwrap();

        msg!("pmint: {}", amount_to_mint);
    }

    require!(amount_to_mint > 0, ErrorCode::NoPoolMintOutput);

    // give pool_mints
    pool_state.total_amount_minted += amount_to_mint;
    
    let mint_ctx: CpiContext<'_, '_, '_, '_, MintTo<'_>> = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            to: ctx.accounts.user_pool_ata.to_account_info(),
            mint: ctx.accounts.pool_mint.to_account_info(),
            authority: ctx.accounts.pool_authority.to_account_info(),
        },
    );
    
    let bump = ctx.bumps.pool_authority;
    let pool_key = ctx.accounts.pool_state.key();
    let pda_sign = &[b"authority", pool_key.as_ref(), &[bump]];

    token::mint_to(mint_ctx.with_signer(
        &[pda_sign],
    ), amount_to_mint)?;

    // transfer tokens
    token::transfer(CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user0.to_account_info(), 
            to: ctx.accounts.vault0.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(), 
        }
    ), deposit0)?;

    token::transfer(CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user1.to_account_info(), 
            to: ctx.accounts.vault1.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(), 
        }
    ), deposit1)?;


    Ok(())
}

pub fn remove_liquidity(ctx: Context<LiquidityOperation>,burn_amount: u64) -> Result<()> {
    // 燃烧用户的LP 将对应份额的资产转给用户
    // 用户持有的LP数量
    let pool_mint_balance = ctx.accounts.user_pool_ata.amount;
    require!(burn_amount <= pool_mint_balance, ErrorCode::NotEnoughBalance);

    let pool_state = &mut ctx.accounts.pool_state;

    require!(pool_state.total_amount_minted >= burn_amount, ErrorCode::BurnTooMuch);

    Ok(())
}


#[derive(Accounts)]
pub struct LiquidityOperation<'info> {
    #[account(mut)]
    pub pool_state: Box<Account<'info, PoolState>>,
    /// CHECK: this is the authority for the pool
    #[account(
        mut,
        seeds=[b"authority", pool_state.key().as_ref()],
        bump
    )]
    pub pool_authority: AccountInfo<'info>,

    #[account(
        mut,
        constraint = vault1.mint == user1.mint,
        seeds=[b"vault0", pool_state.key().as_ref()],
        bump,
    )]
    pub vault0: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = vault1.mint == user1.mint,
        seeds=[b"vault1", pool_state.key().as_ref()],
        bump,
    )]
    pub vault1: Box<Account<'info, TokenAccount>>,
    #[account(
        mut, 
        constraint = user_pool_ata.mint == pool_mint.key(),
        seeds=[b"pool_mint", pool_state.key().as_ref()],
        bump
    )]
    pub pool_mint: Box<Account<'info, Mint>>,  

    // user token accounts
    #[account(
        mut,
        has_one = owner,
    )]
    pub user0: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        has_one = owner,
    )]
    pub user1: Box<Account<'info, TokenAccount>>,
    #[account(mut, has_one = owner)]
    pub user_pool_ata: Box<Account<'info, TokenAccount>>,

    pub owner: Signer<'info>,

    pub token_program: Program<'info, Token>,
}
