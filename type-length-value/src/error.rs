//! Error types
use solana_program_error::{ProgramError, ToStr};

/// Errors that may be returned by the Token program.
#[repr(u32)]
#[derive(
    Clone,
    Debug,
    Eq,
    thiserror::Error,
    num_enum::TryFromPrimitive,
    num_derive::FromPrimitive,
    PartialEq,
)]
pub enum TlvError {
    /// Type not found in TLV data
    #[error("Type not found in TLV data")]
    TypeNotFound = 1_202_666_432,
    /// Type already exists in TLV data
    #[error("Type already exists in TLV data")]
    TypeAlreadyExists,
}

impl From<TlvError> for ProgramError {
    fn from(e: TlvError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl ToStr for TlvError {
    fn to_str(&self) -> &'static str {
        match self {
            TlvError::TypeNotFound => "Type not found in TLV data",
            TlvError::TypeAlreadyExists => "Type already exists in TLV data",
        }
    }
}
