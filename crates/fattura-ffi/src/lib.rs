//! Stable C ABI for `fattura-core` (stub until E10.1).
//!
//! The ABI is additive-only and may use `unsafe`; all other workspace crates are
//! `forbid(unsafe_code)`.

/// Crate version exposed to C consumers.
pub const VERSION: &str = fattura_core::VERSION;
