# SPL Generic Token

Library that provides bare-bones, dependency-minimized access to SPL Token balance information.

## Example usage

This library provides two core structs:

```rust
spl_generic_token::generic_token::Account {
    mint: Pubkey,
    owner: Pubkey,
    amount: u64,
}

spl_generic_token::generic_token::Mint {
    supply: u64,
    decimals: u8,
}
```

Both provide a static function `fn unpack(account_data: &[u8], program_id: &Pubkey) -> Option<Self>`
which extracts the above fields from a raw buffer in a manner that is generic across `spl_token`
and `spl_token_2022`, without depending on either library.

This is only intended as a simple way to determine balances and direct account ownership. Users
who require additional information such as delegation, mint authority, and so on, should use
the full account parsers in the respective token libraries, as those use-cases exceed the scope of
this tool.

We also provide the trait `GenericTokenAccount` which exposes direct access to the fields named above.

## Note to maintainers

This library is used in parts of Agave that _must not_ depend on `spl_token`, `spl_token_2022`, or
other outside Solana libraries. Care should be taken not to introduce dependencies that may
complicate this situation.
