//! Serialization-aware collection wrappers for Solana account data.
//!
//! This crate provides wrappers around collection types to support custom serialization
//! logic. This is useful for programs that have specific requirements for how data is
//! stored.

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
mod string;
#[cfg(feature = "alloc")]
mod vec;

#[cfg(feature = "alloc")]
pub use string::*;
#[cfg(feature = "alloc")]
pub use vec::*;
