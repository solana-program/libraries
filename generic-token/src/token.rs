/// Partial SPL Token declarations to avoid a dependency on the spl-token crate.
use {
    solana_pubkey::{Pubkey, PUBKEY_BYTES},
    std::mem,
};

solana_pubkey::declare_id!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

pub mod program_v3_4_0 {
    solana_pubkey::declare_id!("NToK4t5AQzxPNpUA84DkxgfXaVDbDQQjpHKCqsbY46B");
}

/*
    spl_token::state::Account {
        mint: Pubkey,
        owner: Pubkey,
        amount: u64,
        delegate: COption<Pubkey>,
        state: AccountState,
        is_native: COption<u64>,
        delegated_amount: u64,
        close_authority: COption<Pubkey>,
    }
*/
const SPL_TOKEN_ACCOUNT_MINT_OFFSET: usize = 0;
const SPL_TOKEN_ACCOUNT_OWNER_OFFSET: usize = 32;
const SPL_TOKEN_ACCOUNT_AMOUNT_OFFSET: usize = 64;
const SPL_TOKEN_ACCOUNT_LENGTH: usize = 165;

macro_rules! define_checked_getter {
    ($checked_fn:ident, $unchecked_fn:ident, $typ:ty) => {
        fn $checked_fn(account_data: &[u8]) -> Option<&$typ> {
            if Self::valid_account_data(account_data) {
                Some(Self::$unchecked_fn(account_data))
            } else {
                None
            }
        }
    };
}

// NOTE if future token programs have new offsets, these helpers could be added to the trait
// for now, no impls need to override any getters, so exposing arbitrary-offset unpacking is useless
fn unpack_pubkey_unchecked(account_data: &[u8], offset: usize) -> &Pubkey {
    bytemuck::from_bytes(&account_data[offset..offset.wrapping_add(PUBKEY_BYTES)])
}

fn unpack_u64_unchecked(account_data: &[u8], offset: usize) -> &u64 {
    bytemuck::from_bytes(&account_data[offset..offset.wrapping_add(mem::size_of::<u64>())])
}

fn unpack_u8_unchecked(account_data: &[u8], offset: usize) -> &u8 {
    bytemuck::from_bytes(&account_data[offset..offset.wrapping_add(mem::size_of::<u8>())])
}

// Trait for retrieving mint address, owner, and amount from any token account-like buffer.
// A token program that copies the spl_token layout need only impl `valid_account_data()`.
pub trait GenericTokenAccount {
    fn valid_account_data(account_data: &[u8]) -> bool;

    define_checked_getter!(unpack_account_mint, unpack_account_mint_unchecked, Pubkey);
    define_checked_getter!(unpack_account_owner, unpack_account_owner_unchecked, Pubkey);
    define_checked_getter!(unpack_account_amount, unpack_account_amount_unchecked, u64);

    // Call after account length has already been verified
    fn unpack_account_mint_unchecked(account_data: &[u8]) -> &Pubkey {
        unpack_pubkey_unchecked(account_data, SPL_TOKEN_ACCOUNT_MINT_OFFSET)
    }

    // Call after account length has already been verified
    fn unpack_account_owner_unchecked(account_data: &[u8]) -> &Pubkey {
        unpack_pubkey_unchecked(account_data, SPL_TOKEN_ACCOUNT_OWNER_OFFSET)
    }

    // Call after account length has already been verified
    fn unpack_account_amount_unchecked(account_data: &[u8]) -> &u64 {
        unpack_u64_unchecked(account_data, SPL_TOKEN_ACCOUNT_AMOUNT_OFFSET)
    }
}

pub struct Account;
impl Account {
    pub fn get_packed_len() -> usize {
        SPL_TOKEN_ACCOUNT_LENGTH
    }
}

impl GenericTokenAccount for Account {
    fn valid_account_data(account_data: &[u8]) -> bool {
        account_data.len() == SPL_TOKEN_ACCOUNT_LENGTH
    }
}

/*
    spl_token::state::Mint {
        mint_authority: COption<Pubkey>,
        supply: u64,
        decimals: u8,
        is_initialized: bool,
        freeze_authority: COption<Pubkey>,
    }
*/
const SPL_TOKEN_MINT_SUPPLY_OFFSET: usize = 36;
const SPL_TOKEN_MINT_DECIMALS_OFFSET: usize = 44;
const SPL_TOKEN_MINT_LENGTH: usize = 82;

// Trait for retrieving supply and decimals from any token mint-like buffer.
// A token program that copies the spl_token layout need only impl `valid_account_data()`.
pub trait GenericTokenMint {
    fn valid_account_data(account_data: &[u8]) -> bool;

    define_checked_getter!(unpack_mint_supply, unpack_mint_supply_unchecked, u64);
    define_checked_getter!(unpack_mint_decimals, unpack_mint_decimals_unchecked, u8);

    // Call after account length has already been verified
    fn unpack_mint_supply_unchecked(account_data: &[u8]) -> &u64 {
        unpack_u64_unchecked(account_data, SPL_TOKEN_MINT_SUPPLY_OFFSET)
    }

    // Call after account length has already been verified
    fn unpack_mint_decimals_unchecked(account_data: &[u8]) -> &u8 {
        unpack_u8_unchecked(account_data, SPL_TOKEN_MINT_DECIMALS_OFFSET)
    }
}

pub struct Mint;
impl Mint {
    pub fn get_packed_len() -> usize {
        SPL_TOKEN_MINT_LENGTH
    }
}

impl GenericTokenMint for Mint {
    fn valid_account_data(account_data: &[u8]) -> bool {
        account_data.len() == SPL_TOKEN_MINT_LENGTH
    }
}

pub mod native_mint {
    solana_pubkey::declare_id!("So11111111111111111111111111111111111111112");

    /*
        spl_token::state::Mint {
            mint_authority: COption::None,
            supply: 0,
            decimals: 9,
            is_initialized: true,
            freeze_authority: COption::None,
        }
    */
    pub const ACCOUNT_DATA: [u8; 82] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
}
