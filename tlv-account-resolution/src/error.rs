//! Error types

use solana_program_error::{ProgramError, ToStr};

/// Errors that may be returned by the Account Resolution library.
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
pub enum AccountResolutionError {
    /// Incorrect account provided
    #[error("Incorrect account provided")]
    IncorrectAccount = 2_724_315_840,
    /// Not enough accounts provided
    #[error("Not enough accounts provided")]
    NotEnoughAccounts,
    /// No value initialized in TLV data
    #[error("No value initialized in TLV data")]
    TlvUninitialized,
    /// Some value initialized in TLV data
    #[error("Some value initialized in TLV data")]
    TlvInitialized,
    /// Too many pubkeys provided
    #[error("Too many pubkeys provided")]
    TooManyPubkeys,
    /// Failed to parse `Pubkey` from bytes
    #[error("Failed to parse `Pubkey` from bytes")]
    InvalidPubkey,
    /// Attempted to deserialize an `AccountMeta` but the underlying type has
    /// PDA configurations rather than a fixed address
    #[error(
        "Attempted to deserialize an `AccountMeta` but the underlying type has PDA configurations rather \
         than a fixed address"
    )]
    AccountTypeNotAccountMeta,
    /// Provided list of seed configurations too large for a validation account
    #[error("Provided list of seed configurations too large for a validation account")]
    SeedConfigsTooLarge,
    /// Not enough bytes available to pack seed configuration
    #[error("Not enough bytes available to pack seed configuration")]
    NotEnoughBytesForSeed,
    /// The provided bytes are not valid for a seed configuration
    #[error("The provided bytes are not valid for a seed configuration")]
    InvalidBytesForSeed,
    /// Tried to pack an invalid seed configuration
    #[error("Tried to pack an invalid seed configuration")]
    InvalidSeedConfig,
    /// Instruction data too small for seed configuration
    #[error("Instruction data too small for seed configuration")]
    InstructionDataTooSmall,
    /// Could not find account at specified index
    #[error("Could not find account at specified index")]
    AccountNotFound,
    /// Error in checked math operation
    #[error("Error in checked math operation")]
    CalculationFailure,
    /// Could not find account data at specified index
    #[error("Could not find account data at specified index")]
    AccountDataNotFound,
    /// Account data too small for requested seed configuration
    #[error("Account data too small for requested seed configuration")]
    AccountDataTooSmall,
    /// Failed to fetch account
    #[error("Failed to fetch account")]
    AccountFetchFailed,
    /// Not enough bytes available to pack pubkey data configuration.
    #[error("Not enough bytes available to pack pubkey data configuration")]
    NotEnoughBytesForPubkeyData,
    /// The provided bytes are not valid for a pubkey data configuration
    #[error("The provided bytes are not valid for a pubkey data configuration")]
    InvalidBytesForPubkeyData,
    /// Tried to pack an invalid pubkey data configuration
    #[error("Tried to pack an invalid pubkey data configuration")]
    InvalidPubkeyDataConfig,
}

impl From<AccountResolutionError> for ProgramError {
    fn from(e: AccountResolutionError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl ToStr for AccountResolutionError {
    fn to_str(&self) -> &'static str {
        match self {
            AccountResolutionError::IncorrectAccount => {
                "Incorrect account provided"
            }
            AccountResolutionError::NotEnoughAccounts => {
                "Not enough accounts provided"
            }
            AccountResolutionError::TlvUninitialized => {
                "No value initialized in TLV data"
            }
            AccountResolutionError::TlvInitialized => {
                "Some value initialized in TLV data"
            }
            AccountResolutionError::TooManyPubkeys => {
                "Too many pubkeys provided"
            }
            AccountResolutionError::InvalidPubkey => {
                "Failed to parse `Pubkey` from bytes"
            }
            AccountResolutionError::AccountTypeNotAccountMeta => {
                "Attempted to deserialize an `AccountMeta` but the underlying type has PDA configs rather than a fixed address"
            }
            AccountResolutionError::SeedConfigsTooLarge => {
                "Provided list of seed configurations too large for a validation account"
            }
            AccountResolutionError::NotEnoughBytesForSeed => {
                "Not enough bytes available to pack seed configuration"
            }
            AccountResolutionError::InvalidBytesForSeed => {
                "The provided bytes are not valid for a seed configuration"
            }
            AccountResolutionError::InvalidSeedConfig => {
                "Tried to pack an invalid seed configuration"
            }
            AccountResolutionError::InstructionDataTooSmall => {
                "Instruction data too small for seed configuration"
            }
            AccountResolutionError::AccountNotFound => {
                "Could not find account at specified index"
            }
            AccountResolutionError::CalculationFailure => {
                "Error in checked math operation"
            }
            AccountResolutionError::AccountDataNotFound => {
                "Could not find account data at specified index"
            }
            AccountResolutionError::AccountDataTooSmall => {
                "Account data too small for requested seed configuration"
            }
            AccountResolutionError::AccountFetchFailed => {
                "Failed to fetch account"
            }
            AccountResolutionError::NotEnoughBytesForPubkeyData => {
                "Not enough bytes available to pack pubkey data configuration"
            }
            AccountResolutionError::InvalidBytesForPubkeyData => {
                "The provided bytes are not valid for a pubkey data configuration"
            }
            AccountResolutionError::InvalidPubkeyDataConfig => {
                "Tried to pack an invalid pubkey data configuration"
            }
        }
    }
}
