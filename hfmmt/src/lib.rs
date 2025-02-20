use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};


declare_id!("GsvEYrds1qtwamYbHJpUTx3jeEV6XrCSdxDy8UCf6y9H");

#[program]
pub mod hfmm_token {
    use super::*;
    
    /// Initialize the protocol's configuration.
    pub fn initialize(ctx: Context<Initialize>, config_params: ConfigParams) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.reward_rate = config_params.reward_rate;
        config.volatility_threshold = config_params.volatility_threshold;
        Ok(())
    }
    
    /// Market makers stake HFMMT tokens.
    pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64) -> Result<()> {
        token::transfer(ctx.accounts.into_transfer_to_vault_context(), amount)?;
        let staker = &mut ctx.accounts.staker;
        staker.staked_amount = staker.staked_amount.checked_add(amount).unwrap();
        // Update both last trade and stake timestamps.
        let current_time = Clock::get()?.unix_timestamp;
        staker.last_trade_time = current_time;
        staker.last_stake_time = current_time;
        Ok(())
    }
    
    /// Update market maker performance metrics.
    pub fn update_performance(
        ctx: Context<UpdatePerformance>,
        execution_volume: u64,
        spread_efficiency: u64,
        order_flow: u64,
    ) -> Result<()> {
        let staker = &mut ctx.accounts.staker;
        staker.execution_volume = staker.execution_volume.checked_add(execution_volume).unwrap();
        staker.spread_efficiency = staker.spread_efficiency.checked_add(spread_efficiency).unwrap();
        staker.order_flow = staker.order_flow.checked_add(order_flow).unwrap();
        staker.last_trade_time = Clock::get()?.unix_timestamp;
        Ok(())
    }
    
    /// Calculate and distribute rewards to market makers.
    /// If auto-compound is enabled, rewards are added to the staked amount.
    pub fn distribute_rewards(ctx: Context<DistributeRewards>) -> Result<()> {
        let staker = &mut ctx.accounts.staker;
        let config = &ctx.accounts.config;
        // Example weighted score: 2×execution + 3×spread efficiency + 1×order flow.
        let score = staker.execution_volume.checked_mul(2).unwrap()
            .checked_add(staker.spread_efficiency.checked_mul(3).unwrap()).unwrap()
            .checked_add(staker.order_flow).unwrap();
        let reward_amount = score.checked_div(config.reward_rate).unwrap();
        if staker.auto_compound {
            staker.staked_amount = staker.staked_amount.checked_add(reward_amount).unwrap();
        } else {
            token::transfer(ctx.accounts.into_transfer_to_staker_context(), reward_amount)?;
        }
        Ok(())
    }
    
    /// Liquidity providers add tokens to the pool.
    pub fn provide_liquidity(ctx: Context<ProvideLiquidity>, amount: u64) -> Result<()> {
        let liquidity_provider = &mut ctx.accounts.liquidity_provider;
        liquidity_provider.liquidity = liquidity_provider.liquidity.checked_add(amount).unwrap();
        liquidity_provider.last_deposit_time = Clock::get()?.unix_timestamp;
        token::transfer(ctx.accounts.into_transfer_to_pool_context(), amount)?;
        Ok(())
    }
    
    /// Liquidity providers withdraw tokens from the pool.
    pub fn withdraw_liquidity(ctx: Context<WithdrawLiquidity>, amount: u64) -> Result<()> {
        let liquidity_provider = &mut ctx.accounts.liquidity_provider;
        require!(liquidity_provider.liquidity >= amount, CustomError::InsufficientFunds);
        liquidity_provider.liquidity = liquidity_provider.liquidity.checked_sub(amount).unwrap();
        token::transfer(ctx.accounts.into_transfer_to_user_context(), amount)?;
        Ok(())
    }
    
    /// Request priority execution for market makers.
    pub fn request_priority_execution(ctx: Context<RequestPriority>) -> Result<()> {
        let staker = &mut ctx.accounts.staker;
        require!(staker.staked_amount >= 1000, CustomError::NotEnoughStake);
        staker.has_priority_access = true;
        Ok(())
    }
    
    /// Governance voting for protocol optimizations.
    pub fn vote(ctx: Context<Vote>, proposal_id: u64, vote_weight: u64) -> Result<()> {
        let voter = &mut ctx.accounts.voter;
        let proposal = &mut ctx.accounts.proposal;
        require!(voter.token_balance >= vote_weight, CustomError::NotEnoughTokens);
        proposal.votes = proposal.votes.checked_add(vote_weight).unwrap();
        Ok(())
    }
    
    ///  Dynamic Fee Rebates for Market Makers.
    pub fn claim_fee_rebate(ctx: Context<ClaimFeeRebate>) -> Result<()> {
        let staker_execution_volume = ctx.accounts.staker.execution_volume;
        let fee_total_fees = ctx.accounts.fee_rebate_pool.total_fees;
        let fee_total_exec = ctx.accounts.fee_rebate_pool.total_execution_volume;
        let rebate_amount = fee_total_fees
            .checked_mul(staker_execution_volume)
            .unwrap()
            .checked_div(fee_total_exec)
            .unwrap();
        require!(rebate_amount > 0, CustomError::NoRebateAvailable);
        token::transfer(ctx.accounts.into_transfer_rebate_context(), rebate_amount)?;
        let fee_pool = &mut ctx.accounts.fee_rebate_pool;
        fee_pool.total_fees = fee_pool.total_fees.checked_sub(rebate_amount).unwrap();
        Ok(())
    }
    
    /// Time-Based Liquidity Rewards.
    pub fn lock_liquidity(ctx: Context<LockLiquidity>, lock_duration: u64) -> Result<()> {
        let provider = &mut ctx.accounts.liquidity_provider;
        provider.lock_duration = lock_duration;
        // Set reward multiplier: 1-month = 1x, 3-month = 1.5x, 6-month = 2x.
        match lock_duration {
            3 => provider.reward_multiplier = 150,
            6 => provider.reward_multiplier = 200,
            _ => provider.reward_multiplier = 100,
        }
        Ok(())
    }
    
    ///  Auto-Compounding Staking.
    pub fn enable_auto_compound(ctx: Context<EnableAutoCompound>) -> Result<()> {
        let staker = &mut ctx.accounts.staker;
        staker.auto_compound = true;
        Ok(())
    }
    
    ///  Flash Loan Resistance for Liquidity Providers.
    pub fn claim_liquidity_rewards(ctx: Context<ClaimLiquidityRewards>) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let last_deposit_time = ctx.accounts.liquidity_provider.last_deposit_time;
        require!(current_time > last_deposit_time + 604800, CustomError::FlashLoanDetected);
        let reward = ctx.accounts.liquidity_provider.reward_balance;
        require!(reward > 0, CustomError::NoRewardAvailable);
        token::transfer(ctx.accounts.into_transfer_rewards_context(), reward)?;
        let liquidity_provider = &mut ctx.accounts.liquidity_provider;
        liquidity_provider.reward_balance = 0;
        Ok(())
    }
    
    ///  Staking Slashing for Inactivity.
    pub fn enforce_activity_slashing(ctx: Context<EnforceSlashing>) -> Result<()> {
        let staker = &mut ctx.accounts.staker;
        let current_time = Clock::get()?.unix_timestamp;
        if current_time > staker.last_trade_time + 2592000 {
            let slash_amount = staker.staked_amount / 10;
            staker.staked_amount = staker.staked_amount.checked_sub(slash_amount).unwrap();
            token::transfer(ctx.accounts.into_transfer_slash_context(), slash_amount)?;
        }
        Ok(())
    }
    
    /// --- Additional Enhancements ---
    
    /// Enforce a minimum trade volume to help prevent Sybil attacks.
    pub fn enforce_min_trade_volume(ctx: Context<EnforceTradeVolume>) -> Result<()> {
        let staker = &ctx.accounts.staker;
        require!(staker.execution_volume >= 10_000, CustomError::TradeVolumeTooLow);
        Ok(())
    }
    
    /// Enforce a cooldown period after staking to prevent front-running.
    pub fn enforce_stake_cooldown(ctx: Context<EnforceStakeCooldown>) -> Result<()> {
        let staker = &ctx.accounts.staker;
        let current_time = Clock::get()?.unix_timestamp;
        require!(current_time > staker.last_stake_time + 60, CustomError::CooldownActive);
        Ok(())
    }
    
    /// Adjust rewards dynamically based on market volatility.
    pub fn adjust_rewards_by_volatility(ctx: Context<AdjustRewards>, volatility_index: u64) -> Result<()> {
        let staker = &mut ctx.accounts.staker;
        if volatility_index > 50 {
            staker.reward_multiplier = 2;
        } else {
            staker.reward_multiplier = 1;
        }
        Ok(())
    }
    
    /// Claim gas fee rebates for high-frequency traders.
    pub fn claim_gas_fee_rebate(ctx: Context<ClaimGasRebate>) -> Result<()> {
        let staker = &ctx.accounts.staker;
        require!(staker.execution_volume > 100_000, CustomError::NotEligibleForGasRebate);
        let rebate_amount = staker.execution_volume / 1000;
        token::transfer(ctx.accounts.into_transfer_gas_rebate_context(), rebate_amount)?;
        // Optionally, you might want to update the GasFeePool's funds here.
        Ok(())
    }
    
    /// Claim an insurance payout from the liquidity insurance pool when a loss is detected.
    pub fn claim_insurance_payout(ctx: Context<ClaimInsurance>) -> Result<()> {
        let provider = &ctx.accounts.liquidity_provider;
        require!(provider.loss_occurred, CustomError::NoLossDetected);
        let insurance_payout = provider.liquidity / 10;
        token::transfer(ctx.accounts.into_transfer_insurance_context(), insurance_payout)?;
        Ok(())
    }
}

//
// Contexts for Existing Instructions
//

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + Config::LEN)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub staker: Account<'info, MarketMaker>,
    #[account(mut)]
    pub staker_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> StakeTokens<'info> {
    fn into_transfer_to_vault_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.staker_token_account.to_account_info(),
                to: self.vault.to_account_info(),
                authority: self.staker.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct UpdatePerformance<'info> {
    #[account(mut)]
    pub staker: Account<'info, MarketMaker>,
}

#[derive(Accounts)]
pub struct DistributeRewards<'info> {
    #[account(mut)]
    pub staker: Account<'info, MarketMaker>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub staker_token_account: Account<'info, TokenAccount>,
    pub config: Account<'info, Config>,
    pub token_program: Program<'info, Token>,
}

impl<'info> DistributeRewards<'info> {
    fn into_transfer_to_staker_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.vault.to_account_info(),
                to: self.staker_token_account.to_account_info(),
                authority: self.staker.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct ProvideLiquidity<'info> {
    #[account(mut)]
    pub liquidity_provider: Account<'info, LiquidityProvider>,
    #[account(mut)]
    pub provider_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> ProvideLiquidity<'info> {
    fn into_transfer_to_pool_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.provider_token_account.to_account_info(),
                to: self.pool.to_account_info(),
                authority: self.liquidity_provider.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct WithdrawLiquidity<'info> {
    #[account(mut)]
    pub liquidity_provider: Account<'info, LiquidityProvider>,
    #[account(mut)]
    pub pool: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> WithdrawLiquidity<'info> {
    fn into_transfer_to_user_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.pool.to_account_info(),
                to: self.user_token_account.to_account_info(),
                authority: self.liquidity_provider.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct RequestPriority<'info> {
    #[account(mut)]
    pub staker: Account<'info, MarketMaker>,
}

#[derive(Accounts)]
pub struct Vote<'info> {
    #[account(mut)]
    pub voter: Account<'info, Voter>,
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
}

//
// Contexts for New Enhancements
//

#[derive(Accounts)]
pub struct ClaimFeeRebate<'info> {
    #[account(mut)]
    pub staker: Account<'info, MarketMaker>,
    #[account(mut)]
    pub fee_rebate_pool: Account<'info, FeeRebatePool>,
    #[account(mut)]
    pub staker_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> ClaimFeeRebate<'info> {
    fn into_transfer_rebate_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.fee_rebate_pool.to_account_info(),
                to: self.staker_token_account.to_account_info(),
                authority: self.staker.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct LockLiquidity<'info> {
    #[account(mut)]
    pub liquidity_provider: Account<'info, LiquidityProvider>,
}

#[derive(Accounts)]
pub struct EnableAutoCompound<'info> {
    #[account(mut)]
    pub staker: Account<'info, MarketMaker>,
}

#[derive(Accounts)]
pub struct ClaimLiquidityRewards<'info> {
    #[account(mut)]
    pub liquidity_provider: Account<'info, LiquidityProvider>,
    #[account(mut)]
    pub pool: Account<'info, TokenAccount>,
    #[account(mut)]
    pub provider_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> ClaimLiquidityRewards<'info> {
    fn into_transfer_rewards_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.pool.to_account_info(),
                to: self.provider_token_account.to_account_info(),
                authority: self.liquidity_provider.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct EnforceSlashing<'info> {
    #[account(mut)]
    pub staker: Account<'info, MarketMaker>,
    #[account(mut)]
    pub treasury: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> EnforceSlashing<'info> {
    fn into_transfer_slash_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.treasury.to_account_info(),
                to: self.treasury.to_account_info(),
                authority: self.staker.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct EnforceTradeVolume<'info> {
    #[account(mut)]
    pub staker: Account<'info, MarketMaker>,
}

#[derive(Accounts)]
pub struct EnforceStakeCooldown<'info> {
    #[account(mut)]
    pub staker: Account<'info, MarketMaker>,
}

#[derive(Accounts)]
pub struct AdjustRewards<'info> {
    #[account(mut)]
    pub staker: Account<'info, MarketMaker>,
}

#[derive(Accounts)]
pub struct ClaimGasRebate<'info> {
    #[account(mut)]
    pub staker: Account<'info, MarketMaker>,
    #[account(mut)]
    pub staker_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub gas_fee_pool: Account<'info, GasFeePool>,
    pub token_program: Program<'info, Token>,
}

impl<'info> ClaimGasRebate<'info> {
    fn into_transfer_gas_rebate_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.gas_fee_pool.to_account_info(),
                to: self.staker_token_account.to_account_info(),
                authority: self.staker.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct ClaimInsurance<'info> {
    #[account(mut)]
    pub liquidity_provider: Account<'info, LiquidityProvider>,
    #[account(mut)]
    pub insurance_pool: Account<'info, InsurancePool>,
    #[account(mut)]
    pub provider_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> ClaimInsurance<'info> {
    fn into_transfer_insurance_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.insurance_pool.to_account_info(),
                to: self.provider_token_account.to_account_info(),
                authority: self.liquidity_provider.to_account_info(),
            },
        )
    }
}

//
// Data Structures & Account Definitions
//

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ConfigParams {
    pub reward_rate: u64,
    pub volatility_threshold: u64,
}

#[account]
pub struct Config {
    pub reward_rate: u64,
    pub volatility_threshold: u64,
}

impl Config {
    const LEN: usize = 8 + 8;
}

#[account]
pub struct MarketMaker {
    pub staked_amount: u64,
    pub execution_volume: u64,
    pub spread_efficiency: u64,
    pub order_flow: u64,
    pub has_priority_access: bool,
    pub auto_compound: bool,
    pub last_trade_time: i64,
    pub last_stake_time: i64,    // For cooldown enforcement.
    pub reward_multiplier: u64,  // For dynamic risk-based rewards.
}

#[account]
pub struct LiquidityProvider {
    pub liquidity: u64,
    pub lock_duration: u64,
    pub reward_multiplier: u64,
    pub last_deposit_time: i64,
    pub reward_balance: u64,
    pub loss_occurred: bool,     // For insurance payout.
}

#[account]
pub struct FeeRebatePool {
    pub total_fees: u64,
    pub total_execution_volume: u64,
}

#[account]
pub struct GasFeePool {
    pub total_funds: u64,
}

#[account]
pub struct InsurancePool {
    pub total_funds: u64,
}

#[account]
pub struct Proposal {
    pub proposal_id: u64,
    pub votes: u64,
}

#[account]
pub struct Voter {
    pub token_balance: u64,
}

//
// Custom Errors for Better Debugging
//

#[error_code]
pub enum CustomError {
    #[msg("Insufficient funds to withdraw.")]
    InsufficientFunds,
    #[msg("Not enough stake to receive priority execution.")]
    NotEnoughStake,
    #[msg("You do not have enough HFMMT tokens to vote.")]
    NotEnoughTokens,
    #[msg("No rebate available for claiming.")]
    NoRebateAvailable,
    #[msg("Flash loan activity detected. Rewards are locked.")]
    FlashLoanDetected,
    #[msg("No liquidity reward available.")]
    NoRewardAvailable,
    #[msg("Trade volume is too low to earn rewards.")]
    TradeVolumeTooLow,
    #[msg("Stake cooldown period is active.")]
    CooldownActive,
    #[msg("Not eligible for gas fee rebate.")]
    NotEligibleForGasRebate,
    #[msg("No loss detected; insurance payout is not available.")]
    NoLossDetected,
}
