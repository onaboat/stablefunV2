use crate::state::*;
// use crate::errors::*;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ burn, Burn, close_account, CloseAccount, Mint, Token, TokenAccount},
    token_2022::{
        transfer_checked, TransferChecked,
        Token2022,
    },
};
use switchboard_on_demand::on_demand::accounts::pull_feed::PullFeedAccountData;


pub fn burn_tokens(ctx: Context<BurnTokens>, amount: u64) -> Result<()> {
// // Check if the amount is valid
// require!(amount > 0, CustomError::InvalidQuantity);
msg!("burn_tokens called with amount: {}", amount);


    msg!("bond_mint key: {}", ctx.accounts.bond_mint.key());
    msg!("coin_account key: {}", ctx.accounts.coin_account.key());
    msg!("treasury key: {}", ctx.accounts.treasury.key());
    msg!("treasury_bond_ata key: {}", ctx.accounts.treasury_bond_ata.key());
    msg!("coin_account_bond_ata key: {}", ctx.accounts.coin_account_bond_ata.key());



let cpi_accounts = Burn {
    mint: ctx.accounts.mint.to_account_info(), // Stablecoin mint
    from: ctx.accounts.user_stablecoin_ata.to_account_info(), // User's ATA for stablecoin
    authority: ctx.accounts.payer.to_account_info(), // User is the authority
};
let cpi_context = CpiContext::new(
    ctx.accounts.token_program.to_account_info(), // Use the appropriate Token program
    cpi_accounts,
);


burn(cpi_context, amount)?;


let coin_account_bump = ctx.accounts.coin_account.bump;
let mint_key = ctx.accounts.mint.key();
let seeds = &[
    COIN_ACCOUNT_SEED.as_ref(),
    mint_key.as_ref(),
    &[coin_account_bump]
];
let signer = &[&seeds[..]];

let bond_decimals = 6; 

let cpi_accounts = TransferChecked {
    from: ctx.accounts.coin_account_bond_ata.to_account_info(), // CoinAccount's ATA for bond tokens
    to: ctx.accounts.treasury_bond_ata.to_account_info(), // Treasury's ATA for bond tokens
    authority: ctx.accounts.coin_account.to_account_info(), // CoinAccount is the authority
    mint: ctx.accounts.bond_mint.to_account_info(), // Bond token mint
};

let cpi_context = CpiContext::new_with_signer(
    ctx.accounts.token_program_2022.to_account_info(), // Token 2022 program
    cpi_accounts,
    signer,
);

transfer_checked(cpi_context, amount, bond_decimals)?;


let feed_account = ctx.accounts.feed.data.borrow();
let feed = PullFeedAccountData::parse(feed_account).unwrap();
let price_decimal = feed.value().unwrap();
let price_f64 = price_decimal.to_string().parse::<f64>().expect("Failed to convert Decimal to f64");

//calculate SOL Amount 
let sol_decimals = 9; // SOL has 9 decimals
let sol_amount_f64 = amount as f64 * price_f64 * 10_f64.powi(sol_decimals as i32);
let sol_amount = (sol_amount_f64 * 10_f64.powi(sol_decimals as i32)) as u64;

// transfer SOL from CoinAccount to user
let cpi_accounts = anchor_lang::system_program::Transfer {
    from: ctx.accounts.coin_account.to_account_info(), // CoinAccount PDA
    to: ctx.accounts.payer.to_account_info(), // User's account
};

let cpi_context = CpiContext::new_with_signer(
    ctx.accounts.system_program.to_account_info(),
    cpi_accounts,
    signer
);

anchor_lang::system_program::transfer(cpi_context, sol_amount)?;

// close coin account bond ata = 0 
// let coin_account_bond_ata = &ctx.accounts.coin_account_bond_ata;
// if coin_account_bond_ata.amount == 0 {
//     let cpi_accounts = CloseAccount {
//         account: coin_account_bond_ata.to_account_info(), // The ATA to close
//         destination: ctx.accounts.payer.to_account_info(), // Where to send the lamports from rent
//         authority: ctx.accounts.coin_account.to_account_info(), // Authority to close the account
//     };
//     let cpi_context = CpiContext::new_with_signer(
//         ctx.accounts.token_program_2022.to_account_info(), // Token 2022 program
//         cpi_accounts,
//         signer,
//     );
//     close_account(cpi_context)?;
// }

Ok(())
}
#[derive(Accounts)]
pub struct BurnTokens<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: manually deserialize and verify the account data
    pub feed: AccountInfo<'info>,
    #[account(mut)]
    pub user_stablecoin_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [COIN_ACCOUNT_SEED, mint.key().as_ref()],
        bump,
    )]
    pub coin_account: Account<'info, CoinAccount>,
    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump,
    )]
    pub treasury: Account<'info, Treasury>,
    /// CHECK: This is safe because we check its mint
    #[account(mut)]
    pub bond_mint: AccountInfo<'info>,
    /// CHECK: treasury bond ata
    #[account(mut)]
    pub treasury_bond_ata: UncheckedAccount<'info>,
    /// CHECK: coin account bond ata
    #[account(mut)]
    pub coin_account_bond_ata: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub token_program_2022: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}