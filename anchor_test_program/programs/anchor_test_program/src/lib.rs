use anchor_lang::prelude::*;

declare_id!("3k9JM1TwoXjxuiD6VwqBsCxPFKHcLEqF1uMHzqnV9FUm");

#[program]
pub mod anchor_test_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
