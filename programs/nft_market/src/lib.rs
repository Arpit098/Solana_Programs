#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

declare_id!("9Pzvc1QpDtH9THVGgTWTeC3HYohab93i1ULDG3BJpxHs");

#[program]
mod marketplace {
    use super::*;

    pub fn create_auction(
        ctx: Context<CreateAuction>,
        nft_name: String,
        price: u64,
        validity: u64,
        wallet_address: Pubkey,
    ) -> Result<()> {
        require!(
            ctx.accounts.token_mint.decimals == 0,
            ErrorCode::InvalidNftDecimals
        );

        let expected_owner = pubkey!("7AZnxzSjwaYvcrGEJbek15zd13PX8VfeNAFeuQ8zDaQR");
        require!(
            ctx.accounts.owner.key() == expected_owner,
            ErrorCode::InvalidAuth
        );

        let auction = &mut ctx.accounts.auction;
        auction.nft_name = nft_name;
        auction.price = price;
        auction.tokens_left = 1;
        auction.validity = validity;
        auction.token_address = ctx.accounts.token_mint.key();
        auction.bump = ctx.bumps.auction;
        auction.creator = wallet_address;

        transfer_tokens_checked(
            &ctx.accounts.maker_token_account,
            &ctx.accounts.vault,
            1,
            &ctx.accounts.token_mint,
            &ctx.accounts.owner,
            &ctx.accounts.token_program,
        )?;

        Ok(())
    }

    pub fn buy_token(ctx: Context<PurchaseToken>) -> Result<()> {
        let auction = &mut ctx.accounts.auction;
        require!(auction.tokens_left > 0, ErrorCode::InsufficientTokens);
        
        let clock = Clock::get()?;
        require!(
            (clock.unix_timestamp as u64) < auction.validity,
            ErrorCode::Expired
        );
        auction.tokens_left = auction.tokens_left.checked_sub(1).ok_or(ErrorCode::Overflow)?;

        transfer_tokens_checked(
            &ctx.accounts.buyer_token_account_money,
            &ctx.accounts.owner_token_recieve_account,
            auction.price,
            &ctx.accounts.buyer_token_mint,
            &ctx.accounts.buyer,
            &ctx.accounts.token_program,
        )?;

        let token_address_key = auction.token_address.key();     
        let creator_key = auction.creator.key();                 
        let bump = [auction.bump];                          

        let seeds = &[
            token_address_key.as_ref(),
            creator_key.as_ref(),
            &bump[..],
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

        transfer_checked(cpi_context, 1, ctx.accounts.token_mint.decimals)?;
        Ok(())
    }

    pub fn recreate_auction(
        ctx: Context<ReCreateAuction>,
        price: u64,
        wallet_address: Pubkey
    ) -> Result<()> {

        require!(
            ctx.accounts.token_mint.decimals == 0,
            ErrorCode::InvalidNftDecimals
        );
    
        let original_auction = &ctx.accounts.original_auction;
        let auction = &mut ctx.accounts.auction;
        
        auction.nft_name = original_auction.nft_name.clone();
        auction.price = price;
        auction.tokens_left = 1;
        auction.validity = original_auction.validity;
        auction.token_address = ctx.accounts.token_mint.key();
        auction.bump = ctx.bumps.auction;
        auction.creator = wallet_address;
    
        transfer_tokens_checked(
            &ctx.accounts.maker_token_account,
            &ctx.accounts.vault,
            1,
            &ctx.accounts.token_mint,
            &ctx.accounts.maker,
            &ctx.accounts.token_program,
        )?;
    
        Ok(())
    }
}

fn transfer_tokens_checked<'info>(
    from: &InterfaceAccount<'info, TokenAccount>,
    to: &InterfaceAccount<'info, TokenAccount>,
    amount: u64,
    mint: &InterfaceAccount<'info, Mint>,
    authority: &Signer<'info>,
    token_program: &Interface<'info, TokenInterface>,
) -> Result<()> {
    let transfer_accounts = TransferChecked {
        from: from.to_account_info(),
        mint: mint.to_account_info(),
        to: to.to_account_info(),
        authority: authority.to_account_info(),
    };

    let cpi_context = CpiContext::new(
        token_program.to_account_info(),
        transfer_accounts,
    );

    transfer_checked(cpi_context, amount, mint.decimals)
}

#[derive(Accounts)]
pub struct CreateAuction<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        seeds = [
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
#[instruction(price: u64)]
pub struct ReCreateAuction<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        mint::token_program = token_program
    )]
    pub token_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: Verified through the constraint on original_auction
    pub original_creator: UncheckedAccount<'info>,

    #[account(
        seeds = [
            token_mint.key().as_ref(),
            original_creator.key().as_ref()
        ],
        bump,
        constraint = original_auction.token_address == token_mint.key() @ ErrorCode::InvalidNftAddress,
        constraint = original_auction.creator == original_creator.key() @ ErrorCode::InvalidOriginalCreator
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
       seeds = [maker.key().as_ref(), token_mint.key().as_ref()], 
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
    pub price: u64,
    #[max_len(50)]
    pub nft_name: String,
    pub token_address: Pubkey,
    pub validity: u64,
    pub bump: u8,
    pub tokens_left: u8,
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
    #[msg("Not an existing nft_name")]
    InvalidNftName,
    #[msg("Not an existing nft_address")]
    InvalidNftAddress,
    #[msg("Invalid NFT decimals - must be 0")]
    InvalidNftDecimals,
    #[msg("Original Auction creator is not the same")]
    InvalidOriginalCreator
}