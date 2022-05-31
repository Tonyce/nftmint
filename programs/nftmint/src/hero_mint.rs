use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token::{
    self, InitializeAccount, InitializeMint, Mint, MintTo, SetAuthority, Token, TokenAccount,
};
use mpl_token_metadata::instruction::{
    create_master_edition_v3, create_metadata_accounts_v2,
    mint_edition_from_master_edition_via_vault_proxy,
};
use spl_token::instruction::AuthorityType;

use crate::ID;

pub const HERO_MINT_SEED: &[u8] = b"hero_mint_seed";
pub const HERO_MINT_TOKEN_ACCOUNT_SEED: &[u8] = b"hero_mint_token_account_seed";

#[derive(Clone)]
pub struct TokenMetadata;

impl anchor_lang::Id for TokenMetadata {
    fn id() -> Pubkey {
        mpl_token_metadata::ID
    }
}

#[derive(Accounts)]
#[instruction(name: String, symbol: String, uri: String)]
pub struct HeroMint<'info> {
    #[account(
        init,
        payer = user,
        space = Mint::LEN,
        owner = token_program.key(),
        seeds = [HERO_MINT_SEED, ID.as_ref(), name_seed(&name), name_seed(&symbol), name_seed(&uri)], bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub hero_mint: AccountInfo<'info>,
    #[account(
        init,
        payer = user,
        owner = token_program.key(),
        space = TokenAccount::LEN,
        seeds = [HERO_MINT_TOKEN_ACCOUNT_SEED, ID.as_ref(), name_seed(&name), name_seed(&symbol), name_seed(&uri)], bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub hero_token_account: AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub hero_metadata_account: AccountInfo<'info>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub mpl_program: Program<'info, TokenMetadata>,
}

impl<'info> HeroMint<'info> {
    fn initialize_mint_context(&self) -> CpiContext<'_, '_, '_, 'info, InitializeMint<'info>> {
        let cpi_accounts = InitializeMint {
            mint: self.hero_mint.clone(),
            rent: self.rent.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
    fn initialize_mint_to_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: self.hero_mint.clone(),
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

    fn disable_mint_to_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.hero_mint.to_account_info(),
            current_authority: self.user.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

pub fn handler(ctx: Context<HeroMint>, name: String, symbol: String, uri: String) -> Result<()> {
    // create mint account
    token::initialize_mint(
        ctx.accounts.initialize_mint_context(),
        0,
        &ctx.accounts.user.key,
        None,
    )?;
    msg!("create mint account");

    // create spl token account
    token::initialize_account(ctx.accounts.initialize_account_context())?;
    msg!("create spl token account");

    // // mint 1 nft
    token::mint_to(ctx.accounts.initialize_mint_to_context(), 1)?;

    // create metadata
    solana_program::program::invoke(
        &create_metadata_accounts_v2(
            mpl_token_metadata::ID,
            ctx.accounts.hero_metadata_account.key(),
            ctx.accounts.hero_mint.to_account_info().key(),
            ctx.accounts.user.to_account_info().key(),
            ctx.accounts.user.to_account_info().key(),
            ctx.accounts.user.to_account_info().key(),
            name,
            symbol,
            uri,
            None,
            0,
            true,
            false,
            None,
            None,
        ),
        &[
            ctx.accounts.hero_metadata_account.clone(),
            ctx.accounts.hero_mint.to_account_info(),
            ctx.accounts.user.to_account_info(),
            ctx.accounts.user.to_account_info(),
            ctx.accounts.user.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
    )?;

    // mint disable
    // let cpi_accounts = SetAuthority {
    //     account_or_mint: ctx.accounts.hero_mint.to_account_info(),
    //     current_authority: ctx.accounts.user.to_account_info(),
    // };
    // let ctx_auth = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    // token::set_authority(ctx_auth, AuthorityType::MintTokens, None)?;

    token::set_authority(
        ctx.accounts.disable_mint_to_context(),
        AuthorityType::MintTokens,
        None,
    )?;

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
