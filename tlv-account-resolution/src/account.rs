//! Struct for managing extra required account configurations
//!
//! For example, defining accounts required for your interface program, which
//! can be `AccountMeta`s - which have fixed addresses - or PDAs - which have
//! addresses derived from a collection of seeds

use {
    crate::{error::AccountResolutionError, pubkey_data::PubkeyData, seeds::Seed},
    bytemuck::{Pod, Zeroable},
    solana_account_info::AccountInfo,
    solana_instruction::AccountMeta,
    solana_program_error::ProgramError,
    solana_pubkey::{Pubkey, PUBKEY_BYTES},
    solana_zero_copy::unaligned::Bool,
};

/// Resolve a program-derived address (PDA) from the instruction data
/// and the accounts that have already been resolved
fn resolve_pda<'a, F>(
    seeds: &[Seed],
    instruction_data: &[u8],
    program_id: &Pubkey,
    get_account_key_data_fn: F,
) -> Result<Pubkey, ProgramError>
where
    F: Fn(usize) -> Option<(&'a Pubkey, Option<&'a [u8]>)>,
{
    let mut pda_seeds: Vec<&[u8]> = vec![];
    for config in seeds {
        match config {
            Seed::Uninitialized => (),
            Seed::Literal { bytes } => pda_seeds.push(bytes),
            Seed::InstructionData { index, length } => {
                let arg_start = *index as usize;
                let arg_end = arg_start + *length as usize;
                if arg_end > instruction_data.len() {
                    return Err(AccountResolutionError::InstructionDataTooSmall.into());
                }
                pda_seeds.push(&instruction_data[arg_start..arg_end]);
            }
            Seed::AccountKey { index } => {
                let account_index = *index as usize;
                let address = get_account_key_data_fn(account_index)
                    .ok_or::<ProgramError>(AccountResolutionError::AccountNotFound.into())?
                    .0;
                pda_seeds.push(address.as_ref());
            }
            Seed::AccountData {
                account_index,
                data_index,
                length,
            } => {
                let account_index = *account_index as usize;
                let account_data = get_account_key_data_fn(account_index)
                    .ok_or::<ProgramError>(AccountResolutionError::AccountNotFound.into())?
                    .1
                    .ok_or::<ProgramError>(AccountResolutionError::AccountDataNotFound.into())?;
                let arg_start = *data_index as usize;
                let arg_end = arg_start + *length as usize;
                if account_data.len() < arg_end {
                    return Err(AccountResolutionError::AccountDataTooSmall.into());
                }
                pda_seeds.push(&account_data[arg_start..arg_end]);
            }
        }
    }
    #[cfg(any(target_os = "solana", target_arch = "bpf"))]
    if let Some(address) = derive_pda(&pda_seeds, program_id) {
        return Ok(address);
    }
    Ok(Pubkey::find_program_address(&pda_seeds, program_id).0)
}

/// Derive the canonical PDA using the SHA-256 and curve validation syscalls.
/// Leave seed lists outside the accepted range to the standard SDK path.
#[cfg(any(target_os = "solana", target_arch = "bpf", test))]
fn derive_pda(seeds: &[&[u8]], program_id: &Pubkey) -> Option<Pubkey> {
    use solana_pubkey::{MAX_SEEDS, MAX_SEED_LEN};

    const PDA_MARKER: &[u8] = b"ProgramDerivedAddress";
    const BUMPS: [u8; u8::MAX as usize] = {
        let mut bumps = [0; u8::MAX as usize];
        let mut index = 0;
        while index < bumps.len() {
            bumps[index] = u8::MAX - index as u8;
            index += 1;
        }
        bumps
    };

    let len = seeds.len();
    // The bump occupies one seed slot.
    if len >= MAX_SEEDS || seeds.iter().any(|seed| seed.len() > MAX_SEED_LEN) {
        return None;
    }

    let mut parts: [&[u8]; MAX_SEEDS + 2] = [&[]; MAX_SEEDS + 2];
    parts[..len].copy_from_slice(seeds);
    parts[len + 1] = program_id.as_ref();
    parts[len + 2] = PDA_MARKER;
    let parts = &mut parts[..len + 3];

    // Try the same bumps as the runtime (255 through 1).
    for bump in &BUMPS {
        parts[len] = core::slice::from_ref(bump);
        let hash = solana_sha256_hasher::hashv(parts);
        let address = Pubkey::new_from_array(hash.to_bytes());
        if !address.is_on_curve() {
            return Some(address);
        }
    }
    panic!("Unable to find a viable program address bump seed");
}

/// Resolve a pubkey from a pubkey data configuration.
fn resolve_key_data<'a, F>(
    key_data: &PubkeyData,
    instruction_data: &[u8],
    get_account_key_data_fn: F,
) -> Result<Pubkey, ProgramError>
where
    F: Fn(usize) -> Option<(&'a Pubkey, Option<&'a [u8]>)>,
{
    match key_data {
        PubkeyData::Uninitialized => Err(ProgramError::InvalidAccountData),
        PubkeyData::InstructionData { index } => {
            let key_start = *index as usize;
            let key_end = key_start + PUBKEY_BYTES;
            if key_end > instruction_data.len() {
                return Err(AccountResolutionError::InstructionDataTooSmall.into());
            }
            Ok(Pubkey::new_from_array(
                instruction_data[key_start..key_end].try_into().unwrap(),
            ))
        }
        PubkeyData::AccountData {
            account_index,
            data_index,
        } => {
            let account_index = *account_index as usize;
            let account_data = get_account_key_data_fn(account_index)
                .ok_or::<ProgramError>(AccountResolutionError::AccountNotFound.into())?
                .1
                .ok_or::<ProgramError>(AccountResolutionError::AccountDataNotFound.into())?;
            let arg_start = *data_index as usize;
            let arg_end = arg_start + PUBKEY_BYTES;
            if account_data.len() < arg_end {
                return Err(AccountResolutionError::AccountDataTooSmall.into());
            }
            Ok(Pubkey::new_from_array(
                account_data[arg_start..arg_end].try_into().unwrap(),
            ))
        }
    }
}

/// `Pod` type for defining a required account in a validation account.
///
/// This can be any of the following:
///
/// * A standard `AccountMeta`
/// * A PDA (with seed configurations)
/// * A pubkey stored in some data (account or instruction data)
///
/// Can be used in TLV-encoded data.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct ExtraAccountMeta {
    /// Discriminator to tell whether this represents a standard
    /// `AccountMeta`, PDA, or pubkey data.
    pub discriminator: u8,
    /// This `address_config` field can either be the pubkey of the account,
    /// the seeds used to derive the pubkey from provided inputs (PDA), or the
    /// data used to derive the pubkey (account or instruction data).
    pub address_config: [u8; 32],
    /// Whether the account should sign
    pub is_signer: Bool,
    /// Whether the account should be writable
    pub is_writable: Bool,
}
/// Helper used to know when the top bit is set, to interpret the
/// discriminator as an index rather than as a type
const U8_TOP_BIT: u8 = 1 << 7;
impl ExtraAccountMeta {
    /// Create a `ExtraAccountMeta` from a public key,
    /// thus representing a standard `AccountMeta`
    pub fn new_with_pubkey(
        pubkey: &Pubkey,
        is_signer: bool,
        is_writable: bool,
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            discriminator: 0,
            address_config: pubkey.to_bytes(),
            is_signer: is_signer.into(),
            is_writable: is_writable.into(),
        })
    }

    /// Create a `ExtraAccountMeta` from a list of seed configurations,
    /// thus representing a PDA
    pub fn new_with_seeds(
        seeds: &[Seed],
        is_signer: bool,
        is_writable: bool,
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            discriminator: 1,
            address_config: Seed::pack_into_address_config(seeds)?,
            is_signer: is_signer.into(),
            is_writable: is_writable.into(),
        })
    }

    /// Create a `ExtraAccountMeta` from a pubkey data configuration.
    pub fn new_with_pubkey_data(
        key_data: &PubkeyData,
        is_signer: bool,
        is_writable: bool,
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            discriminator: 2,
            address_config: PubkeyData::pack_into_address_config(key_data)?,
            is_signer: is_signer.into(),
            is_writable: is_writable.into(),
        })
    }

    /// Create a `ExtraAccountMeta` from a list of seed configurations,
    /// representing a PDA for an external program
    ///
    /// This PDA belongs to a program elsewhere in the account list, rather
    /// than the executing program. For a PDA on the executing program, use
    /// `ExtraAccountMeta::new_with_seeds`.
    pub fn new_external_pda_with_seeds(
        program_index: u8,
        seeds: &[Seed],
        is_signer: bool,
        is_writable: bool,
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            discriminator: program_index
                .checked_add(U8_TOP_BIT)
                .ok_or(AccountResolutionError::InvalidSeedConfig)?,
            address_config: Seed::pack_into_address_config(seeds)?,
            is_signer: is_signer.into(),
            is_writable: is_writable.into(),
        })
    }

    /// Resolve an `ExtraAccountMeta` into an `AccountMeta`, potentially
    /// resolving a program-derived address (PDA) if necessary
    pub fn resolve<'a, F>(
        &self,
        instruction_data: &[u8],
        program_id: &Pubkey,
        get_account_key_data_fn: F,
    ) -> Result<AccountMeta, ProgramError>
    where
        F: Fn(usize) -> Option<(&'a Pubkey, Option<&'a [u8]>)>,
    {
        match self.discriminator {
            0 => AccountMeta::try_from(self),
            x if x == 1 || x >= U8_TOP_BIT => {
                let program_id = if x == 1 {
                    program_id
                } else {
                    get_account_key_data_fn(x.saturating_sub(U8_TOP_BIT) as usize)
                        .ok_or::<ProgramError>(AccountResolutionError::AccountNotFound.into())?
                        .0
                };
                let seeds = Seed::unpack_address_config(&self.address_config)?;
                Ok(AccountMeta {
                    pubkey: resolve_pda(
                        &seeds,
                        instruction_data,
                        program_id,
                        get_account_key_data_fn,
                    )?,
                    is_signer: self.is_signer.into(),
                    is_writable: self.is_writable.into(),
                })
            }
            2 => {
                let key_data = PubkeyData::unpack(&self.address_config)?;
                Ok(AccountMeta {
                    pubkey: resolve_key_data(&key_data, instruction_data, get_account_key_data_fn)?,
                    is_signer: self.is_signer.into(),
                    is_writable: self.is_writable.into(),
                })
            }
            _ => Err(ProgramError::InvalidAccountData),
        }
    }
}

impl From<&AccountMeta> for ExtraAccountMeta {
    fn from(meta: &AccountMeta) -> Self {
        Self {
            discriminator: 0,
            address_config: meta.pubkey.to_bytes(),
            is_signer: meta.is_signer.into(),
            is_writable: meta.is_writable.into(),
        }
    }
}
impl From<AccountMeta> for ExtraAccountMeta {
    fn from(meta: AccountMeta) -> Self {
        ExtraAccountMeta::from(&meta)
    }
}
impl From<&AccountInfo<'_>> for ExtraAccountMeta {
    fn from(account_info: &AccountInfo) -> Self {
        Self {
            discriminator: 0,
            address_config: account_info.key.to_bytes(),
            is_signer: account_info.is_signer.into(),
            is_writable: account_info.is_writable.into(),
        }
    }
}
impl From<AccountInfo<'_>> for ExtraAccountMeta {
    fn from(account_info: AccountInfo) -> Self {
        ExtraAccountMeta::from(&account_info)
    }
}

impl TryFrom<&ExtraAccountMeta> for AccountMeta {
    type Error = ProgramError;

    fn try_from(pod: &ExtraAccountMeta) -> Result<Self, Self::Error> {
        if pod.discriminator == 0 {
            Ok(AccountMeta {
                pubkey: Pubkey::from(pod.address_config),
                is_signer: pod.is_signer.into(),
                is_writable: pod.is_writable.into(),
            })
        } else {
            Err(AccountResolutionError::AccountTypeNotAccountMeta.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        solana_pubkey::{MAX_SEEDS, MAX_SEED_LEN},
    };

    #[test]
    fn canonical_pda_matches_sdk() {
        for case in 0..128u8 {
            let program_id = Pubkey::new_from_array(solana_sha256_hasher::hash(&[case]).to_bytes());
            let seed_data: [[u8; MAX_SEED_LEN]; MAX_SEEDS - 1] = core::array::from_fn(|index| {
                solana_sha256_hasher::hash(&[case, index as u8]).to_bytes()
            });
            let seeds: [&[u8]; MAX_SEEDS - 1] = core::array::from_fn(|index| {
                let len = (usize::from(case) + index) % (MAX_SEED_LEN + 1);
                &seed_data[index][..len]
            });
            let seeds = &seeds[..usize::from(case) % MAX_SEEDS];
            assert_eq!(
                derive_pda(seeds, &program_id),
                Pubkey::try_find_program_address(seeds, &program_id).map(|(address, _)| address),
                "case {case}"
            );
        }
    }

    #[test]
    fn out_of_range_seeds_use_the_sdk_path() {
        let program_id = Pubkey::new_unique();
        let key = Pubkey::new_unique();
        let data = [0; MAX_SEED_LEN + 1];
        let oversized: &[&[Seed]] = &[
            &[const { Seed::AccountKey { index: 0 } }; MAX_SEEDS],
            &[Seed::InstructionData {
                index: 0,
                length: data.len() as u8,
            }],
            &[Seed::AccountData {
                account_index: 0,
                data_index: 0,
                length: data.len() as u8,
            }],
        ];
        assert_eq!(derive_pda(&[key.as_ref(); MAX_SEEDS], &program_id), None);
        assert_eq!(derive_pda(&[&data], &program_id), None);
        for seeds in oversized {
            let meta = ExtraAccountMeta::new_with_seeds(seeds, false, false).unwrap();
            let result = std::panic::catch_unwind(|| {
                meta.resolve(&data, &program_id, |_| Some((&key, Some(&data))))
            });
            assert!(result.is_err(), "seeds: {seeds:?}");
        }
    }

    #[test]
    fn pda_seed_errors_are_preserved() {
        let program_id = Pubkey::new_unique();
        let key = Pubkey::new_unique();
        let data = [42];
        let resolve = |index| match index {
            0 => Some((&key, Some(data.as_slice()))),
            1 => Some((&key, None)),
            _ => None,
        };
        let cases = [
            (
                Seed::InstructionData {
                    index: 1,
                    length: 1,
                },
                AccountResolutionError::InstructionDataTooSmall,
            ),
            (
                Seed::AccountKey { index: 2 },
                AccountResolutionError::AccountNotFound,
            ),
            (
                Seed::AccountData {
                    account_index: 2,
                    data_index: 0,
                    length: 1,
                },
                AccountResolutionError::AccountNotFound,
            ),
            (
                Seed::AccountData {
                    account_index: 1,
                    data_index: 0,
                    length: 1,
                },
                AccountResolutionError::AccountDataNotFound,
            ),
            (
                Seed::AccountData {
                    account_index: 0,
                    data_index: 1,
                    length: 1,
                },
                AccountResolutionError::AccountDataTooSmall,
            ),
        ];
        for (seed, error) in cases {
            let meta = ExtraAccountMeta::new_with_seeds(&[seed], false, false).unwrap();
            assert_eq!(meta.resolve(&data, &program_id, resolve), Err(error.into()));
        }

        let mut meta = ExtraAccountMeta::new_external_pda_with_seeds(2, &[], false, false).unwrap();
        assert_eq!(
            meta.resolve(&data, &program_id, resolve),
            Err(AccountResolutionError::AccountNotFound.into())
        );
        meta.discriminator = 1;
        meta.address_config[0] = u8::MAX;
        assert_eq!(
            meta.resolve(&data, &program_id, resolve),
            Err(ProgramError::InvalidAccountData)
        );
        meta.address_config = [1; 32];
        assert_eq!(
            meta.resolve(&data, &program_id, resolve),
            Err(AccountResolutionError::InvalidBytesForSeed.into())
        );
    }
}
