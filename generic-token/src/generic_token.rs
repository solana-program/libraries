//! Minimum viable SPL Token parsers to avoid a dependency on the spl-token and spl-token-2022 crates.
//! Users may use the generic traits directly, but this requires them to select the correct implementation
//! based on the account's program id. `generic_token::Account` and `generic_token::Mint` abstract over
//! this and require no knowledge of the different token programs on the part of the caller at all.
//!
//! We provide the minimum viable interface to determine balances and ownership. For more advanced use-cases,
//! it is recommended to use to full token program crates instead.

use {
    crate::{
        token::{self, GenericTokenAccount, GenericTokenMint},
        token_2022,
    },
    solana_pubkey::Pubkey,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Account {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
}

impl Account {
    pub fn unpack(account_data: &[u8], program_id: &Pubkey) -> Option<Self> {
        let (mint, owner, amount) = if *program_id == token::id() {
            token::Account::valid_account_data(account_data).then_some(())?;

            let mint = token::Account::unpack_account_mint_unchecked(account_data);
            let owner = token::Account::unpack_account_owner_unchecked(account_data);
            let amount = token::Account::unpack_account_amount_unchecked(account_data);

            (*mint, *owner, amount)
        } else if *program_id == token_2022::id() {
            token_2022::Account::valid_account_data(account_data).then_some(())?;

            let mint = token_2022::Account::unpack_account_mint_unchecked(account_data);
            let owner = token_2022::Account::unpack_account_owner_unchecked(account_data);
            let amount = token_2022::Account::unpack_account_amount_unchecked(account_data);

            (*mint, *owner, amount)
        } else {
            return None;
        };

        Some(Self {
            mint,
            owner,
            amount,
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Mint {
    pub supply: u64,
    pub decimals: u8,
}

impl Mint {
    pub fn unpack(account_data: &[u8], program_id: &Pubkey) -> Option<Self> {
        let (supply, decimals) = if *program_id == token::id() {
            token::Mint::valid_account_data(account_data).then_some(())?;

            let supply = token::Mint::unpack_mint_supply_unchecked(account_data);
            let decimals = token::Mint::unpack_mint_decimals_unchecked(account_data);

            (supply, decimals)
        } else if *program_id == token_2022::id() {
            token_2022::Mint::valid_account_data(account_data).then_some(())?;

            let supply = token_2022::Mint::unpack_mint_supply_unchecked(account_data);
            let decimals = token_2022::Mint::unpack_mint_decimals_unchecked(account_data);

            (supply, decimals)
        } else {
            return None;
        };

        Some(Self { supply, decimals })
    }
}
