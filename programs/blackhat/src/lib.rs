use anchor_lang::prelude::*;
use solana_program::keccak::hashv;

declare_id!("GmvKmJgjSBC2VJBQLij9ziJ14iwEyxcGLL6FWPv7Ziq8");

#[program]
pub mod blackhat {
    use super::*;

    pub fn setup(ctx: Context<Setup>, bet: u64, commitment: [u8; 32]) -> Result<()> {
        ctx.accounts.game.creator = ctx.accounts.creator.key();
        ctx.accounts.game.player = ctx.accounts.player.key();
        ctx.accounts.game.bet = bet;
        ctx.accounts.game.commitment = commitment;

        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.creator.to_account_info(),
                    to: ctx.accounts.game_authority.to_account_info(),
                },
            ),
            9 * bet,
        )?;

        Ok(())
    }

    pub fn join(ctx: Context<Join>, r: u64) -> Result<()> {
        ctx.accounts.game.user_random = Some(r);

        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.player.to_account_info(),
                    to: ctx.accounts.game_authority.to_account_info(),
                },
            ),
            ctx.accounts.game.bet,
        )?;

        Ok(())
    }

    pub fn submit(ctx: Context<Submit>, score: u64) -> Result<()> {
        // Score can only be set once
        ctx.accounts
            .game
            .score
            .try_into()
            .map_err(|_| err!("score already exists"))
        // let score = ctx.accounts.game.score.try_into();
        // match ctx.accounts.game.score {
        //     Some(_score) => panic!("score already set"),
        //     None => ctx.accounts.game.score = Some(score),
        // }
        // Ok(())
    }

    pub fn settle(ctx: Context<Settle>, max_score: u64, salt: u64, score: u64) -> Result<()> {
        // Reveal commitment (and verify it's legit)
        let buffer: &[&[u8]] = &[&max_score.to_le_bytes(), &salt.to_le_bytes()];
        let hash = hashv(buffer);
        msg!(
            "{} {} {} {}",
            hex::encode(ctx.accounts.game.commitment),
            hex::encode(hash.0),
            max_score,
            salt
        );
        if hash.0 != ctx.accounts.game.commitment {
            panic!("commitment doesn't match")
        }

        // Compute payout (score - secret)
        let payout: u64;
        if score > max_score {
            // Invalid result
            payout = 0;
        } else {
            payout = (100 - score) * ctx.accounts.game.bet / 10;
        }

        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.game_authority.to_account_info(),
                    to: ctx.accounts.player.to_account_info(),
                },
                &[&[
                    b"authority".as_ref(),
                    ctx.accounts.game.key().as_ref(),
                    &[*ctx.bumps.get("game_authority").unwrap()],
                ]],
            ),
            payout,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Setup<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    /// CHECK:
    #[account()]
    pub player: AccountInfo<'info>,

    #[account(
        init,
        seeds = [b"game".as_ref(), player.key().as_ref()],
        bump,
        payer = creator,
        space = 440
    )]
    pub game: Account<'info, Game>,

    /// CHECK:
    #[account(
            mut,
            seeds = [b"authority".as_ref(), game.key().as_ref()],
            bump
        )]
    pub game_authority: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Join<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
            mut,
            seeds = [b"game".as_ref(), player.key().as_ref()],
            bump,
            constraint = game.player == player.key()
        )]
    pub game: Account<'info, Game>,

    /// CHECK:
    #[account(
            mut,
            seeds = [b"authority".as_ref(), game.key().as_ref()],
            bump
        )]
    pub game_authority: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Submit<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
            mut,
            seeds = [b"game".as_ref(), player.key().as_ref()],
            bump,
            constraint = game.player == player.key()
        )]
    pub game: Account<'info, Game>,
}

#[derive(Accounts)]
pub struct Settle<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    /// CHECK:
    #[account(mut)]
    pub player: AccountInfo<'info>,

    #[account(
            mut,
            seeds = [b"game".as_ref(), player.key().as_ref()],
            bump,
            constraint = game.creator == creator.key()
        )]
    pub game: Account<'info, Game>,

    /// CHECK:
    #[account(
            mut,
            seeds = [b"authority".as_ref(), game.key().as_ref()],
            bump
        )]
    pub game_authority: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Debug, PartialEq, Eq, Copy)]
pub struct Game {
    pub creator: Pubkey,
    pub player: Pubkey,
    pub bet: u64,
    pub commitment: [u8; 32],
    pub user_random: Option<u64>,
    pub payout: Option<u64>,
    pub score: Option<u64>,
}
