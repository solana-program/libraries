pub mod bench;
pub mod into;
pub mod spl;
pub mod to_str;

#[cfg(test)]
mod tests {
    use {
        super::*,
        solana_program_error::{ProgramError, ToStr},
    };

    // `#[derive(IntoProgramError)]`
    #[test]
    fn test_derive_into_program_error() {
        // `Into<ProgramError>`
        assert_eq!(
            Into::<ProgramError>::into(bench::ExampleError::MintHasNoMintAuthority),
            Into::<ProgramError>::into(into::ExampleError::MintHasNoMintAuthority),
        );
        assert_eq!(
            Into::<ProgramError>::into(bench::ExampleError::IncorrectMintAuthority),
            Into::<ProgramError>::into(into::ExampleError::IncorrectMintAuthority),
        );
    }

    // `#[derive(ToStr)]`
    #[test]
    fn test_derive_to_str() {
        // `Into<ProgramError>`
        assert_eq!(
            Into::<ProgramError>::into(bench::ExampleError::MintHasNoMintAuthority),
            Into::<ProgramError>::into(to_str::ExampleError::MintHasNoMintAuthority),
        );
        assert_eq!(
            Into::<ProgramError>::into(bench::ExampleError::IncorrectMintAuthority),
            Into::<ProgramError>::into(to_str::ExampleError::IncorrectMintAuthority),
        );
        // `ToStr`
        assert_eq!(
            ToStr::to_str(&to_str::ExampleError::MintHasNoMintAuthority,),
            "Mint has no mint authority"
        );
        assert_eq!(
            ToStr::to_str(&to_str::ExampleError::IncorrectMintAuthority,),
            "Incorrect mint authority has signed the instruction"
        );
    }

    // `#[spl_program_error]`
    #[test]
    fn test_spl_program_error() {
        // `Into<ProgramError>`
        assert_eq!(
            Into::<ProgramError>::into(bench::ExampleError::MintHasNoMintAuthority),
            Into::<ProgramError>::into(spl::ExampleError::MintHasNoMintAuthority),
        );
        assert_eq!(
            Into::<ProgramError>::into(bench::ExampleError::IncorrectMintAuthority),
            Into::<ProgramError>::into(spl::ExampleError::IncorrectMintAuthority),
        );
        // `ToStr`
        assert_eq!(
            ToStr::to_str(&spl::ExampleError::MintHasNoMintAuthority),
            "Mint has no mint authority"
        );
        assert_eq!(
            ToStr::to_str(&spl::ExampleError::IncorrectMintAuthority),
            "Incorrect mint authority has signed the instruction",
        );
    }
}
