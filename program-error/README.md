# SPL Program Error

Macros for implementing error-based traits on enums.

- `#[derive(IntoProgramError)]`: automatically derives the trait `From<Self> for solana_program_error::ProgramError`.
- `#[derive(ToStr)]`: automatically derives the trait `solana_program_error::ToStr`.
- `#[spl_program_error]`: Automatically derives all below traits:
  - `Clone`
  - `Debug`
  - `Eq`
  - `IntoProgramError`
  - `ToStr`
  - `thiserror::Error`
  - `num_derive::FromPrimitive`
  - `num_enum::TryFromPrimitive`
  - `PartialEq`

### `#[derive(IntoProgramError)]`

This derive macro automatically derives the trait `From<Self> for solana_program_error::ProgramError`.

Your enum must implement the following traits in order for this macro to work:

- `Clone`
- `Debug`
- `Eq`
- `thiserror::Error`
- `num_derive::FromPrimitive`
- `num_enum::TryFromPrimitive`
- `PartialEq`

Sample code:

```rust
/// Example error
#[derive(
    Clone, Debug, Eq, IntoProgramError, thiserror::Error, num_derive::FromPrimitive, PartialEq,
)]
pub enum ExampleError {
    /// Mint has no mint authority
    #[error("Mint has no mint authority")]
    MintHasNoMintAuthority,
    /// Incorrect mint authority has signed the instruction
    #[error("Incorrect mint authority has signed the instruction")]
    IncorrectMintAuthority,
}
```

### `#[derive(ToStr)]`

This derive macro automatically derives the trait `solana_program_error::ToStr`.

Your enum must implement the following traits in order for this macro to work:

- `Clone`
- `Debug`
- `Eq`
- `IntoProgramError` (above)
- `num_enum::TryFromPrimitive`
- `PartialEq`

Sample code:

```rust
/// Example error
#[derive(
    Clone,
    Debug,
    Eq,
    IntoProgramError,
    thiserror::Error,
    num_derive::FromPrimitive,
    num_enum::TryFromPrimitive,
    PartialEq,
)]
pub enum ExampleError {
    /// Mint has no mint authority
    #[error("Mint has no mint authority")]
    MintHasNoMintAuthority,
    /// Incorrect mint authority has signed the instruction
    #[error("Incorrect mint authority has signed the instruction")]
    IncorrectMintAuthority,
}
```

### `#[spl_program_error]`

It can be cumbersome to ensure your program's defined errors - typically represented
in an enum - implement the required traits and will print to the program's logs when they're
invoked.

This procedural macro will give you all of the required implementations out of the box:

- `Clone`
- `Debug`
- `Eq`
- `thiserror::Error`
- `num_derive::FromPrimitive`
- `num_enum::TryFromPrimitive`
- `PartialEq`

It also imports the required crates so you don't have to in your program:

- `num_derive`
- `num_enum`
- `num_traits`
- `thiserror`

---

Just annotate your enum...

```rust
use spl_program_error_derive::*;

/// Example error
#[spl_program_error]
pub enum ExampleError {
    /// Mint has no mint authority
    #[error("Mint has no mint authority")]
    MintHasNoMintAuthority,
    /// Incorrect mint authority has signed the instruction
    #[error("Incorrect mint authority has signed the instruction")]
    IncorrectMintAuthority,
}
```

...and get:

```rust
/// Example error
pub enum ExampleError {
    /// Mint has no mint authority
    #[error("Mint has no mint authority")]
    MintHasNoMintAuthority,
    /// Incorrect mint authority has signed the instruction
    #[error("Incorrect mint authority has signed the instruction")]
    IncorrectMintAuthority,
}
#[automatically_derived]
impl ::core::clone::Clone for ExampleError {
    #[inline]
    fn clone(&self) -> ExampleError {
        match self {
            ExampleError::MintHasNoMintAuthority => ExampleError::MintHasNoMintAuthority,
            ExampleError::IncorrectMintAuthority => ExampleError::IncorrectMintAuthority,
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for ExampleError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                ExampleError::MintHasNoMintAuthority => "MintHasNoMintAuthority",
                ExampleError::IncorrectMintAuthority => "IncorrectMintAuthority",
            },
        )
    }
}
#[automatically_derived]
impl ::core::marker::StructuralEq for ExampleError {}
#[automatically_derived]
impl ::core::cmp::Eq for ExampleError {
    #[inline]
    #[doc(hidden)]
    #[no_coverage]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
#[allow(unused_qualifications)]
impl std::error::Error for ExampleError {}
#[allow(unused_qualifications)]
impl std::fmt::Display for ExampleError {
    fn fmt(&self, __formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        #[allow(unused_variables, deprecated, clippy::used_underscore_binding)]
        match self {
            ExampleError::MintHasNoMintAuthority {} => {
                __formatter.write_fmt(format_args!("Mint has no mint authority"))
            }
            ExampleError::IncorrectMintAuthority {} => {
                __formatter
                    .write_fmt(
                        format_args!(
                            "Incorrect mint authority has signed the instruction"
                        ),
                    )
            }
        }
    }
}
#[allow(non_upper_case_globals, unused_qualifications)]
const _IMPL_NUM_FromPrimitive_FOR_ExampleError: () = {
    #[allow(clippy::useless_attribute)]
    #[allow(rust_2018_idioms)]
    extern crate num_traits as _num_traits;
    impl _num_traits::FromPrimitive for ExampleError {
        #[allow(trivial_numeric_casts)]
        #[inline]
        fn from_i64(n: i64) -> Option<Self> {
            if n == ExampleError::MintHasNoMintAuthority as i64 {
                Some(ExampleError::MintHasNoMintAuthority)
            } else if n == ExampleError::IncorrectMintAuthority as i64 {
                Some(ExampleError::IncorrectMintAuthority)
            } else {
                None
            }
        }
        #[inline]
        fn from_u64(n: u64) -> Option<Self> {
            Self::from_i64(n as i64)
        }
    }
};
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ExampleError {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ExampleError {
    #[inline]
    fn eq(&self, other: &ExampleError) -> bool {
        let __self_tag = ::core::intrinsics::discriminant_value(self);
        let __arg1_tag = ::core::intrinsics::discriminant_value(other);
        __self_tag == __arg1_tag
    }
}
impl From<ExampleError> for solana_program_error::ProgramError {
    fn from(e: ExampleError) -> Self {
        solana_program_error::ProgramError::Custom(e as u32)
    }
}
impl solana_program_error::ToStr for ExampleError {
    fn to_str<E>(&self) -> &'static str {
        match self {
            ExampleError::MintHasNoMintAuthority => "Mint has no mint authority",
            ExampleError::IncorrectMintAuthority => "Incorrect mint authority has signed the instruction",
        }
    }
}
```
