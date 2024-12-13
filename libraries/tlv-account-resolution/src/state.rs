//! State transition types

use {
    crate::{account::ExtraAccountMeta, error::AccountResolutionError},
    solana_program::{
        account_info::AccountInfo,
        instruction::{AccountMeta, Instruction},
        program_error::ProgramError,
    },
    spl_discriminator::SplDiscriminate,
    spl_type_length_value::{
        pod::{PodSlice, PodSliceMut},
        state::{TlvState, TlvStateBorrowed, TlvStateMut},
    },
};

/// Stateless helper for storing additional accounts required for an
/// instruction.
///
/// This struct works with any `SplDiscriminate`, and stores the extra accounts
/// needed for that specific instruction, using the given `ArrayDiscriminator`
/// as the type-length-value `ArrayDiscriminator`, and then storing all of the
/// given `AccountMeta`s as a zero-copy slice.
///
/// Sample usage:
///
/// ```
/// use {
///     solana_program::{
///         account_info::AccountInfo, instruction::{AccountMeta, Instruction},
///         pubkey::Pubkey
///     },
///     spl_discriminator::{ArrayDiscriminator, SplDiscriminate},
///     spl_tlv_account_resolution::state::ExtraAccountMetaList,
/// };
///
/// struct MyInstruction;
/// impl SplDiscriminate for MyInstruction {
///     // Give it a unique discriminator, can also be generated using a hash function
///     const SPL_DISCRIMINATOR: ArrayDiscriminator = ArrayDiscriminator::new([1; ArrayDiscriminator::LENGTH]);
/// }
///
/// // actually put it in the additional required account keys and signer / writable
/// let extra_metas = [
///     AccountMeta::new(Pubkey::new_unique(), false),
///     AccountMeta::new(Pubkey::new_unique(), true),
///     AccountMeta::new_readonly(Pubkey::new_unique(), true),
///     AccountMeta::new_readonly(Pubkey::new_unique(), false),
/// ];
///
/// // assume that this buffer is actually account data, already allocated to `account_size`
/// let account_size = ExtraAccountMetaList::size_of(extra_metas.len()).unwrap();
/// let mut buffer = vec![0; account_size];
///
/// // Initialize the structure for your instruction
/// ExtraAccountMetaList::init_with_account_metas::<MyInstruction>(&mut buffer, &extra_metas).unwrap();
///
/// // Off-chain, you can add the additional accounts directly from the account data
/// let program_id = Pubkey::new_unique();
/// let mut instruction = Instruction::new_with_bytes(program_id, &[], vec![]);
/// ExtraAccountMetaList::add_to_instruction::<MyInstruction>(&mut instruction, &buffer).unwrap();
///
/// // On-chain, you can add the additional accounts *and* account infos
/// let mut cpi_instruction = Instruction::new_with_bytes(program_id, &[], vec![]);
/// let mut cpi_account_infos = vec![]; // assume the other required account infos are already included
/// let remaining_account_infos: &[AccountInfo<'_>] = &[]; // these are the account infos provided to the instruction that are *not* part of any other known interface
/// ExtraAccountMetaList::add_to_cpi_instruction::<MyInstruction>(
///     &mut cpi_instruction,
///     &mut cpi_account_infos,
///     &buffer,
///     &remaining_account_infos,
/// );
/// ```
pub struct ExtraAccountMetaList;
impl ExtraAccountMetaList {
    /// Initialize pod slice data for the given instruction and any type
    /// convertible to account metas
    fn init<'a, T: SplDiscriminate, F, M>(
        data: &mut [u8],
        convertible_account_metas: &'a [M],
        conversion_fn: F,
    ) -> Result<(), ProgramError>
    where
        F: Fn(&'a M) -> ExtraAccountMeta,
    {
        let mut state = TlvStateMut::unpack(data).unwrap();
        let tlv_size = PodSlice::<ExtraAccountMeta>::size_of(convertible_account_metas.len())?;
        let (bytes, _) = state.alloc::<T>(tlv_size, false)?;
        let mut extra_account_metas = PodSliceMut::init(bytes)?;
        for account_meta in convertible_account_metas {
            extra_account_metas.push(conversion_fn(account_meta))?;
        }
        Ok(())
    }

    /// Initialize a TLV entry for the given discriminator, populating the data
    /// with the given account infos
    pub fn init_with_account_infos<T: SplDiscriminate>(
        data: &mut [u8],
        account_infos: &[AccountInfo<'_>],
    ) -> Result<(), ProgramError> {
        Self::init::<T, _, AccountInfo>(data, account_infos, |account_info| {
            ExtraAccountMeta::from(account_info)
        })
    }

    /// Initialize a TLV entry for the given discriminator, populating the data
    /// with the given account metas
    pub fn init_with_account_metas<T: SplDiscriminate>(
        data: &mut [u8],
        account_metas: &[AccountMeta],
    ) -> Result<(), ProgramError> {
        Self::init::<T, _, AccountMeta>(data, account_metas, |account_meta| {
            ExtraAccountMeta::from(account_meta)
        })
    }

    /// Initialize a TLV entry for the given discriminator, populating the data
    /// with the given required accounts - which can be standard `AccountMeta`s
    /// or PDAs
    pub fn init_with_pod_account_metas<T: SplDiscriminate>(
        data: &mut [u8],
        pod_account_metas: &[ExtraAccountMeta],
    ) -> Result<(), ProgramError> {
        Self::init::<T, _, ExtraAccountMeta>(data, pod_account_metas, |pod_account_meta| {
            *pod_account_meta
        })
    }

    /// Get the underlying `PodSlice<ExtraAccountMeta>` from an unpacked TLV
    ///
    /// Due to lifetime annoyances, this function can't just take in the bytes,
    /// since then we would be returning a reference to a locally created
    /// `TlvStateBorrowed`. I hope there's a better way to do this!
    pub fn unpack_with_tlv_state<'a, T: SplDiscriminate>(
        tlv_state: &'a TlvStateBorrowed,
    ) -> Result<PodSlice<'a, ExtraAccountMeta>, ProgramError> {
        let bytes = tlv_state.get_first_bytes::<T>()?;
        PodSlice::<ExtraAccountMeta>::unpack(bytes)
    }

    /// Get the byte size required to hold `num_items` items
    pub fn size_of(num_items: usize) -> Result<usize, ProgramError> {
        Ok(TlvStateBorrowed::get_base_len()
            .saturating_add(PodSlice::<ExtraAccountMeta>::size_of(num_items)?))
    }

    /// Add the additional account metas to an existing instruction
    pub fn add_to_instruction<T: SplDiscriminate>(
        instruction: &mut Instruction,
        data: &[u8],
    ) -> Result<(), ProgramError> {
        let state = TlvStateBorrowed::unpack(data)?;
        let bytes = state.get_first_bytes::<T>()?;
        let extra_account_metas = PodSlice::<ExtraAccountMeta>::unpack(bytes)?;

        for extra_meta in extra_account_metas.data().iter() {
            instruction.accounts.push(extra_meta.resolve(instruction)?);
        }
        Ok(())
    }

    /// Add the additional account metas and account infos for a CPI
    pub fn add_to_cpi_instruction<'a, T: SplDiscriminate>(
        cpi_instruction: &mut Instruction,
        cpi_account_infos: &mut Vec<AccountInfo<'a>>,
        data: &[u8],
        account_infos: &[AccountInfo<'a>],
    ) -> Result<(), ProgramError> {
        let initial_instruction_metas_len = cpi_instruction.accounts.len();

        Self::add_to_instruction::<T>(cpi_instruction, data)?;

        for account_meta in cpi_instruction
            .accounts
            .iter()
            .skip(initial_instruction_metas_len)
        {
            let account_info = account_infos
                .iter()
                .find(|&x| *x.key == account_meta.pubkey)
                .ok_or(AccountResolutionError::IncorrectAccount)?
                .clone();
            cpi_account_infos.push(account_info);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::seeds::Seed,
        solana_program::{clock::Epoch, instruction::AccountMeta, pubkey::Pubkey},
        spl_discriminator::{ArrayDiscriminator, SplDiscriminate},
    };

    pub struct TestInstruction;
    impl SplDiscriminate for TestInstruction {
        const SPL_DISCRIMINATOR: ArrayDiscriminator =
            ArrayDiscriminator::new([1; ArrayDiscriminator::LENGTH]);
    }

    pub struct TestOtherInstruction;
    impl SplDiscriminate for TestOtherInstruction {
        const SPL_DISCRIMINATOR: ArrayDiscriminator =
            ArrayDiscriminator::new([2; ArrayDiscriminator::LENGTH]);
    }

    #[test]
    fn init_with_metas() {
        let metas = [
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), true),
            AccountMeta::new_readonly(Pubkey::new_unique(), true),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
        ];
        let account_size = ExtraAccountMetaList::size_of(metas.len()).unwrap();
        let mut buffer = vec![0; account_size];

        ExtraAccountMetaList::init_with_account_metas::<TestInstruction>(&mut buffer, &metas)
            .unwrap();

        let mut instruction = Instruction::new_with_bytes(Pubkey::new_unique(), &[], vec![]);
        ExtraAccountMetaList::add_to_instruction::<TestInstruction>(&mut instruction, &buffer)
            .unwrap();

        assert_eq!(
            instruction
                .accounts
                .iter()
                .map(ExtraAccountMeta::from)
                .collect::<Vec<_>>(),
            metas.iter().map(ExtraAccountMeta::from).collect::<Vec<_>>()
        );
    }

    #[test]
    fn init_with_infos() {
        let pubkey1 = Pubkey::new_unique();
        let mut lamports1 = 0;
        let mut data1 = [];
        let pubkey2 = Pubkey::new_unique();
        let mut lamports2 = 0;
        let mut data2 = [];
        let pubkey3 = Pubkey::new_unique();
        let mut lamports3 = 0;
        let mut data3 = [];
        let owner = Pubkey::new_unique();
        let account_infos = [
            AccountInfo::new(
                &pubkey1,
                false,
                true,
                &mut lamports1,
                &mut data1,
                &owner,
                false,
                Epoch::default(),
            ),
            AccountInfo::new(
                &pubkey2,
                true,
                false,
                &mut lamports2,
                &mut data2,
                &owner,
                false,
                Epoch::default(),
            ),
            AccountInfo::new(
                &pubkey3,
                false,
                false,
                &mut lamports3,
                &mut data3,
                &owner,
                false,
                Epoch::default(),
            ),
        ];
        let account_size = ExtraAccountMetaList::size_of(account_infos.len()).unwrap();
        let mut buffer = vec![0; account_size];

        ExtraAccountMetaList::init_with_account_infos::<TestInstruction>(
            &mut buffer,
            &account_infos,
        )
        .unwrap();

        let mut instruction = Instruction::new_with_bytes(Pubkey::new_unique(), &[], vec![]);
        ExtraAccountMetaList::add_to_instruction::<TestInstruction>(&mut instruction, &buffer)
            .unwrap();

        assert_eq!(
            instruction
                .accounts
                .iter()
                .map(ExtraAccountMeta::from)
                .collect::<Vec<_>>(),
            account_infos
                .iter()
                .map(ExtraAccountMeta::from)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn init_with_pod_account_metas() {
        let program_id = Pubkey::new_unique();

        let extra_meta3_literal_str = "seed_prefix";

        let ix_account1 = AccountMeta::new(Pubkey::new_unique(), false);
        let ix_account2 = AccountMeta::new(Pubkey::new_unique(), true);

        let extra_meta1 = AccountMeta::new(Pubkey::new_unique(), false);
        let extra_meta2 = AccountMeta::new(Pubkey::new_unique(), true);
        let extra_meta3 = ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: extra_meta3_literal_str.as_bytes().to_vec(),
                },
                Seed::InstructionData {
                    index: 1,
                    length: 1, // u8
                },
                Seed::AccountKey { index: 0 },
                Seed::AccountKey { index: 2 },
            ],
            false,
            true,
        )
        .unwrap();

        let metas = [
            ExtraAccountMeta::from(&extra_meta1),
            ExtraAccountMeta::from(&extra_meta2),
            extra_meta3,
        ];

        let ix_data = vec![1, 2, 3, 4];
        let ix_accounts = vec![ix_account1.clone(), ix_account2.clone()];
        let mut instruction = Instruction::new_with_bytes(program_id, &ix_data, ix_accounts);

        let account_size = ExtraAccountMetaList::size_of(metas.len()).unwrap();
        let mut buffer = vec![0; account_size];

        // Notice we use `init_with_required_accounts` instead of
        // `init_with_account_metas`
        ExtraAccountMetaList::init_with_pod_account_metas::<TestInstruction>(&mut buffer, &metas)
            .unwrap();

        ExtraAccountMetaList::add_to_instruction::<TestInstruction>(&mut instruction, &buffer)
            .unwrap();

        let check_extra_meta3_u8_arg = ix_data[1];
        let check_extra_meta3_pubkey = Pubkey::find_program_address(
            &[
                extra_meta3_literal_str.as_bytes(),
                &[check_extra_meta3_u8_arg],
                ix_account1.pubkey.as_ref(),
                extra_meta1.pubkey.as_ref(),
            ],
            &program_id,
        )
        .0;
        let check_metas = [
            ix_account1,
            ix_account2,
            extra_meta1,
            extra_meta2,
            AccountMeta::new(check_extra_meta3_pubkey, false),
        ];

        assert_eq!(
            instruction.accounts.get(4).unwrap().pubkey,
            check_extra_meta3_pubkey,
        );
        assert_eq!(
            instruction
                .accounts
                .iter()
                .map(ExtraAccountMeta::from)
                .collect::<Vec<_>>(),
            check_metas
                .iter()
                .map(ExtraAccountMeta::from)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn init_multiple() {
        let extra_meta5_literal_str = "seed_prefix";
        let extra_meta5_literal_u32 = 4u32;
        let other_meta2_literal_str = "other_seed_prefix";

        let extra_meta1 = AccountMeta::new(Pubkey::new_unique(), false);
        let extra_meta2 = AccountMeta::new(Pubkey::new_unique(), true);
        let extra_meta3 = AccountMeta::new_readonly(Pubkey::new_unique(), true);
        let extra_meta4 = AccountMeta::new_readonly(Pubkey::new_unique(), false);
        let extra_meta5 = ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: extra_meta5_literal_str.as_bytes().to_vec(),
                },
                Seed::Literal {
                    bytes: extra_meta5_literal_u32.to_le_bytes().to_vec(),
                },
                Seed::InstructionData {
                    index: 5,
                    length: 1, // u8
                },
                Seed::AccountKey { index: 2 },
            ],
            false,
            true,
        )
        .unwrap();

        let other_meta1 = AccountMeta::new(Pubkey::new_unique(), false);
        let other_meta2 = ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: other_meta2_literal_str.as_bytes().to_vec(),
                },
                Seed::InstructionData {
                    index: 1,
                    length: 4, // u32
                },
                Seed::AccountKey { index: 0 },
            ],
            false,
            true,
        )
        .unwrap();

        let metas = [
            ExtraAccountMeta::from(&extra_meta1),
            ExtraAccountMeta::from(&extra_meta2),
            ExtraAccountMeta::from(&extra_meta3),
            ExtraAccountMeta::from(&extra_meta4),
            extra_meta5,
        ];
        let other_metas = [ExtraAccountMeta::from(&other_meta1), other_meta2];

        let account_size = ExtraAccountMetaList::size_of(metas.len()).unwrap()
            + ExtraAccountMetaList::size_of(other_metas.len()).unwrap();
        let mut buffer = vec![0; account_size];

        ExtraAccountMetaList::init_with_pod_account_metas::<TestInstruction>(&mut buffer, &metas)
            .unwrap();
        ExtraAccountMetaList::init_with_pod_account_metas::<TestOtherInstruction>(
            &mut buffer,
            &other_metas,
        )
        .unwrap();

        let program_id = Pubkey::new_unique();
        let ix_data = vec![0, 0, 0, 0, 0, 7, 0, 0];
        let ix_accounts = vec![];
        let mut instruction = Instruction::new_with_bytes(program_id, &ix_data, ix_accounts);
        ExtraAccountMetaList::add_to_instruction::<TestInstruction>(&mut instruction, &buffer)
            .unwrap();

        let check_extra_meta5_u8_arg = ix_data[5];
        let check_extra_meta5_pubkey = Pubkey::find_program_address(
            &[
                extra_meta5_literal_str.as_bytes(),
                extra_meta5_literal_u32.to_le_bytes().as_ref(),
                &[check_extra_meta5_u8_arg],
                extra_meta3.pubkey.as_ref(),
            ],
            &program_id,
        )
        .0;
        let check_metas = [
            extra_meta1,
            extra_meta2,
            extra_meta3,
            extra_meta4,
            AccountMeta::new(check_extra_meta5_pubkey, false),
        ];

        assert_eq!(
            instruction.accounts.get(4).unwrap().pubkey,
            check_extra_meta5_pubkey,
        );
        assert_eq!(
            instruction
                .accounts
                .iter()
                .map(ExtraAccountMeta::from)
                .collect::<Vec<_>>(),
            check_metas
                .iter()
                .map(ExtraAccountMeta::from)
                .collect::<Vec<_>>()
        );

        let program_id = Pubkey::new_unique();
        let ix_account1 = AccountMeta::new(Pubkey::new_unique(), false);
        let ix_account2 = AccountMeta::new(Pubkey::new_unique(), true);
        let ix_accounts = vec![ix_account1.clone(), ix_account2.clone()];
        let ix_data = vec![0, 26, 0, 0, 0, 0, 0];
        let mut instruction = Instruction::new_with_bytes(program_id, &ix_data, ix_accounts);
        ExtraAccountMetaList::add_to_instruction::<TestOtherInstruction>(&mut instruction, &buffer)
            .unwrap();

        let check_other_meta2_u32_arg = u32::from_le_bytes(ix_data[1..5].try_into().unwrap());
        let check_other_meta2_pubkey = Pubkey::find_program_address(
            &[
                other_meta2_literal_str.as_bytes(),
                check_other_meta2_u32_arg.to_le_bytes().as_ref(),
                ix_account1.pubkey.as_ref(),
            ],
            &program_id,
        )
        .0;
        let check_other_metas = [
            ix_account1,
            ix_account2,
            other_meta1,
            AccountMeta::new(check_other_meta2_pubkey, false),
        ];

        assert_eq!(
            instruction.accounts.get(3).unwrap().pubkey,
            check_other_meta2_pubkey,
        );
        assert_eq!(
            instruction
                .accounts
                .iter()
                .map(ExtraAccountMeta::from)
                .collect::<Vec<_>>(),
            check_other_metas
                .iter()
                .map(ExtraAccountMeta::from)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn init_mixed() {
        let extra_meta5_literal_str = "seed_prefix";
        let extra_meta6_literal_u64 = 28u64;

        let pubkey1 = Pubkey::new_unique();
        let mut lamports1 = 0;
        let mut data1 = [];
        let pubkey2 = Pubkey::new_unique();
        let mut lamports2 = 0;
        let mut data2 = [];
        let pubkey3 = Pubkey::new_unique();
        let mut lamports3 = 0;
        let mut data3 = [];
        let owner = Pubkey::new_unique();
        let account_infos = [
            AccountInfo::new(
                &pubkey1,
                false,
                true,
                &mut lamports1,
                &mut data1,
                &owner,
                false,
                Epoch::default(),
            ),
            AccountInfo::new(
                &pubkey2,
                true,
                false,
                &mut lamports2,
                &mut data2,
                &owner,
                false,
                Epoch::default(),
            ),
            AccountInfo::new(
                &pubkey3,
                false,
                false,
                &mut lamports3,
                &mut data3,
                &owner,
                false,
                Epoch::default(),
            ),
        ];

        let extra_meta1 = AccountMeta::new(Pubkey::new_unique(), false);
        let extra_meta2 = AccountMeta::new(Pubkey::new_unique(), true);
        let extra_meta3 = AccountMeta::new_readonly(Pubkey::new_unique(), true);
        let extra_meta4 = AccountMeta::new_readonly(Pubkey::new_unique(), false);
        let extra_meta5 = ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: extra_meta5_literal_str.as_bytes().to_vec(),
                },
                Seed::InstructionData {
                    index: 1,
                    length: 8, // [u8; 8]
                },
                Seed::InstructionData {
                    index: 9,
                    length: 32, // Pubkey
                },
                Seed::AccountKey { index: 2 },
            ],
            false,
            true,
        )
        .unwrap();
        let extra_meta6 = ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: extra_meta6_literal_u64.to_le_bytes().to_vec(),
                },
                Seed::AccountKey { index: 1 },
                Seed::AccountKey { index: 4 },
            ],
            false,
            true,
        )
        .unwrap();

        let metas = [
            ExtraAccountMeta::from(&extra_meta1),
            ExtraAccountMeta::from(&extra_meta2),
            ExtraAccountMeta::from(&extra_meta3),
            ExtraAccountMeta::from(&extra_meta4),
            extra_meta5,
            extra_meta6,
        ];

        let account_size = ExtraAccountMetaList::size_of(account_infos.len()).unwrap()
            + ExtraAccountMetaList::size_of(metas.len()).unwrap();
        let mut buffer = vec![0; account_size];

        ExtraAccountMetaList::init_with_account_infos::<TestInstruction>(
            &mut buffer,
            &account_infos,
        )
        .unwrap();
        ExtraAccountMetaList::init_with_pod_account_metas::<TestOtherInstruction>(
            &mut buffer,
            &metas,
        )
        .unwrap();

        let program_id = Pubkey::new_unique();
        let mut instruction = Instruction::new_with_bytes(program_id, &[], vec![]);
        ExtraAccountMetaList::add_to_instruction::<TestInstruction>(&mut instruction, &buffer)
            .unwrap();
        assert_eq!(
            instruction
                .accounts
                .iter()
                .map(ExtraAccountMeta::from)
                .collect::<Vec<_>>(),
            account_infos
                .iter()
                .map(ExtraAccountMeta::from)
                .collect::<Vec<_>>()
        );

        let program_id = Pubkey::new_unique();
        let instruction_u8array_arg = [1, 2, 3, 4, 5, 6, 7, 8];
        let instruction_pubkey_arg = Pubkey::new_unique();
        let mut instruction_data = vec![0];
        instruction_data.extend_from_slice(&instruction_u8array_arg);
        instruction_data.extend_from_slice(instruction_pubkey_arg.as_ref());
        let mut instruction = Instruction::new_with_bytes(program_id, &instruction_data, vec![]);
        ExtraAccountMetaList::add_to_instruction::<TestOtherInstruction>(&mut instruction, &buffer)
            .unwrap();

        let check_extra_meta5_pubkey = Pubkey::find_program_address(
            &[
                extra_meta5_literal_str.as_bytes(),
                &instruction_u8array_arg,
                instruction_pubkey_arg.as_ref(),
                extra_meta3.pubkey.as_ref(),
            ],
            &program_id,
        )
        .0;

        let check_extra_meta6_pubkey = Pubkey::find_program_address(
            &[
                extra_meta6_literal_u64.to_le_bytes().as_ref(),
                extra_meta2.pubkey.as_ref(),
                check_extra_meta5_pubkey.as_ref(), // The first PDA should be at index 4
            ],
            &program_id,
        )
        .0;

        let check_metas = vec![
            extra_meta1,
            extra_meta2,
            extra_meta3,
            extra_meta4,
            AccountMeta::new(check_extra_meta5_pubkey, false),
            AccountMeta::new(check_extra_meta6_pubkey, false),
        ];

        assert_eq!(
            instruction.accounts.get(4).unwrap().pubkey,
            check_extra_meta5_pubkey,
        );
        assert_eq!(
            instruction.accounts.get(5).unwrap().pubkey,
            check_extra_meta6_pubkey,
        );
        assert_eq!(
            instruction
                .accounts
                .iter()
                .map(ExtraAccountMeta::from)
                .collect::<Vec<_>>(),
            check_metas
                .iter()
                .map(ExtraAccountMeta::from)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn cpi_instruction() {
        // Say we have a program that CPIs to another program.
        //
        // Say that _other_ program will need extra account infos.

        // This will be our program. Let's ignore the account info
        // for the other program in this example.
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        // First let's build a list of account infos for the CPI
        // instruction itself.
        let pubkey_ix_1 = Pubkey::new_unique();
        let mut lamports_ix_1 = 0;
        let mut data_ix_1 = [];
        let pubkey_ix_2 = Pubkey::new_unique();
        let mut lamports_ix_2 = 0;
        let mut data_ix_2 = [];
        // For the CPI account infos themselves.
        let ix_account_infos = [
            AccountInfo::new(
                &pubkey_ix_1,
                false,
                true,
                &mut lamports_ix_1,
                &mut data_ix_1,
                &owner,
                false,
                Epoch::default(),
            ),
            AccountInfo::new(
                &pubkey_ix_2,
                false,
                true,
                &mut lamports_ix_2,
                &mut data_ix_2,
                &owner,
                false,
                Epoch::default(),
            ),
        ];
        // For the CPI instruction's list of account metas.
        let ix_accounts = vec![
            AccountMeta::new(*ix_account_infos[0].key, false),
            AccountMeta::new(*ix_account_infos[1].key, false),
        ];

        // Now let's build a list of extra account infos required by
        // the program we are going to CPI to.
        let pubkey1 = Pubkey::new_unique();
        let mut lamports1 = 0;
        let mut data1 = [];
        let pubkey2 = Pubkey::new_unique();
        let mut lamports2 = 0;
        let mut data2 = [];
        let pubkey3 = Pubkey::new_unique();
        let mut lamports3 = 0;
        let mut data3 = [];
        let owner = Pubkey::new_unique();
        let extra_account_infos = [
            AccountInfo::new(
                &pubkey1,
                false,
                true,
                &mut lamports1,
                &mut data1,
                &owner,
                false,
                Epoch::default(),
            ),
            AccountInfo::new(
                &pubkey2,
                true,
                false,
                &mut lamports2,
                &mut data2,
                &owner,
                false,
                Epoch::default(),
            ),
            AccountInfo::new(
                &pubkey3,
                false,
                false,
                &mut lamports3,
                &mut data3,
                &owner,
                false,
                Epoch::default(),
            ),
        ];

        // Let's also add 2 required PDAs to the extra required accounts.

        let required_pda1_literal_string = "required_pda1";
        let required_pda2_literal_u32 = 4u32;

        let required_pda1 = ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: required_pda1_literal_string.as_bytes().to_vec(),
                },
                Seed::InstructionData {
                    index: 1,
                    length: 8, // [u8; 8]
                },
                Seed::AccountKey { index: 1 },
            ],
            false,
            true,
        )
        .unwrap();
        let required_pda2 = ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: required_pda2_literal_u32.to_le_bytes().to_vec(),
                },
                Seed::InstructionData {
                    index: 9,
                    length: 8, // u64
                },
                Seed::AccountKey { index: 5 },
            ],
            false,
            true,
        )
        .unwrap();

        // The program to CPI to has 2 account metas and
        // 5 extra required accounts (3 metas, 2 PDAs).

        // Now we set up the validation account data

        let mut required_accounts = extra_account_infos
            .iter()
            .map(ExtraAccountMeta::from)
            .collect::<Vec<_>>();
        required_accounts.push(required_pda1);
        required_accounts.push(required_pda2);

        let account_size = ExtraAccountMetaList::size_of(required_accounts.len()).unwrap();
        let mut buffer = vec![0; account_size];

        ExtraAccountMetaList::init_with_pod_account_metas::<TestInstruction>(
            &mut buffer,
            &required_accounts,
        )
        .unwrap();

        // Make an instruction to check later
        // We'll also check the instruction seed components later
        let instruction_u8array_arg = [1, 2, 3, 4, 5, 6, 7, 8];
        let instruction_u64_arg = 208u64;
        let mut instruction_data = vec![0];
        instruction_data.extend_from_slice(&instruction_u8array_arg);
        instruction_data.extend_from_slice(instruction_u64_arg.to_le_bytes().as_ref());

        let mut instruction =
            Instruction::new_with_bytes(program_id, &instruction_data, ix_accounts.clone());
        ExtraAccountMetaList::add_to_instruction::<TestInstruction>(&mut instruction, &buffer)
            .unwrap();

        // Now our program is going to use its own set of account infos.
        //
        // These account infos must contain all required account infos for the CPI.
        //
        // We'll mess them up a bit to make sure the ordering doesn't matter when
        // performing account resolution.
        let mut messed_account_infos = Vec::new();

        // First add the instruction account infos.
        messed_account_infos.extend(ix_account_infos.clone());

        // Next add the extra account infos.
        messed_account_infos.extend(extra_account_infos.iter().cloned());

        // Also add the extra PDAs with their actual addresses.
        let check_required_pda1_pubkey = Pubkey::find_program_address(
            &[
                required_pda1_literal_string.as_bytes(),
                &instruction_u8array_arg,
                ix_account_infos.get(1).unwrap().key.as_ref(), // The second account
            ],
            &program_id,
        )
        .0;
        let check_required_pda2_pubkey = Pubkey::find_program_address(
            &[
                required_pda2_literal_u32.to_le_bytes().as_ref(),
                instruction_u64_arg.to_le_bytes().as_ref(),
                check_required_pda1_pubkey.as_ref(), // The first PDA should be at index 5
            ],
            &program_id,
        )
        .0;

        let mut lamports_pda1 = 0;
        let mut data_pda1 = [];
        let extra_pda_info1 = AccountInfo::new(
            &check_required_pda1_pubkey,
            false,
            true,
            &mut lamports_pda1,
            &mut data_pda1,
            &owner,
            false,
            Epoch::default(),
        );
        messed_account_infos.push(extra_pda_info1.clone());

        let mut lamports_pda2 = 0;
        let mut data_pda2 = [];
        let extra_pda_info2 = AccountInfo::new(
            &check_required_pda2_pubkey,
            false,
            true,
            &mut lamports_pda2,
            &mut data_pda2,
            &owner,
            false,
            Epoch::default(),
        );
        messed_account_infos.push(extra_pda_info2.clone());

        // Now throw in a few extras that might be just for our program.
        let pubkey4 = Pubkey::new_unique();
        let mut lamports4 = 0;
        let mut data4 = [];
        messed_account_infos.push(AccountInfo::new(
            &pubkey4,
            false,
            true,
            &mut lamports4,
            &mut data4,
            &owner,
            false,
            Epoch::default(),
        ));
        let pubkey5 = Pubkey::new_unique();
        let mut lamports5 = 0;
        let mut data5 = [];
        messed_account_infos.push(AccountInfo::new(
            &pubkey5,
            false,
            true,
            &mut lamports5,
            &mut data5,
            &owner,
            false,
            Epoch::default(),
        ));

        // Mess 'em up!
        messed_account_infos.swap(0, 4);
        messed_account_infos.swap(1, 2);
        messed_account_infos.swap(3, 4);

        // Perform the account resolution.
        let mut cpi_instruction =
            Instruction::new_with_bytes(program_id, &instruction_data, ix_accounts);
        let mut cpi_account_infos = ix_account_infos.to_vec();
        ExtraAccountMetaList::add_to_cpi_instruction::<TestInstruction>(
            &mut cpi_instruction,
            &mut cpi_account_infos,
            &buffer,
            &messed_account_infos,
        )
        .unwrap();

        // Our CPI instruction should match the check instruction.
        assert_eq!(cpi_instruction, instruction);

        // CPI account infos should have the instruction account infos
        // and the extra required account infos from the validation account,
        // and they should be in the correct order.
        let mut all_account_infos = ix_account_infos.to_vec();
        all_account_infos.extend(extra_account_infos.iter().cloned());
        all_account_infos.push(extra_pda_info1);
        all_account_infos.push(extra_pda_info2);

        assert_eq!(cpi_account_infos.len(), all_account_infos.len());
        for (a, b) in std::iter::zip(cpi_account_infos, all_account_infos) {
            assert_eq!(a.key, b.key);
            assert_eq!(a.is_signer, b.is_signer);
            assert_eq!(a.is_writable, b.is_writable);
        }
    }
}
