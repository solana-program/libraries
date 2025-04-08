/// Partial SPL Token declarations to avoid a dependency on the spl-token-2022 crate.
use crate::token::{self, GenericTokenAccount, GenericTokenMint};

solana_pubkey::declare_id!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

// `spl_token_program_2022::extension::AccountType::Account` ordinal value
const ACCOUNTTYPE_ACCOUNT: u8 = 2;

pub struct Account;
impl GenericTokenAccount for Account {
    fn valid_account_data(account_data: &[u8]) -> bool {
        token::Account::valid_account_data(account_data)
            || ACCOUNTTYPE_ACCOUNT
                == *account_data
                    .get(token::Account::get_packed_len())
                    .unwrap_or(&0)
    }
}

// `spl_token_program_2022::extension::AccountType::Mint` ordinal value
const ACCOUNTTYPE_MINT: u8 = 1;

pub struct Mint;
impl GenericTokenMint for Mint {
    fn valid_account_data(account_data: &[u8]) -> bool {
        token::Mint::valid_account_data(account_data)
            || ACCOUNTTYPE_MINT
                == *account_data
                    .get(token::Mint::get_packed_len())
                    .unwrap_or(&0)
    }
}
