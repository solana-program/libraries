//! Serialization-aware collection wrappers for Solana account data.
//!
//! This crate provides wrappers around collection types to support custom serialization
//! logic. This is useful for programs that have specific requirements for how data is
//! stored.

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

extern crate alloc;

mod str;
mod vec;

pub use str::*;
pub use vec::*;
