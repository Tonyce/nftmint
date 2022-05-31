use anchor_lang::prelude::*;

use crate::ID;
use anchor_lang::solana_program;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, InitializeAccount, InitializeMint, Mint, MintTo, Token, TokenAccount},
};
use mpl_token_metadata::instruction::{
    create_master_edition_v3, create_metadata_accounts_v2,
    mint_edition_from_master_edition_via_vault_proxy,
};
use spl_associated_token_account::ID as AssociatedTokenAccountID;

pub const HERO_MINT_SEED: &[u8] = b"hero_mint_seed";
pub const HERO_MINT_TOKEN_ACCOUNT_SEED: &[u8] = b"hero_mint_token_account_seed";

#[derive(Clone)]
pub struct AssociatedTokenAccount;

impl anchor_lang::Id for AssociatedTokenAccount {
    fn id() -> Pubkey {
        spl_associated_token_account::ID
    }
}

#[derive(Accounts)]
#[instruction(name: String, symbol: String, uri: String)]
pub struct MintTokenAccount<'info> {
    #[account(
        init,
        payer = user,
        space = Mint::LEN,
        owner = token_program.key(),
        seeds = [b"nft-mint-seed"], bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub hero_mint: AccountInfo<'info>,
    // #[account(mut)]
    // pub hero_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = hero_mint.to_account_info(),
        associated_token::authority = user,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub hero_token_account: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    // #[account(mut)]
    // pub hero_metadata_account: AccountInfo<'info>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> MintTokenAccount<'info> {
    fn initialize_mint_context(&self) -> CpiContext<'_, '_, '_, 'info, InitializeMint<'info>> {
        let cpi_accounts = InitializeMint {
            mint: self.hero_mint.clone(),
            rent: self.rent.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
    fn initialize_mint_to_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: self.hero_mint.to_account_info(),
            to: self.hero_token_account.to_account_info(),
            authority: self.user.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
    fn initialize_account_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, InitializeAccount<'info>> {
        let cpi_accounts = InitializeAccount {
            account: self.hero_token_account.to_account_info(),
            mint: self.hero_mint.to_account_info(),
            authority: self.user.to_account_info(),
            rent: self.rent.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

pub fn handler(
    ctx: Context<MintTokenAccount>,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    // create mint account
    token::initialize_mint(
        ctx.accounts.initialize_mint_context(),
        0,
        &ctx.accounts.user.key,
        None,
    )?;
    msg!("create mint account");

    // create spl token account
    // token::initialize_account(ctx.accounts.initialize_account_context())?;
    msg!("create spl token account");

    // // // mint 1 nft
    token::mint_to(ctx.accounts.initialize_mint_to_context(), 1)?;

    Ok(())
}

fn name_seed(name: &str) -> &[u8] {
    let b = name.as_bytes();
    if b.len() > 32 {
        &b[0..32]
    } else {
        b
    }
}
