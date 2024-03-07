use anchor_lang::prelude::*;
use anchor_spl::{
    token,
    token::{Token, TokenAccount, Transfer},
};

use crate::error::ErrorCode;
use crate::state::PoolState;

pub fn swap(ctx: Context<Swap>, amount_in: u64, min_amount_out: u64) -> Result<()> {
    // 检查用户余额
    let user_token_in_balance = ctx.accounts.user_in.amount;
    require!(
        amount_in <= user_token_in_balance,
        ErrorCode::NotEnoughBalance
    );

    let vault_token_in_balance = ctx.accounts.vault_in.amount as u128;
    // 检查池子余额
    let vault_token_out_balance = ctx.accounts.vault_out.amount as u128;
    require!(
        vault_token_out_balance >= min_amount_out.into(),
        ErrorCode::NotEnoughBalance
    );

    // 计算输出数量
    let k = vault_token_in_balance * vault_token_out_balance;
    let token_out_amount = vault_token_out_balance
        .checked_sub(
            k.checked_div(
                vault_token_in_balance
                    .checked_add(amount_in as u128)
                    .unwrap(),
            )
            .unwrap(),
        )
        .unwrap() as u64;

    // TODO: token_out_amount > min_amount_out
    require!(token_out_amount >= min_amount_out, ErrorCode::NotEnoughOut);

    // 转账
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_in.to_account_info(),
                to: ctx.accounts.vault_in.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        ),
        amount_in,
    )?;

    let bump = ctx.bumps.pool_authority;
    let pool_key = ctx.accounts.pool_state.key();
    let pda_sign = &[b"authority", pool_key.as_ref(), &[bump]];

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_out.to_account_info(),
                to: ctx.accounts.user_out.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
        )
        .with_signer(&[pda_sign]),
        token_out_amount,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct Swap<'info> {
    pub pool_state: Box<Account<'info, PoolState>>,
    /// CHECK: this is the authority for the pool
    #[account(
        seeds=[b"authority", pool_state.key().as_ref()],
        bump
    )]
    pub pool_authority: UncheckedAccount<'info>,
    // 用户 tokenIn 账户
    #[account(
        mut,
        has_one = owner
    )]
    pub user_in: Box<Account<'info, TokenAccount>>,
    // 用户 tokenOut 账户
    #[account(
        mut,
        has_one = owner
    )]
    pub user_out: Box<Account<'info, TokenAccount>>,
    // 池子 tokenIn 账户
    #[account(
        mut,
        constraint = vault_in.mint == user_in.mint,
        constraint = vault_in.owner == pool_authority.key()
    )]
    pub vault_in: Box<Account<'info, TokenAccount>>,
    // 池子 tokenOut 账户
    #[account(
        mut,
        constraint = vault_out.mint == user_out.mint,
        constraint = vault_out.owner == pool_authority.key()
    )]
    pub vault_out: Box<Account<'info, TokenAccount>>,

    pub owner: Signer<'info>,

    pub token_program: Program<'info, Token>,
}
