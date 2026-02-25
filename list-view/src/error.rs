//! Error types
use {
    core::num::TryFromIntError,
    solana_program_error::{ProgramError, ToStr},
};

/// Errors that may be returned by the spl-list-view library.
#[repr(u32)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    thiserror::Error,
    num_enum::TryFromPrimitive,
    num_derive::FromPrimitive,
)]
pub enum ListViewError {
    /// Error in checked math operation
    #[error("Error in checked math operation")]
    CalculationFailure,
    /// Provided byte buffer too small for expected type
    #[error("Provided byte buffer too small for expected type")]
    BufferTooSmall,
    /// An integer conversion failed because the value was out of range for the target type
    #[error("An integer conversion failed because the value was out of range for the target type")]
    ValueOutOfRange,
}

impl From<ListViewError> for ProgramError {
    fn from(e: ListViewError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl ToStr for ListViewError {
    fn to_str(&self) -> &'static str {
        match self {
            ListViewError::CalculationFailure => "Error in checked math operation",
            ListViewError::BufferTooSmall => "Provided byte buffer too small for expected type",
            ListViewError::ValueOutOfRange => "An integer conversion failed because the value was out of range for the target type"
        }
    }
}

impl From<TryFromIntError> for ListViewError {
    fn from(_: TryFromIntError) -> Self {
        ListViewError::ValueOutOfRange
    }
}
