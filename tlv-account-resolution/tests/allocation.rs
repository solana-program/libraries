#![allow(clippy::arithmetic_side_effects)]

//! Guards against reintroducing per-meta list rebuilds during account
//! resolution, which grow heap usage quadratically under Solana's
//! never-freeing SBF bump allocator.

use {
    solana_account_info::AccountInfo,
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
    spl_discriminator::{ArrayDiscriminator, SplDiscriminate},
    spl_tlv_account_resolution::{account::ExtraAccountMeta, state::ExtraAccountMetaList},
    std::{
        alloc::{GlobalAlloc, Layout, System},
        sync::atomic::{AtomicUsize, Ordering},
    },
};

struct CountingAllocator;

static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATED_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        // SAFETY: forwards the caller's layout unchanged to the system
        // allocator, adding only an atomic counter update
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: forwards a pointer and layout produced by the matching
        // `alloc` above to the system allocator
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

struct TestInstruction;
impl SplDiscriminate for TestInstruction {
    const SPL_DISCRIMINATOR: ArrayDiscriminator =
        ArrayDiscriminator::new([1; ArrayDiscriminator::LENGTH]);
}

fn cpi_resolution_allocated_bytes(extra_meta_count: usize) -> usize {
    let program_id = Pubkey::new_unique();
    let owner = Pubkey::new_unique();

    let keys: Vec<Pubkey> = (0..extra_meta_count + 2)
        .map(|_| Pubkey::new_unique())
        .collect();
    let mut lamports: Vec<u64> = vec![0; keys.len()];
    let mut datas: Vec<Vec<u8>> = vec![Vec::new(); keys.len()];

    let account_infos: Vec<AccountInfo> = keys
        .iter()
        .zip(lamports.iter_mut())
        .zip(datas.iter_mut())
        .map(|((key, lamports), data)| {
            AccountInfo::new(key, false, false, lamports, data, &owner, false)
        })
        .collect();

    let extra_metas: Vec<ExtraAccountMeta> = keys[2..]
        .iter()
        .map(|key| ExtraAccountMeta::from(&AccountMeta::new_readonly(*key, false)))
        .collect();
    let mut buffer = vec![0; ExtraAccountMetaList::size_of(extra_metas.len()).unwrap()];
    ExtraAccountMetaList::init::<TestInstruction>(&mut buffer, &extra_metas).unwrap();

    let instruction_accounts = vec![
        AccountMeta::new_readonly(keys[0], false),
        AccountMeta::new_readonly(keys[1], false),
    ];
    let mut cpi_instruction = Instruction::new_with_bytes(program_id, &[], instruction_accounts);
    let mut cpi_account_infos = account_infos[..2].to_vec();

    let before = ALLOCATED_BYTES.load(Ordering::Relaxed);
    ExtraAccountMetaList::add_to_cpi_instruction::<TestInstruction>(
        &mut cpi_instruction,
        &mut cpi_account_infos,
        &buffer,
        &account_infos,
    )
    .unwrap();
    let after = ALLOCATED_BYTES.load(Ordering::Relaxed);

    assert_eq!(cpi_instruction.accounts.len(), keys.len());
    assert_eq!(cpi_account_infos.len(), keys.len());

    after - before
}

fn check_resolution_allocated_bytes(extra_meta_count: usize) -> usize {
    let program_id = Pubkey::new_unique();
    let owner = Pubkey::new_unique();

    let keys: Vec<Pubkey> = (0..extra_meta_count + 2)
        .map(|_| Pubkey::new_unique())
        .collect();
    let mut lamports: Vec<u64> = vec![0; keys.len()];
    let mut datas: Vec<Vec<u8>> = vec![Vec::new(); keys.len()];

    let account_infos: Vec<AccountInfo> = keys
        .iter()
        .zip(lamports.iter_mut())
        .zip(datas.iter_mut())
        .map(|((key, lamports), data)| {
            AccountInfo::new(key, false, false, lamports, data, &owner, false)
        })
        .collect();

    let extra_metas: Vec<ExtraAccountMeta> = keys[2..]
        .iter()
        .map(|key| ExtraAccountMeta::from(&AccountMeta::new_readonly(*key, false)))
        .collect();
    let mut buffer = vec![0; ExtraAccountMetaList::size_of(extra_metas.len()).unwrap()];
    ExtraAccountMetaList::init::<TestInstruction>(&mut buffer, &extra_metas).unwrap();

    let before = ALLOCATED_BYTES.load(Ordering::Relaxed);
    ExtraAccountMetaList::check_account_infos::<TestInstruction>(
        &account_infos,
        &[],
        &program_id,
        &buffer,
    )
    .unwrap();
    let after = ALLOCATED_BYTES.load(Ordering::Relaxed);

    after - before
}

fn assert_linear_growth(label: &str, small: usize, medium: usize, large: usize) {
    let first_growth = medium.saturating_sub(small);
    let second_growth = large.saturating_sub(medium);

    assert!(
        second_growth <= first_growth.saturating_mul(2).saturating_add(small),
        "{label} allocation growth accelerated: 8 metas allocated {small} bytes, 16 allocated \
         {medium}, and 32 allocated {large}; this indicates per-meta buffer rebuilds"
    );
}

#[test]
fn resolution_allocations_scale_linearly_with_meta_count() {
    assert_linear_growth(
        "add_to_cpi_instruction",
        cpi_resolution_allocated_bytes(8),
        cpi_resolution_allocated_bytes(16),
        cpi_resolution_allocated_bytes(32),
    );
    assert_linear_growth(
        "check_account_infos",
        check_resolution_allocated_bytes(8),
        check_resolution_allocated_bytes(16),
        check_resolution_allocated_bytes(32),
    );
}
