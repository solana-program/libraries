//! Bench case with manual implementations
use spl_program_error::*;

/// Example error
#[derive(Clone, Debug, Eq, thiserror::Error, num_derive::FromPrimitive, PartialEq)]
pub enum ExampleError {
    /// Mint has no mint authority
    #[error("Mint has no mint authority")]
    MintHasNoMintAuthority,
    /// Incorrect mint authority has signed the instruction
    #[error("Incorrect mint authority has signed the instruction")]
    IncorrectMintAuthority,
}

impl From<ExampleError> for solana_program_error::ProgramError {
    fn from(e: ExampleError) -> Self {
        solana_program_error::ProgramError::Custom(e as u32)
    }
}

impl solana_program_error::ToStr for ExampleError {
    fn to_str(&self) -> &'static str {
        match self {
            ExampleError::MintHasNoMintAuthority => "Mint has no mint authority",
            ExampleError::IncorrectMintAuthority => {
                "Incorrect mint authority has signed the instruction"
            }
        }
    }
}

/// Tests that all macros compile
#[test]
fn test_macros_compile() {
    let _ = ExampleError::MintHasNoMintAuthority;
}
