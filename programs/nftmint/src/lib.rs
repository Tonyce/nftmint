use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token::{
    self, InitializeAccount, InitializeMint, Mint, MintTo, Token, TokenAccount,
};
use mpl_token_metadata::instruction::create_metadata_accounts_v2;

mod hero_mint;
mod mint_with_tokenaccount;

use hero_mint::*;
use mint_with_tokenaccount::*;

declare_id!("EzskZvcwqeiNMnTjzyAReFgnH53QRxrztVfSJbmC5r3");

pub const NFT_MINT_SEED: &[u8] = b"nft-mint-seed";
pub const NFT_TOKEN_ACCOUNT_SEED: &[u8] = b"nft-token-account-seed";

#[program]
pub mod nftmint {
    use super::*;

    pub fn initialize(
        ctx: Context<NFTMint>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        token::initialize_mint(
            ctx.accounts.initialize_mint_context(),
            0,
            ctx.accounts.user.key,
            None,
        )?;
        msg!("create mint account");

        // create spl token account
        token::initialize_account(ctx.accounts.initialize_account_context())?;
        msg!("create spl token account");

        // mint 1 nft
        token::mint_to(ctx.accounts.initialize_mint_to_context(), 1)?;

        // create metadata
        solana_program::program::invoke(
            &create_metadata_accounts_v2(
                mpl_token_metadata::ID,
                ctx.accounts.nft_meta_data_account.key(),
                ctx.accounts.nft_mint_account.to_account_info().key(),
                ctx.accounts.user.to_account_info().key(),
                ctx.accounts.user.to_account_info().key(),
                ctx.accounts.user.to_account_info().key(),
                name,
                symbol,
                uri,
                None,
                0,
                true,
                true,
                None,
                None,
            ),
            &[
                ctx.accounts.nft_meta_data_account.clone(),
                ctx.accounts.nft_mint_account.to_account_info(),
                ctx.accounts.user.to_account_info(),
                ctx.accounts.user.to_account_info(),
                ctx.accounts.user.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
        )?;
        Ok(())
    }

    pub fn hero_mint(
        ctx: Context<HeroMint>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        hero_mint::handler(ctx, name, symbol, uri)
    }

    pub fn mint_with_tokenaccount(
        ctx: Context<MintTokenAccount>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        mint_with_tokenaccount::handler(ctx, name, symbol, uri)
    }
}

#[derive(Accounts)]
#[instruction(name: String, symbol: String)]
pub struct NFTMint<'info> {
    // Do this instruction when the parent do NOT has any metadata associated
    // with it. This is checked offchain before sending this tx.
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub nft_meta_data_account: AccountInfo<'info>,

    #[account(
        init,
        payer = user,
        space = Mint::LEN,
        owner = token_program.key(),
        seeds = [
            NFT_MINT_SEED,
            user.to_account_info().key.as_ref(),
            name_seed(&name),
            name_seed(&symbol)
        ],
        bump
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_mint_account: AccountInfo<'info>,

    #[account(
        init,
        payer = user,
        owner = token_program.key(),
        space = TokenAccount::LEN,
        seeds = [
            NFT_TOKEN_ACCOUNT_SEED,
            user.to_account_info().key.as_ref(),
            name_seed(&name),
            name_seed(&symbol)
        ],
        bump
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub mpl_program: Program<'info, TokenMetadata>,
}

impl<'info> NFTMint<'info> {
    fn initialize_mint_context(&self) -> CpiContext<'_, '_, '_, 'info, InitializeMint<'info>> {
        let cpi_accounts = InitializeMint {
            mint: self.nft_mint_account.clone(),
            rent: self.rent.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn initialize_mint_to_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: self.nft_mint_account.clone(),
            to: self.nft_token_account.to_account_info(),
            authority: self.user.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn initialize_account_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, InitializeAccount<'info>> {
        let cpi_accounts = InitializeAccount {
            account: self.nft_token_account.to_account_info(),
            mint: self.nft_mint_account.to_account_info(),
            authority: self.user.to_account_info(),
            rent: self.rent.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[derive(Clone)]
pub struct TokenMetadata;

impl anchor_lang::Id for TokenMetadata {
    fn id() -> Pubkey {
        mpl_token_metadata::ID
    }
}

pub fn name_seed(name: &str) -> &[u8] {
    let b = name.as_bytes();
    if b.len() > 32 {
        &b[0..32]
    } else {
        b
    }
}
