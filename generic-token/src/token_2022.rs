//! Partial SPL Token declarations to avoid a dependency on the spl-token-2022 crate.

use crate::token::{
    self, is_initialized_account, is_initialized_mint, GenericTokenAccount, GenericTokenMint,
    SPL_TOKEN_ACCOUNT_LENGTH,
};

solana_pubkey::declare_id!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

// `spl_token_program_2022::extension::AccountType::Account` ordinal value
pub const ACCOUNTTYPE_ACCOUNT: u8 = 2;

// Token2022 enforces that TLV data cannot make a Mint or Account that is precisely
// the length of a Multisig, to allow them to be distinguished.
const SPL_TOKEN_MULTISIG_LENGTH: usize = 355;

pub struct Account;
impl GenericTokenAccount for Account {
    fn valid_account_data(account_data: &[u8]) -> bool {
        token::Account::valid_account_data(account_data)
            || (account_data.len() > SPL_TOKEN_ACCOUNT_LENGTH
                && account_data.len() != SPL_TOKEN_MULTISIG_LENGTH
                && ACCOUNTTYPE_ACCOUNT == account_data[SPL_TOKEN_ACCOUNT_LENGTH]
                && is_initialized_account(account_data))
    }
}

// `spl_token_program_2022::extension::AccountType::Mint` ordinal value
const ACCOUNTTYPE_MINT: u8 = 1;

pub struct Mint;
impl GenericTokenMint for Mint {
    // NOTE `account_data.len() > SPL_TOKEN_ACCOUNT_LENGTH` is intentional.
    // We use Account length, not Mint length, because an extended Mint is
    // padded out to Account length so an Account cannot masquerade as a Mint.
    fn valid_account_data(account_data: &[u8]) -> bool {
        token::Mint::valid_account_data(account_data)
            || (account_data.len() > SPL_TOKEN_ACCOUNT_LENGTH
                && account_data.len() != SPL_TOKEN_MULTISIG_LENGTH
                && ACCOUNTTYPE_MINT == account_data[SPL_TOKEN_ACCOUNT_LENGTH]
                && is_initialized_mint(account_data))
    }
}
