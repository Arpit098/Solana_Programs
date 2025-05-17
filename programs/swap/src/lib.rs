use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
    
};
use anchor_lang::solana_program::system_instruction;

declare_id!("6R4YsCRSwCCb9neCuCwJJgUfit3VoL8adJr3RJGXydB2");

pub const ADMIN: Pubkey = pubkey!("EWMh9mKq95AXjuDJyNHet91B7Xka9PuRYhQtmCTijYw");

#[program]
pub mod swap {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        price_per_token: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.authority.key() == ADMIN,
            CustomError::InvalidAuth
        );
        let vault = &mut ctx.accounts.vault;
        vault.token_mint = ctx.accounts.token_mint.key();
        vault.vault_token_account = ctx.accounts.vault_token_account.key();
        vault.price_per_token = price_per_token;
        vault.total_tokens = 0;
        vault.bump = ctx.bumps.vault;
        Ok(())
    }

    pub fn update_price(
        ctx: Context<UpdatePrice>,
        new_price: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.authority.key() == ADMIN,
            CustomError::InvalidAuth
        );
        let vault = &mut ctx.accounts.vault;
        vault.price_per_token = new_price;
        Ok(())
    }

    pub fn deposit_tokens(
        ctx: Context<DepositTokens>,
        amount: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.authority.key() == ADMIN,
            CustomError::InvalidAuth
        );
        let cpi_accounts = TransferChecked {
            mint: ctx.accounts.token_mint.to_account_info(),
            from: ctx.accounts.admin_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer_checked(cpi_ctx, amount, ctx.accounts.token_mint.decimals)?;
        let vault = &mut ctx.accounts.vault;
        vault.total_tokens += amount;
        Ok(())
    }

    pub fn purchase_tokens(
        ctx: Context<PurchaseTokens>,
        amount: u64,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        require!(amount <= vault.total_tokens, CustomError::InsufficientTokens);
        let total_price = amount
            .checked_mul(vault.price_per_token)
            .ok_or(CustomError::Overflow)?;

        // Lamports transfer via CPI to system program
        let ix = system_instruction::transfer(
            ctx.accounts.buyer.key,
            ctx.accounts.admin.key,
            total_price,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.admin.to_account_info(),
            ],
        )?;

        // Transfer tokens from vault to user
        let cpi_accounts = TransferChecked {
            mint: ctx.accounts.token_mint.to_account_info(),
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.vault_signer.to_account_info(),
        };
        let mint = ctx.accounts.token_mint.key();

        let seeds = &[
            b"vault",
            mint.as_ref(),
            &[vault.bump],
        ];
        let signer_seeds = &[&seeds[..]];
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        transfer_checked(cpi_ctx, amount, ctx.accounts.token_mint.decimals)?;
        vault.total_tokens -= amount;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        seeds = [b"vault", token_mint.key().as_ref()],
        bump,
        payer = authority,
        space = 8 + Vault::INIT_SPACE
    )]
    pub vault: Account<'info, Vault>,
    pub token_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = authority,
        token::mint = token_mint,
        token::authority = vault,
        token::token_program = token_program
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub vault: Account<'info, Vault>,
}

#[derive(Accounts)]
pub struct DepositTokens<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    pub token_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub admin_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct PurchaseTokens<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    /// CHECK: This is the admin recipient (must match ADMIN)
    #[account(mut, address = ADMIN)]
    pub admin: UncheckedAccount<'info>,
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    pub token_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: PDA signer for vault
    pub vault_signer: UncheckedAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Vault {
    pub token_mint: Pubkey,
    pub vault_token_account: Pubkey,
    pub price_per_token: u64,
    pub total_tokens: u64,
    pub bump: u8,
}

#[error_code]
pub enum CustomError {
    #[msg("Not authorized to call this transaction")]
    InvalidAuth,
    #[msg("Insufficient tokens remaining")]
    InsufficientTokens,
    #[msg("Arithmetic overflow")]
    Overflow,
}