use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod escrow {
    use super::*;

    /// Create an escrow by locking SOL in a PDA account.
    /// The buyer specifies the seller address and amount.
    pub fn create_escrow(
        ctx: Context<CreateEscrow>,
        seller: Pubkey,
        amount: u64,
        escrow_id: u64,
    ) -> Result<()> {
        require!(amount > 0, EscrowError::InvalidAmount);

        let escrow = &mut ctx.accounts.escrow_account;
        escrow.buyer = ctx.accounts.buyer.key();
        escrow.seller = seller;
        escrow.amount = amount;
        escrow.status = EscrowStatus::Pending;
        escrow.escrow_id = escrow_id;
        escrow.bump = ctx.bumps.escrow_account;

        // Transfer SOL from buyer to PDA
        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.escrow_account.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(cpi_ctx, amount)?;

        msg!("Escrow created: {} lamports locked for seller {}", amount, seller);
        Ok(())
    }

    /// Release funds to the seller. Only the buyer can call this.
    pub fn release_funds(ctx: Context<ReleaseFunds>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;

        require!(
            ctx.accounts.buyer.key() == escrow.buyer,
            EscrowError::NotBuyer
        );
        require!(
            escrow.status == EscrowStatus::Pending,
            EscrowError::NotPending
        );

        let amount = escrow.amount;
        escrow.status = EscrowStatus::Completed;

        // Transfer SOL from PDA to seller using PDA signer seeds
        let buyer_key = escrow.buyer;
        let escrow_id_bytes = escrow.escrow_id.to_le_bytes();
        let bump = escrow.bump;
        let seeds: &[&[u8]] = &[
            b"escrow",
            buyer_key.as_ref(),
            &escrow_id_bytes,
            &[bump],
        ];
        let signer = &[seeds];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.escrow_account.to_account_info(),
                to: ctx.accounts.seller.to_account_info(),
            },
            signer,
        );
        anchor_lang::system_program::transfer(cpi_ctx, amount)?;

        msg!("Funds released: {} lamports sent to seller", amount);
        Ok(())
    }

    /// Cancel the escrow and refund the buyer. Only the buyer can call this.
    pub fn cancel_escrow(ctx: Context<CancelEscrow>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;

        require!(
            ctx.accounts.buyer.key() == escrow.buyer,
            EscrowError::NotBuyer
        );
        require!(
            escrow.status == EscrowStatus::Pending,
            EscrowError::NotPending
        );

        let amount = escrow.amount;
        escrow.status = EscrowStatus::Cancelled;

        // Refund SOL from PDA back to buyer
        let buyer_key = escrow.buyer;
        let escrow_id_bytes = escrow.escrow_id.to_le_bytes();
        let bump = escrow.bump;
        let seeds: &[&[u8]] = &[
            b"escrow",
            buyer_key.as_ref(),
            &escrow_id_bytes,
            &[bump],
        ];
        let signer = &[seeds];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.escrow_account.to_account_info(),
                to: ctx.accounts.buyer.to_account_info(),
            },
            signer,
        );
        anchor_lang::system_program::transfer(cpi_ctx, amount)?;

        msg!("Escrow cancelled: {} lamports refunded to buyer", amount);
        Ok(())
    }
}

/// Account context for creating an escrow
#[derive(Accounts)]
#[instruction(seller: Pubkey, amount: u64, escrow_id: u64)]
pub struct CreateEscrow<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        init,
        payer = buyer,
        space = EscrowAccount::SIZE,
        seeds = [b"escrow", buyer.key().as_ref(), &escrow_id.to_le_bytes()],
        bump
    )]
    pub escrow_account: Account<'info, EscrowAccount>,

    pub system_program: Program<'info, System>,
}

/// Account context for releasing funds to seller
#[derive(Accounts)]
pub struct ReleaseFunds<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: This is the seller account, validated against escrow data
    #[account(mut)]
    pub seller: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"escrow", buyer.key().as_ref(), &escrow_account.escrow_id.to_le_bytes()],
        bump = escrow_account.bump,
        constraint = escrow_account.buyer == buyer.key() @ EscrowError::NotBuyer
    )]
    pub escrow_account: Account<'info, EscrowAccount>,

    pub system_program: Program<'info, System>,
}

/// Account context for cancelling an escrow
#[derive(Accounts)]
pub struct CancelEscrow<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"escrow", buyer.key().as_ref(), &escrow_account.escrow_id.to_le_bytes()],
        bump = escrow_account.bump,
        constraint = escrow_account.buyer == buyer.key() @ EscrowError::NotBuyer
    )]
    pub escrow_account: Account<'info, EscrowAccount>,

    pub system_program: Program<'info, System>,
}

/// On-chain escrow account data
#[account]
pub struct EscrowAccount {
    pub buyer: Pubkey,       // 32 bytes
    pub seller: Pubkey,      // 32 bytes
    pub amount: u64,         // 8 bytes
    pub status: EscrowStatus, // 1 byte (enum)
    pub escrow_id: u64,      // 8 bytes
    pub bump: u8,            // 1 byte
}

impl EscrowAccount {
    // 8 (discriminator) + 32 + 32 + 8 + 1 + 8 + 1
    pub const SIZE: usize = 8 + 32 + 32 + 8 + 1 + 8 + 1;
}

/// Escrow lifecycle status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum EscrowStatus {
    Pending,
    Completed,
    Cancelled,
}

/// Custom program errors
#[error_code]
pub enum EscrowError {
    #[msg("Only the buyer can perform this action")]
    NotBuyer,
    #[msg("Escrow is not in pending status")]
    NotPending,
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
}
