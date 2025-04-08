/// Minimum viable SPL Token Account parser to avoid a dependency on the spl-token and spl-token-2022 crates.
/// Users may use `GenericTokenAccount` directly, but this requires them to select the correct implementation
/// based on the account's program id. `generic_token::Account` abstracts over this and requires no knowledge
/// of the different token programs on the part of the caller at all.
///
/// We provide the minimum viable interface to determine balances and ownership. For more advanced usecases,
/// it is recommended to use to full token program crates instead.
use {
    crate::{
        token::{self, GenericTokenAccount},
        token_2022,
    },
    solana_pubkey::Pubkey,
};

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

            (*mint, *owner, *amount)
        } else if *program_id == token_2022::id() {
            token_2022::Account::valid_account_data(account_data).then_some(())?;

            let mint = token_2022::Account::unpack_account_mint_unchecked(account_data);
            let owner = token_2022::Account::unpack_account_owner_unchecked(account_data);
            let amount = token_2022::Account::unpack_account_amount_unchecked(account_data);

            (*mint, *owner, *amount)
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
