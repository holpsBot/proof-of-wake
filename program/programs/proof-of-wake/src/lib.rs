use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;

declare_id!("2KhoiLTRRzVn4EoEcbHgdtoU4PNJrTxydTb2Mpm1VJbD");

#[program]
pub mod proof_of_wake {
    use super::*;

    pub fn initialize_treasury(ctx: Context<InitializeTreasury>) -> Result<()> {
        let treasury = &mut ctx.accounts.treasury;
        treasury.authority = ctx.accounts.authority.key();
        treasury.total_funded = 0;
        Ok(())
    }

    pub fn start_challenge(ctx: Context<StartChallenge>, alarm_hour: u8, alarm_minute: u8) -> Result<()> {
        let challenge = &mut ctx.accounts.challenge;
        let clock = Clock::get()?;

        challenge.authority = ctx.accounts.authority.key();
        challenge.start_ts = clock.unix_timestamp;
        challenge.last_wake_ts = 0;
        challenge.streak = 0;
        challenge.is_active = true;
        challenge.stake_amount = 100_000_000; // 0.1 SOL in lamports
        challenge.alarm_hour = alarm_hour;
        challenge.alarm_minute = alarm_minute;

        // Transfer 0.1 SOL from user to program vault (the challenge PDA itself)
        let ix = system_instruction::transfer(
            &ctx.accounts.authority.key(),
            &challenge.key(),
            challenge.stake_amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.challenge.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn complete_day(ctx: Context<CompleteDay>) -> Result<()> {
        let challenge = &mut ctx.accounts.challenge;
        let clock = Clock::get()?;

        // Logic for window check would go here.
        // For MVP, we'll just check if at least 20 hours have passed since last wake
        // to prevent double-calling in one morning, and that it's active.
        require!(challenge.is_active, ErrorCode::ChallengeInactive);
        
        if challenge.last_wake_ts != 0 {
            let elapsed = clock.unix_timestamp - challenge.last_wake_ts;
            require!(elapsed > 72000, ErrorCode::TooEarly); // 20 hours
        }

        challenge.last_wake_ts = clock.unix_timestamp;
        challenge.streak += 1;

        if challenge.streak == 21 {
            // payout stake + bonus
            let bonus = (challenge.stake_amount as f64 * 0.069) as u64;
            let total_payout = challenge.stake_amount + bonus;

            challenge.is_active = false;

            // Transfer from challenge PDA (stake) + Treasury (bonus)
            // Note: Challenge PDA holds the 0.1 SOL
            **challenge.to_account_info().try_borrow_mut_lamports()? -= challenge.stake_amount;
            **ctx.accounts.authority.to_account_info().try_borrow_mut_lamports()? += challenge.stake_amount;

            // Transfer bonus from treasury (this requires treasury PDA to sign)
            let seeds = &[b"treasury".as_ref(), &[ctx.bumps.treasury]];
            let signer = &[&seeds[..]];

            let bonus_ix = system_instruction::transfer(
                &ctx.accounts.treasury.key(),
                &ctx.accounts.authority.key(),
                bonus,
            );
            anchor_lang::solana_program::program::invoke_signed(
                &bonus_ix,
                &[
                    ctx.accounts.treasury.to_account_info(),
                    ctx.accounts.authority.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                signer,
            )?;
        }

        Ok(())
    }

    pub fn slash(ctx: Context<Slash>) -> Result<()> {
        let challenge = &mut ctx.accounts.challenge;
        let clock = Clock::get()?;

        // Check if deadline missed.
        // For MVP: if more than 48 hours passed since last wake (or start)
        let last_event = if challenge.last_wake_ts == 0 { challenge.start_ts } else { challenge.last_wake_ts };
        let elapsed = clock.unix_timestamp - last_event;
        
        require!(elapsed > 172800, ErrorCode::NotSlashableYet); // 48 hours
        require!(challenge.is_active, ErrorCode::ChallengeInactive);

        challenge.is_active = false;
        let amount = challenge.stake_amount;
        let dev_commission = (amount as f64 * 0.069) as u64;
        let treasury_share = amount - dev_commission;

        // Transfer from challenge PDA
        **challenge.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.dev.to_account_info().try_borrow_mut_lamports()? += dev_commission;
        **ctx.accounts.treasury.to_account_info().try_borrow_mut_lamports()? += treasury_share;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeTreasury<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8,
        seeds = [b"treasury"],
        bump
    )]
    pub treasury: Account<'info, Treasury>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StartChallenge<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8 + 8 + 8 + 1 + 8 + 1 + 1,
        seeds = [b"challenge", authority.key().as_ref()],
        bump
    )]
    pub challenge: Account<'info, Challenge>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CompleteDay<'info> {
    #[account(mut, seeds = [b"challenge", authority.key().as_ref()], bump)]
    pub challenge: Account<'info, Challenge>,
    #[account(mut, seeds = [b"treasury"], bump)]
    pub treasury: Account<'info, Treasury>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Slash<'info> {
    #[account(mut, seeds = [b"challenge", challenge.authority.as_ref()], bump)]
    pub challenge: Account<'info, Challenge>,
    #[account(mut, seeds = [b"treasury"], bump)]
    pub treasury: Account<'info, Treasury>,
    #[account(mut)]
    pub dev: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Treasury {
    pub authority: Pubkey,
    pub total_funded: u64,
}

#[account]
pub struct Challenge {
    pub authority: Pubkey,
    pub start_ts: i64,
    pub last_wake_ts: i64,
    pub streak: u8,
    pub is_active: bool,
    pub stake_amount: u64,
    pub alarm_hour: u8,
    pub alarm_minute: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Challenge is not active.")]
    ChallengeInactive,
    #[msg("Too early to complete the next day.")]
    TooEarly,
    #[msg("The user hasn't missed the deadline yet.")]
    NotSlashableYet,
}
