#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

declare_id!("28RhbMHvXo8gQA5UiDDSSRfDdG1nzezWiTYr8aZVEmww");

#[program]
mod demo {
    use super::*;

    pub fn create_auction(
        ctx: Context<CreateAuction>,
        serialnum: u64,
        gpu: String,
        token_amount: u64,
        price: u64,
        validity: u64,
        wallet_address: Pubkey,
    ) -> Result<()> {
        let expected_owner = pubkey!("EWMh9mKq95AXjuDJyNHet91B7Xka9PuRYhQtmCTijYw");
        require!(
            ctx.accounts.owner.key() == expected_owner,
            ErrorCode::InvalidAuth
        );
        let auction = &mut ctx.accounts.auction;
        auction.serial_num = serialnum;
        auction.gpu = gpu;
        auction.token_amount = token_amount;
        auction.price_per_unit = price;
        auction.tokens_left = token_amount;
        auction.validity = validity;
        auction.token_address = ctx.accounts.token_mint.key();
        auction.bump = ctx.bumps.auction;
        auction.creator = wallet_address;

        transfer_tokens(
            &ctx.accounts.maker_token_account,
            &ctx.accounts.vault,
            &token_amount,
            &ctx.accounts.token_mint,
            &ctx.accounts.owner,
            &ctx.accounts.token_program,
        );

        Ok(())
    }

   pub fn buy_token(ctx: Context<PurchaseToken>, amount: u64) -> Result<()> {
    let auction = &mut ctx.accounts.auction;
    require!(amount <= auction.tokens_left, ErrorCode::InsufficientTokens);

    let total_price = amount
        .checked_mul(auction.price_per_unit)
        .ok_or(ErrorCode::Overflow)?;
    auction.tokens_left -= amount;

    // Pay the creator
    transfer_tokens(
        &ctx.accounts.buyer_token_account_money,
        &ctx.accounts.owner_token_recieve_account,
        &total_price,
        &ctx.accounts.buyer_token_mint,
        &ctx.accounts.buyer,
        &ctx.accounts.token_program,
    );

    // Transfer tokens from vault to buyer
    let serial_num_bytes = auction.serial_num.to_le_bytes(); 
    let token_address_key = auction.token_address.key();     
    let creator_key = auction.creator.key();                 
    let bump = [auction.bump];                          

    let seeds = &[
        serial_num_bytes.as_ref(),
        token_address_key.as_ref(),
        creator_key.as_ref(),
        &bump[..], // Use slice of the bump array
    ];
    let signer_seeds = [&seeds[..]];

    let accounts = TransferChecked {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.buyer_token_account_item.to_account_info(),
        mint: ctx.accounts.token_mint.to_account_info(),
        authority: ctx.accounts.auction.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );

    transfer_checked(cpi_context, amount, ctx.accounts.token_mint.decimals)?;

    Ok(())
}

    pub fn recreate_auction(
        ctx: Context<ReCreateAuction>,
        serialnum: u64,
        gpu: String,
        token_amount: u64,
        price: u64,
        validity: u64,
        wallet_address: Pubkey,
    ) -> Result<()> {
        let auction = &mut ctx.accounts.auction;
        auction.serial_num = serialnum;
        auction.gpu = gpu;
        auction.token_amount = token_amount;
        auction.price_per_unit = price;
        auction.tokens_left = token_amount;
        auction.validity = validity;
        auction.token_address = ctx.accounts.token_mint.key();
        auction.bump = ctx.bumps.auction;
        auction.creator = wallet_address;

        transfer_tokens(
            &ctx.accounts.maker_token_account,
            &ctx.accounts.vault,
            &token_amount,
            &ctx.accounts.token_mint,
            &ctx.accounts.maker,
            &ctx.accounts.token_program,
        );

        Ok(())
    }
}
fn transfer_tokens<'info>(
    from: &InterfaceAccount<'info, TokenAccount>,
    to: &InterfaceAccount<'info, TokenAccount>,
    amount: &u64,
    mint: &InterfaceAccount<'info, Mint>,
    authority: &Signer<'info>,
    token_program: &Interface<'info, TokenInterface>,
) {
    let transfer_accounts_options = TransferChecked {
        from: from.to_account_info(),
        mint: mint.to_account_info(),
        to: to.to_account_info(),
        authority: authority.to_account_info(),
    };

    let cpi_context = CpiContext::new(token_program.to_account_info(), transfer_accounts_options);

    let result = transfer_checked(cpi_context, *amount, mint.decimals);
    match result {
        Ok(_) => (),
        Err(err) => msg!("Transfer failed: {:?}", err),
    }
}


#[derive(Accounts)]
#[instruction(serialnum: u64)]
pub struct CreateAuction<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        seeds = [
            serialnum.to_le_bytes().as_ref(),
            token_mint.key().as_ref(),
            owner.key().as_ref()
        ],
        bump,
        payer = owner,
        space = 8 + Auction::INIT_SPACE
    )]
    pub auction: Account<'info, Auction>,

    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = owner,
        associated_token::token_program = token_program
    )]
    pub maker_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = owner,
        associated_token::mint = token_mint,
        associated_token::authority = auction,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(serial_num: u64)]
pub struct PurchaseToken<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(mut)]
    pub owner: SystemAccount<'info>,

    pub token_mint: InterfaceAccount<'info, Mint>,
    pub buyer_token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = buyer_token_mint,
        associated_token::authority = buyer,
        associated_token::token_program = token_program,
    )]
    pub buyer_token_account_money: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = buyer,
        associated_token::token_program = token_program,
    )]
    pub buyer_token_account_item: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = buyer_token_mint,
        associated_token::authority = owner,
        associated_token::token_program = token_program,
    )]
    pub owner_token_recieve_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = auction.token_address == token_mint.key(),
        seeds = [
          auction.serial_num.to_le_bytes().as_ref(),
          token_mint.key().as_ref(),
          owner.key().as_ref()
        ],
        bump = auction.bump
    )]
    pub auction: Account<'info, Auction>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = auction,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(serial_num: u64, gpu: String)]
pub struct ReCreateAuction<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        mint::token_program = token_program
    )]
    pub token_mint: InterfaceAccount<'info, Mint>,

    // Add a reference to the original auction account to check values against
    #[account(
        seeds = [
            serial_num.to_le_bytes().as_ref(),
            token_mint.key().as_ref(),
            original_auction.creator.key().as_ref()
        ],
        bump,
        constraint = original_auction.serial_num == serial_num @ ErrorCode::InvalidSerialNumber,
        constraint = original_auction.gpu == gpu @ ErrorCode::InvalidGpu,
    )]
    pub original_auction: Account<'info, Auction>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
       init,
       seeds = [serial_num.to_le_bytes().as_ref(), maker.key().as_ref(), token_mint.key().as_ref()], 
       bump, 
       payer = maker,
       space = 8 + Auction::INIT_SPACE
    )]
    pub auction: Account<'info, Auction>,

    #[account(
        init,
        payer = maker,
        associated_token::mint = token_mint,
        associated_token::authority = auction,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[account]
#[derive(InitSpace)]
pub struct Auction {
    pub creator: Pubkey,
    pub serial_num: u64,
    #[max_len(50)]
    pub gpu: String,
    pub token_amount: u64,
    pub price_per_unit: u64,
    pub tokens_left: u64,
    pub token_address: Pubkey,
    pub validity: u64,
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid token amount")]
    InvalidAmount,
    #[msg("Insufficient tokens remaining")]
    InsufficientTokens,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Auction expired")]
    Expired,
    #[msg("Not authorized to call this transaction")]
    InvalidAuth,
    #[msg("Not an existing gpu")]
    InvalidGpu,
    #[msg("Not an existing serial number")]
    InvalidSerialNumber
}
