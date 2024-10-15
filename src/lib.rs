//! # Skirt
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(feature = "nightly", feature(negative_impls))]

mod lazy_lock;
mod mutex;
mod once;
mod once_lock;
// mod rwlock;

/// Synchronization primitives that rely on spin-locking mechanisms.
pub mod sync;
