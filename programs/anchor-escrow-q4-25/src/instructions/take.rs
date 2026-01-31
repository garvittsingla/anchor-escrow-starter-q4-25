#![allow(unused_imports)]

use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken, token::Token, token_interface::{
        CloseAccount, Mint, TokenAccount, TokenInterface, TransferChecked, close_account, transfer_checked
    }
};

use crate::Escrow;
//we need the takers ata of account mint b, maker ata b, vault account, escrow account, maker account, token program
#[derive(Accounts)]
pub struct Take<'info> {
    //  TODO: Implement Take Accounts
    #[account(mut)]
    pub taker: Signer<'info>,
    pub maker: SystemAccount<'info>,
    pub token_program : Interface<'info, TokenInterface>,
    pub mint_a:InterfaceAccount<'info, Mint>,
    pub mint_b:InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_b: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_ata_b: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_ata_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        close = taker,
        has_one = mint_a,
        has_one = maker,
        seeds = [b"escrow", maker.key().as_ref(), &escrow.seed.to_le_bytes()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,

}

impl<'info> Take<'info> {
    //  TODO: Implement Take Instruction
    //  Includes Deposit, Withdraw and Close Vault
    pub fn deposit(&mut self) -> Result<()>{
            // Transfer mint_b from taker to maker
            let transfer_accounts= TransferChecked{
                from:self.taker_ata_b.to_account_info(),
                mint:self.mint_b.to_account_info(),
                to:self.maker_ata_b.to_account_info(),
                authority:self.taker.to_account_info(),
            };
            let tranfer_cpi_ctx = CpiContext::new(
                self.token_program.to_account_info(),
                transfer_accounts
            );
            transfer_checked(tranfer_cpi_ctx, self.escrow.receive, self.mint_b.decimals)?;

        Ok(())
    }
    pub fn withdraw_and_close_vault(&mut self) -> Result<()>{
        //tranfer token a from maker vault to taker ata a, close the vault
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &self.escrow.seed.to_le_bytes(),
            &[self.escrow.bump],
        ]];
         let transfer_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
         let tranfer_cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_accounts,
            signer_seeds,
        );
        transfer_checked(tranfer_cpi_ctx, self.vault.amount, self.mint_a.decimals)?;
        let close_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.taker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
        let close_cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            close_accounts,
            signer_seeds,
        );
        close_account(close_cpi_ctx)?;
        Ok(())
    }


}
