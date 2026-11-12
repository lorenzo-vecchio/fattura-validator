//! L1 — streaming XML ingestion.
//!
//! One pass over borrowed bytes, dispatching on namespace-agnostic local names.
//! Encoding declaration is checked against the actual bytes; BOM, processing
//! instructions and comments are tolerated (`P3 §8`, plumbing items 1–6).
//!
//! No DOM is built on the validation path.
//!
//! # Stability
//!
//! Internal crate API. The event shape may change until E9.1 freezes the public
//! API; callers outside the crate must use `validate` instead.

// Filled in by E2.1, E2.5 and E7.5.
