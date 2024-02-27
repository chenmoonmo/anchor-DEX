use anchor_lang::prelude::*;

declare_id!("BMP3XsejH17RjqpBTRLN364ZBSWiGBJKQiE73ikUEEBJ");

#[program]
pub mod anchor_dex {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
