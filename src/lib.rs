
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{
        self, Mint, MintTo, Token, TokenAccount,
        TransferChecked,
    },
};

declare_id!("ExTMJaP2jvB1Z4Fc3VZtNaq5k1it7kvoWYMffTnQBha9");

#[program]
pub mod vault_deposit {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let vault = &mut ctx.accounts.vault_state;
        vault.deposit_mint = ctx.accounts.deposit_mint.key();
        vault.total_assets = 0;
        vault.total_shares = 0;

        msg!("Vault initialized");
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        require!(amount > 0, VaultError::InvalidAmount);

        let vault = &ctx.accounts.vault_state;

        let shares_u128 = if vault.total_shares == 0 {
            require!(
                vault.total_assets == 0,
                VaultError::InvalidState
            );
            amount as u128
        } else {
            require!(
                vault.total_assets > 0,
                VaultError::InvalidState
            );

            (amount as u128)
                .checked_mul(vault.total_shares as u128)
                .ok_or(VaultError::MathOverflow)?
                .checked_div(vault.total_assets as u128)
                .ok_or(VaultError::MathOverflow)?
        };

        let shares = u64::try_from(shares_u128)
            .map_err(|_| error!(VaultError::MathOverflow))?;

        require!(shares > 0, VaultError::DepositTooSmall);

        let new_assets = vault.total_assets
            .checked_add(amount)
            .ok_or(VaultError::MathOverflow)?;

        let new_shares = vault.total_shares
            .checked_add(shares)
            .ok_or(VaultError::MathOverflow)?;

        // Transfer deposited tokens to vault.
        let transfer_accounts = TransferChecked {
            from: ctx.accounts.user_tokens.to_account_info(),
            mint: ctx.accounts.deposit_mint.to_account_info(),
            to: ctx.accounts.vault_tokens.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        token::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                transfer_accounts,
            ),
            amount,
            ctx.accounts.deposit_mint.decimals,
        )?;

        // Mint shares using vault PDA authority.
        let mint_key = ctx.accounts.deposit_mint.key();
        let bump = ctx.bumps.vault_state;

        let seeds: &[&[u8]] = &[
            b"vault",
            mint_key.as_ref(),
            &[bump],
        ];

        let signer = &[seeds];

        let mint_accounts = MintTo {
            mint: ctx.accounts.share_mint.to_account_info(),
            to: ctx.accounts.user_shares.to_account_info(),
            authority: ctx.accounts.vault_state.to_account_info(),
        };

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                mint_accounts,
                signer,
            ),
            shares,
        )?;

        let vault = &mut ctx.accounts.vault_state;
        vault.total_assets = new_assets;
        vault.total_shares = new_shares;

        msg!("Deposit successful");
        msg!("Deposited: {}", amount);
        msg!("Shares minted: {}", shares);

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub deposit_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = user,
        space = 8 + 32 + 8 + 8,
        seeds = [b"vault", deposit_mint.key().as_ref()],
        bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        init,
        payer = user,
        mint::decimals = deposit_mint.decimals,
        mint::authority = vault_state,
        seeds = [b"share_mint", deposit_mint.key().as_ref()],
        bump
    )]
    pub share_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = user,
        associated_token::mint = deposit_mint,
        associated_token::authority = vault_state
    )]
    pub vault_tokens: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub deposit_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"vault", deposit_mint.key().as_ref()],
        bump,
        constraint = vault_state.deposit_mint == deposit_mint.key()
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"share_mint", deposit_mint.key().as_ref()],
        bump,
        constraint = share_mint.mint_authority
            == anchor_lang::solana_program::program_option::COption::Some(
                vault_state.key()
            )
    )]
    pub share_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = deposit_mint,
        associated_token::authority = user
    )]
    pub user_tokens: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = deposit_mint,
        associated_token::authority = vault_state
    )]
    pub vault_tokens: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = share_mint,
        associated_token::authority = user
    )]
    pub user_shares: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct VaultState {
    pub deposit_mint: Pubkey,
    pub total_assets: u64,
    pub total_shares: u64,
}

#[error_code]
pub enum VaultError {
    #[msg("Deposit must be greater than zero")]
    InvalidAmount,
    #[msg("Arithmetic overflow")]
    MathOverflow,
    #[msg("Deposit is too small to mint shares")]
    DepositTooSmall,
    #[msg("Vault accounting state is invalid")]
    InvalidState,
}
