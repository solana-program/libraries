//! Crate defining a discriminator type, which creates a set of bytes
//! meant to be unique for instructions or struct types

#![deny(missing_docs)]
#![cfg_attr(not(test), forbid(unsafe_code))]

extern crate self as spl_discriminator;

/// Exports the discriminator module
pub mod discriminator;

// Export for downstream
pub use {
    discriminator::{ArrayDiscriminator, SplDiscriminate},
    spl_discriminator_derive::SplDiscriminate,
};

#[cfg(test)]
mod tests {
    use {super::*, crate::discriminator::ArrayDiscriminator};

    #[allow(dead_code)]
    #[derive(SplDiscriminate)]
    #[discriminator_hash_input("some_discriminator_hash_input")]
    pub struct MyInstruction1 {
        arg1: String,
        arg2: u8,
    }

    #[allow(dead_code)]
    #[derive(SplDiscriminate)]
    #[discriminator_hash_input("yet_another_discriminator_hash_input")]
    pub struct MyInstruction2 {
        arg1: u64,
    }

    #[allow(dead_code)]
    #[derive(SplDiscriminate)]
    #[discriminator_hash_input("global:my_instruction_3")]
    pub enum MyInstruction3 {
        One,
        Two,
        Three,
    }

    fn assert_discriminator<T: spl_discriminator::discriminator::SplDiscriminate>(
        hash_input: &str,
    ) {
        let discriminator = build_discriminator(hash_input);
        assert_eq!(T::SPL_DISCRIMINATOR, discriminator);
        assert_eq!(T::SPL_DISCRIMINATOR_SLICE, discriminator.as_slice());
    }

    fn build_discriminator(hash_input: &str) -> ArrayDiscriminator {
        let preimage = solana_program::hash::hashv(&[hash_input.as_bytes()]);
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&preimage.to_bytes()[..8]);
        ArrayDiscriminator::new(bytes)
    }

    #[test]
    fn test_discrminators() {
        assert_discriminator::<MyInstruction1>("some_discriminator_hash_input");
        assert_discriminator::<MyInstruction2>("yet_another_discriminator_hash_input");
        assert_discriminator::<MyInstruction3>("global:my_instruction_3");
        let runtime_discrim = ArrayDiscriminator::new_with_hash_input("my_new_hash_input");
        assert_eq!(runtime_discrim, build_discriminator("my_new_hash_input"),);
    }
}
