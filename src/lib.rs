//! Offline validator for Italian electronic invoices (*fatture elettroniche*).
//!
//! The crate validates what can be proven from the document alone — file shape,
//! XML well-formedness, XSD conformance, SdI content rules, identifier checksums —
//! and explicitly reports what it could not check.
//!
//! It never returns a boolean "valid": the result is a layered report.
//!
//! The design plan lives in `.planning/PLAN.md` in the repository.
//!
//! # Status
//!
//! The crate is a planning skeleton; the public API is not implemented yet.

/// Crate version, as recorded in the report metadata.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
