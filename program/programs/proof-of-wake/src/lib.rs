use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;

declare_id!("3XY5vp1p4Q9fCeCwQNz3yMikYZhoXFJDDmEp6dBXMpx4");

// Hardcoded dev wallet for slash commissions
const DEV_WALLET: Pubkey = solana_program::pubkey!("HUvXWZcteeatc6LRCn35yCH3kxetq3JcEcD923avQ37Y");

// 6.9% as basis points (690 / 10000)
const COMMISSION_BPS: u64 = 690;
const BPS_DENOMINATOR: u64 = 10000;

// Alarm window: ±1 hour (3600 seconds) around the target wake time
const ALARM_WINDOW_SECONDS: i64 = 3600;

// Stake amount limits (in lamports)
const MIN_STAKE_LAMPORTS: u64 = 10_000_000;   // 0.01 SOL minimum
const MAX_STAKE_LAMPORTS: u64 = 1_000_000_000; // 1 SOL maximum

#[program]
pub mod proof_of_wake {
    use super::*;

    pub fn initialize_treasury(ctx: Context<InitializeTreasury>) -> Result<()> {
        let treasury = &mut ctx.accounts.treasury;
        treasury.authority = ctx.accounts.authority.key();
        treasury.total_funded = 0;
        treasury.bump = ctx.bumps.treasury;
        Ok(())
    }

    pub fn fund_treasury(ctx: Context<FundTreasury>, amount: u64) -> Result<()> {
        let treasury = &mut ctx.accounts.treasury;
        
        let ix = system_instruction::transfer(
            &ctx.accounts.funder.key(),
            &treasury.key(),
            amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.funder.to_account_info(),
                treasury.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        
        treasury.total_funded = treasury.total_funded.checked_add(amount).unwrap();
        Ok(())
    }

    pub fn start_challenge(
        ctx: Context<StartChallenge>, 
        alarm_hour: u8, 
        alarm_minute: u8,
        timezone_offset_seconds: i32,
        stake_amount: u64,
    ) -> Result<()> {
        require!(alarm_hour < 24, ErrorCode::InvalidAlarmTime);
        require!(alarm_minute < 60, ErrorCode::InvalidAlarmTime);
        // Timezone offset should be reasonable (-12h to +14h in seconds)
        require!(timezone_offset_seconds >= -43200 && timezone_offset_seconds <= 50400, ErrorCode::InvalidTimezone);
        // Stake must be within bounds
        require!(stake_amount >= MIN_STAKE_LAMPORTS, ErrorCode::StakeTooLow);
        require!(stake_amount <= MAX_STAKE_LAMPORTS, ErrorCode::StakeTooHigh);

        let challenge = &mut ctx.accounts.challenge;
        let clock = Clock::get()?;

        challenge.authority = ctx.accounts.authority.key();
        challenge.start_ts = clock.unix_timestamp;
        challenge.last_wake_ts = 0;
        challenge.streak = 0;
        challenge.is_active = true;
        challenge.stake_amount = stake_amount;
        challenge.alarm_hour = alarm_hour;
        challenge.alarm_minute = alarm_minute;
        challenge.timezone_offset = timezone_offset_seconds;
        challenge.bump = ctx.bumps.challenge;

        // Transfer 0.1 SOL from user to challenge PDA vault
        let ix = system_instruction::transfer(
            &ctx.accounts.authority.key(),
            &challenge.key(),
            challenge.stake_amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.authority.to_account_info(),
                challenge.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn complete_day(ctx: Context<CompleteDay>) -> Result<()> {
        let challenge = &mut ctx.accounts.challenge;
        let clock = Clock::get()?;

        require!(challenge.is_active, ErrorCode::ChallengeInactive);
        
        // Ensure at least 20 hours since last wake to prevent gaming
        if challenge.last_wake_ts != 0 {
            let elapsed = clock.unix_timestamp - challenge.last_wake_ts;
            require!(elapsed > 72000, ErrorCode::TooEarly); // 20 hours
        }

        // ===== ALARM WINDOW VALIDATION =====
        // Convert current UTC time to user's local time
        let local_timestamp = clock.unix_timestamp + (challenge.timezone_offset as i64);
        
        // Calculate seconds since midnight in local time
        // 86400 = seconds per day
        let seconds_since_midnight = local_timestamp.rem_euclid(86400);
        
        // Calculate target wake time in seconds since midnight
        let target_seconds = (challenge.alarm_hour as i64) * 3600 + (challenge.alarm_minute as i64) * 60;
        
        // Calculate difference, handling wrap-around at midnight
        let mut diff = (seconds_since_midnight - target_seconds).abs();
        if diff > 43200 {
            // Handle wrap-around (e.g., 23:00 vs 01:00 should be 2 hours, not 22)
            diff = 86400 - diff;
        }
        
        require!(diff <= ALARM_WINDOW_SECONDS, ErrorCode::OutsideAlarmWindow);
        // ===== END ALARM VALIDATION =====

        challenge.last_wake_ts = clock.unix_timestamp;
        challenge.streak = challenge.streak.checked_add(1).unwrap();

        if challenge.streak == 21 {
            // Calculate bonus using integer math: 6.9% = 690 basis points
            let bonus = challenge
                .stake_amount
                .checked_mul(COMMISSION_BPS)
                .unwrap()
                .checked_div(BPS_DENOMINATOR)
                .unwrap();

            // Check treasury has enough for bonus
            let treasury_balance = ctx.accounts.treasury.to_account_info().lamports();
            let treasury_rent = Rent::get()?.minimum_balance(ctx.accounts.treasury.to_account_info().data_len());
            let available = treasury_balance.saturating_sub(treasury_rent);
            
            // If treasury can't cover bonus, just return stake (no bonus)
            let actual_bonus = if available >= bonus { bonus } else { 0 };

            challenge.is_active = false;

            // Return stake from challenge PDA to user
            **challenge.to_account_info().try_borrow_mut_lamports()? -= challenge.stake_amount;
            **ctx.accounts.authority.to_account_info().try_borrow_mut_lamports()? += challenge.stake_amount;

            // Transfer bonus from treasury if available
            if actual_bonus > 0 {
                **ctx.accounts.treasury.to_account_info().try_borrow_mut_lamports()? -= actual_bonus;
                **ctx.accounts.authority.to_account_info().try_borrow_mut_lamports()? += actual_bonus;
                
                // Update treasury accounting
                let treasury = &mut ctx.accounts.treasury;
                treasury.total_funded = treasury.total_funded.checked_sub(actual_bonus).unwrap();
            }
        }

        Ok(())
    }

    pub fn slash(ctx: Context<Slash>) -> Result<()> {
        let challenge = &mut ctx.accounts.challenge;
        let clock = Clock::get()?;

        require!(challenge.is_active, ErrorCode::ChallengeInactive);

        // Validate dev wallet is the hardcoded address
        require!(ctx.accounts.dev.key() == DEV_WALLET, ErrorCode::InvalidDevWallet);

        // Check if deadline missed: 48 hours since last activity
        let last_event = if challenge.last_wake_ts == 0 { 
            challenge.start_ts 
        } else { 
            challenge.last_wake_ts 
        };
        let elapsed = clock.unix_timestamp - last_event;
        
        require!(elapsed > 172800, ErrorCode::NotSlashableYet); // 48 hours

        challenge.is_active = false;
        let amount = challenge.stake_amount;
        
        // Calculate commission using integer math
        let dev_commission = amount
            .checked_mul(COMMISSION_BPS)
            .unwrap()
            .checked_div(BPS_DENOMINATOR)
            .unwrap();
        let treasury_share = amount.checked_sub(dev_commission).unwrap();

        // Transfer from challenge PDA
        **challenge.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.dev.to_account_info().try_borrow_mut_lamports()? += dev_commission;
        **ctx.accounts.treasury.to_account_info().try_borrow_mut_lamports()? += treasury_share;

        // Update treasury accounting
        let treasury = &mut ctx.accounts.treasury;
        treasury.total_funded = treasury.total_funded.checked_add(treasury_share).unwrap();

        Ok(())
    }

    pub fn close_challenge(ctx: Context<CloseChallenge>) -> Result<()> {
        let challenge = &ctx.accounts.challenge;
        
        // Can only close inactive challenges
        require!(!challenge.is_active, ErrorCode::ChallengeStillActive);
        
        // Rent is returned automatically via close = authority constraint
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeTreasury<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Treasury::INIT_SPACE,
        seeds = [b"treasury"],
        bump
    )]
    pub treasury: Account<'info, Treasury>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FundTreasury<'info> {
    #[account(mut, seeds = [b"treasury"], bump = treasury.bump)]
    pub treasury: Account<'info, Treasury>,
    #[account(mut)]
    pub funder: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StartChallenge<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Challenge::INIT_SPACE,
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
    #[account(
        mut, 
        seeds = [b"challenge", authority.key().as_ref()], 
        bump = challenge.bump,
        has_one = authority
    )]
    pub challenge: Account<'info, Challenge>,
    #[account(mut, seeds = [b"treasury"], bump = treasury.bump)]
    pub treasury: Account<'info, Treasury>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Slash<'info> {
    #[account(
        mut, 
        seeds = [b"challenge", challenge.authority.as_ref()], 
        bump = challenge.bump
    )]
    pub challenge: Account<'info, Challenge>,
    #[account(mut, seeds = [b"treasury"], bump = treasury.bump)]
    pub treasury: Account<'info, Treasury>,
    /// CHECK: Validated against hardcoded DEV_WALLET constant
    #[account(mut, address = DEV_WALLET)]
    pub dev: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseChallenge<'info> {
    #[account(
        mut,
        seeds = [b"challenge", authority.key().as_ref()],
        bump = challenge.bump,
        has_one = authority,
        close = authority
    )]
    pub challenge: Account<'info, Challenge>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct Treasury {
    pub authority: Pubkey,
    pub total_funded: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Challenge {
    pub authority: Pubkey,
    pub start_ts: i64,
    pub last_wake_ts: i64,
    pub streak: u8,
    pub is_active: bool,
    pub stake_amount: u64,
    pub alarm_hour: u8,
    pub alarm_minute: u8,
    pub timezone_offset: i32,  // User's timezone offset in seconds from UTC
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Challenge is not active.")]
    ChallengeInactive,
    #[msg("Too early to complete the next day.")]
    TooEarly,
    #[msg("The user hasn't missed the deadline yet.")]
    NotSlashableYet,
    #[msg("Invalid alarm time.")]
    InvalidAlarmTime,
    #[msg("Invalid dev wallet address.")]
    InvalidDevWallet,
    #[msg("Challenge is still active.")]
    ChallengeStillActive,
    #[msg("Current time is outside the alarm window (±1 hour).")]
    OutsideAlarmWindow,
    #[msg("Invalid timezone offset.")]
    InvalidTimezone,
    #[msg("Stake amount is below minimum (0.01 SOL).")]
    StakeTooLow,
    #[msg("Stake amount is above maximum (1 SOL).")]
    StakeTooHigh,
}
