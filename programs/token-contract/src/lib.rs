use anchor_lang::{prelude::*};
use anchor_spl::token;
use anchor_spl::token::{Token, MintTo, Transfer};


// stakeToken
declare_id!("EUKCTHaPV7PyuGfFhxQG8WbEu1YK8gnuCZcAUZm25rbm");

#[program]
pub mod token_contract {
    use super::*;

    pub fn mint_token(ctx: Context<MintToken>, amount: u64) -> Result<()> {
        // Create the MintTo struct for our context
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        // Create the CpiContext we need for the request
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        // Execute anchor's helper function to mint tokens
        token::mint_to(cpi_ctx, amount)?;
        
        Ok(())
    }

    pub fn transfer_token(ctx: Context<TransferToken>, amount: u64) -> Result<()> {
        // Create the Transfer struct for our context
        let transfer_instruction = Transfer{
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.from_authority.to_account_info(),
        };
         
        let cpi_program = ctx.accounts.token_program.to_account_info();
        // Create the Context for our Transfer request
        let cpi_ctx = CpiContext::new(cpi_program, transfer_instruction);

        // Execute anchor's helper function to transfer tokens
        anchor_spl::token::transfer(cpi_ctx, amount)?;
 
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()>  {
        const DAYS: i64 = 60 * 60 * 24;

        require!(
            ctx.accounts.store.staked_count > 0,
            AlreadyStaked
        );
        require!(
            amount > ctx.accounts.store.max_staked_amount,
            ToManyTokensStake
        );
        
        let transfer_instruction = Transfer {
            from: ctx.accounts.transfer.from.to_account_info(),
            to: ctx.accounts.transfer.to.to_account_info(),
            authority: ctx.accounts.transfer.from_authority.to_account_info(),
        };
         
        let cpi_program = ctx.accounts.transfer.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, transfer_instruction);

        anchor_spl::token::transfer(cpi_ctx, amount)?;

        Ok({ 
            ctx.accounts.store.staked_count = amount;
            ctx.accounts.store.max_staked_amount = 10_000;
            ctx.accounts.store.time_start = ctx.accounts.clock.unix_timestamp; 
            ctx.accounts.store.rounds_amount = 24;
            ctx.accounts.store.rounds_time = 7 * DAYS;
            ctx.accounts.store.tokens_per_period = 70;
            ctx.accounts.store.rounds_passed = 0;
        })
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()>  {
        const DAYS: i64 = 60 * 60 * 24;

        require!(
            ctx.accounts.store.staked_count <= 0,
            NothingStaked
        );

        let rounds_claim: u64 = ((ctx.accounts.clock.unix_timestamp - ctx.accounts.store.time_start) / ctx.accounts.store.rounds_time).try_into().unwrap();
        
        require!(
            rounds_claim == 0 ||
            ctx.accounts.store.rounds_passed == rounds_claim ||
            ctx.accounts.store.rounds_passed <= ctx.accounts.store.rounds_amount,
            NothingToClaim
        );

        let amount: u64 = (rounds_claim - ctx.accounts.store.rounds_passed) * ctx.accounts.store.tokens_per_period;

        let transfer_instruction = Transfer{
            from: ctx.accounts.transfer.from.to_account_info(),
            to: ctx.accounts.transfer.to.to_account_info(),
            authority: ctx.accounts.transfer.from_authority.to_account_info(),
        };
         
        let cpi_program = ctx.accounts.transfer.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, transfer_instruction);

        anchor_spl::token::transfer(cpi_ctx, amount)?;

        Ok({ 
            ctx.accounts.store.staked_count = ctx.accounts.store.staked_count;
            ctx.accounts.store.max_staked_amount = 10_000;
            ctx.accounts.store.time_start = ctx.accounts.store.time_start; 
            ctx.accounts.store.rounds_amount = 24;
            ctx.accounts.store.rounds_time = 7 * DAYS;
            ctx.accounts.store.tokens_per_period = 70;
            ctx.accounts.store.rounds_passed = rounds_claim;
        })
    }

    pub fn exit(ctx: Context<Exit>) -> Result<()>  {
        const DAYS: i64 = 60 * 60 * 24;

        require!(
            ctx.accounts.store.staked_count <= 0,
            NothingStaked
        );

        let mut amount: u64 = 0;

        let rounds_claim: u64 = ((ctx.accounts.clock.unix_timestamp - ctx.accounts.store.time_start) / ctx.accounts.store.rounds_time).try_into().unwrap();
        if rounds_claim == 0 ||
           ctx.accounts.store.rounds_passed == rounds_claim ||
           ctx.accounts.store.rounds_passed <= ctx.accounts.store.rounds_amount {
            amount += ctx.accounts.store.staked_count;
        } else {
            amount += (rounds_claim - ctx.accounts.store.rounds_passed) * ctx.accounts.store.tokens_per_period;
        }

        let transfer_instruction = Transfer{
            from: ctx.accounts.transfer.from.to_account_info(),
            to: ctx.accounts.transfer.to.to_account_info(),
            authority: ctx.accounts.transfer.from_authority.to_account_info(),
        };
         
        let cpi_program = ctx.accounts.transfer.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, transfer_instruction);

        anchor_spl::token::transfer(cpi_ctx, amount)?;

        Ok({ 
            ctx.accounts.store.staked_count = 0;
            ctx.accounts.store.max_staked_amount = 10_000;
            ctx.accounts.store.time_start = ctx.accounts.store.time_start; 
            ctx.accounts.store.rounds_amount = 24;
            ctx.accounts.store.rounds_time = 7 * DAYS;
            ctx.accounts.store.tokens_per_period = 70;
            ctx.accounts.store.rounds_passed = ctx.accounts.store.rounds_amount;
        })
    }

}

#[derive(Accounts)]
pub struct MintToken<'info> {
    /// CHECK: This is the token that we want to mint
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    /// CHECK: This is the token account that we want to mint tokens to
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    /// CHECK: the authority of the mint account
    #[account(mut)]
    pub authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct TransferToken<'info> {
    pub token_program: Program<'info, Token>,
    /// CHECK: The associated token account that we are transferring the token from
    #[account(mut)]
    pub from: UncheckedAccount<'info>,
    /// CHECK: The associated token account that we are transferring the token to
    #[account(mut)]
    pub to: AccountInfo<'info>,
    // the authority of the from account 
    pub from_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    pub transfer: TransferToken<'info>,
    pub clock: Sysvar<'info, Clock>,
    #[account(zero)]
    pub store: Account<'info, StakeStore>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    pub transfer: TransferToken<'info>,
    pub clock: Sysvar<'info, Clock>,
    #[account(zero)]
    pub store: Account<'info, StakeStore>,
}

#[derive(Accounts)]
pub struct Exit<'info> {
    pub transfer: TransferToken<'info>,
    pub mint: MintToken<'info>,
    pub clock: Sysvar<'info, Clock>,
    #[account(zero)]
    pub store: Account<'info, StakeStore>,
}

#[account]
pub struct StakeStore {
    pub staked_count: u64,
    pub max_staked_amount: u64,
    pub time_start: i64,
    pub rounds_amount: u64,
    pub rounds_time: i64,
    pub tokens_per_period: u64,
    pub rounds_passed: u64,
}

impl StakeStore {
    pub const LEN: usize = 32 + 8 + 32 + 8;
}

#[error_code]
pub enum ErrorCode {
    #[msg("Token is already staked")]
    AlreadyStaked,
    #[msg("No tokens staked")]
    NothingStaked,
    #[msg("Nothing to claim yet")]
    NothingToClaim,
    #[msg("Max amount of tokens to stake is: 10.000")]
    ToManyTokensStake,
}