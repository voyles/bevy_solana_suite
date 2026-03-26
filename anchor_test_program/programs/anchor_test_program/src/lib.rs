use anchor_lang::prelude::*;

declare_id!("3k9JM1TwoXjxuiD6VwqBsCxPFKHcLEqF1uMHzqnV9FUm");

#[program]
pub mod anchor_test_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        msg!("Player: {:?}", ctx.accounts.player.key());
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}