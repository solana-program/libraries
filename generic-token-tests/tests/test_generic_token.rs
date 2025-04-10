use {
    rand::prelude::*,
    spl_generic_token::{generic_token, token, token_2022},
    spl_token::{
        solana_program::program_pack::Pack,
        state::{Account as SplAccount, AccountState as SplAccountState, Mint as SplMint},
    },
    spl_token_2022::{
        extension::set_account_type,
        state::{Account as SplAccount2022, Mint as SplMint2022},
    },
    test_case::test_case,
};

const MULTISIG_LEN: usize = 355;

fn random_token_account() -> SplAccount {
    let mut rng = thread_rng();

    let mint = solana_pubkey::new_rand();
    let owner = solana_pubkey::new_rand();
    let amount = rng.gen();
    let delegate = if rng.gen() {
        Some(solana_pubkey::new_rand())
    } else {
        None
    }
    .into();
    let state = rng.gen_range(0..3).try_into().unwrap();
    let is_native = rng.gen::<Option<u64>>().into();
    let delegated_amount = rng.gen();
    let close_authority = if rng.gen() {
        Some(solana_pubkey::new_rand())
    } else {
        None
    }
    .into();

    SplAccount {
        mint,
        owner,
        amount,
        delegate,
        state,
        is_native,
        delegated_amount,
        close_authority,
    }
}

#[test_case(false; "spl_token")]
#[test_case(true; "spl_token_2022")]
fn test_generic_account(is_token_2022_account: bool) {
    for _ in 0..1000 {
        let expected_account = random_token_account();
        let is_initialized = expected_account.state != SplAccountState::Uninitialized;

        let mut account_data = vec![0; SplAccount::LEN];
        expected_account.pack_into_slice(&mut account_data);

        // check the basic rules of the parser:
        // * uninitialized accounts never parse
        // * standard token accounts parse as both
        // * typed 2022 accounts parse only as 2022
        // note the program_id passed tells the parser how to *treat* the account
        // what the account actually "is" (or "could be") is determined solely by its data
        if is_initialized && is_token_2022_account {
            account_data.resize(SplAccount::LEN + 2, 0);
            set_account_type::<SplAccount2022>(&mut account_data).unwrap();

            // token
            assert_eq!(
                generic_token::Account::unpack(&account_data, &token::id()),
                None
            );

            // token22
            let test_account =
                generic_token::Account::unpack(&account_data, &token_2022::id()).unwrap();

            assert_eq!(test_account.mint, expected_account.mint);
            assert_eq!(test_account.owner, expected_account.owner);
            assert_eq!(test_account.amount, expected_account.amount);
        } else if is_initialized {
            // token
            let test_account = generic_token::Account::unpack(&account_data, &token::id()).unwrap();

            assert_eq!(test_account.mint, expected_account.mint);
            assert_eq!(test_account.owner, expected_account.owner);
            assert_eq!(test_account.amount, expected_account.amount);

            // token22
            let test_account =
                generic_token::Account::unpack(&account_data, &token_2022::id()).unwrap();

            assert_eq!(test_account.mint, expected_account.mint);
            assert_eq!(test_account.owner, expected_account.owner);
            assert_eq!(test_account.amount, expected_account.amount);
        } else {
            // token
            assert_eq!(
                generic_token::Account::unpack(&account_data, &token::id()),
                None
            );

            // token22
            assert_eq!(
                generic_token::Account::unpack(&account_data, &token_2022::id()),
                None
            );
        }

        // a token account should never parse as a mint
        assert_eq!(
            generic_token::Mint::unpack(&account_data, &token::id()),
            None
        );
        assert_eq!(
            generic_token::Mint::unpack(&account_data, &token_2022::id()),
            None
        );

        // an otherwise valid token account should never parse if it is of multisig length
        account_data.resize(MULTISIG_LEN, 0);
        assert_eq!(
            generic_token::Account::unpack(&account_data, &token::id()),
            None
        );
        assert_eq!(
            generic_token::Account::unpack(&account_data, &token_2022::id()),
            None
        );
    }
}

#[test]
fn test_generic_mint() {
    let mut rng = thread_rng();

    for _ in 0..1000 {
        let mint_authority = if rng.gen() {
            Some(solana_pubkey::new_rand())
        } else {
            None
        }
        .into();
        let supply = rng.gen();
        let decimals = rng.gen();
        let is_initialized = rng.gen();
        let freeze_authority = if rng.gen() {
            Some(solana_pubkey::new_rand())
        } else {
            None
        }
        .into();

        let expected_mint = SplMint {
            mint_authority,
            supply,
            decimals,
            is_initialized,
            freeze_authority,
        };

        let mut account_data = vec![0; SplMint::LEN];
        expected_mint.pack_into_slice(&mut account_data);

        let is_token_2022_mint = rng.gen();
        if is_token_2022_mint {
            account_data.push(1);

            assert_eq!(
                generic_token::Mint::unpack(&account_data, &token::id()),
                None
            );
        } else {
            let test_mint = generic_token::Mint::unpack(&account_data, &token::id()).unwrap();

            assert_eq!(test_mint.supply, test_mint.supply);
            assert_eq!(test_mint.decimals, test_mint.decimals);
        }

        let test_mint = generic_token::Mint::unpack(&account_data, &token_2022::id()).unwrap();

        assert_eq!(test_mint.supply, test_mint.supply);
        assert_eq!(test_mint.decimals, test_mint.decimals);

        assert_eq!(
            generic_token::Account::unpack(&account_data, &token::id()),
            None
        );

        assert_eq!(
            generic_token::Account::unpack(&account_data, &token_2022::id()),
            None
        );
    }
}
