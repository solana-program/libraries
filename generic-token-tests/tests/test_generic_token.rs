use {
    rand::prelude::*,
    spl_generic_token::{generic_token, token, token_2022},
    spl_token::{
        solana_program::program_pack::Pack,
        state::{Account as SplAccount, Mint as SplMint},
    },
};

// TODO actually use the 22 structs, i got this wrong

#[test]
fn test_generic_account() {
    let mut rng = thread_rng();

    for _ in 0..1000 {
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

        let expected_account = SplAccount {
            mint,
            owner,
            amount,
            delegate,
            state,
            is_native,
            delegated_amount,
            close_authority,
        };

        let mut account_data = vec![0; SplAccount::LEN];
        expected_account.pack_into_slice(&mut account_data);

        let is_token_2022_account = rng.gen();
        if is_token_2022_account {
            account_data.push(2);

            assert_eq!(
                generic_token::Account::unpack(&account_data, &token::id()),
                None
            );
        } else {
            let test_account = generic_token::Account::unpack(&account_data, &token::id()).unwrap();

            assert_eq!(test_account.mint, expected_account.mint);
            assert_eq!(test_account.owner, expected_account.owner);
            assert_eq!(test_account.amount, expected_account.amount);
        }

        let test_account =
            generic_token::Account::unpack(&account_data, &token_2022::id()).unwrap();

        assert_eq!(test_account.mint, expected_account.mint);
        assert_eq!(test_account.owner, expected_account.owner);
        assert_eq!(test_account.amount, expected_account.amount);

        assert_eq!(
            generic_token::Mint::unpack(&account_data, &token::id()),
            None
        );

        assert_eq!(
            generic_token::Mint::unpack(&account_data, &token_2022::id()),
            None
        )
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
