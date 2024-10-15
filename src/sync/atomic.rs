#[cfg(not(feature = "portable"))]
pub use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
#[cfg(feature = "portable")]
pub use portable_atomic::{AtomicBool, AtomicU8, Ordering};
