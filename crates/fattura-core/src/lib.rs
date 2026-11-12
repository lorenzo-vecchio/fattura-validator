//! Offline validator for Italian electronic invoices (*fatture elettroniche*).
//!
//! The crate validates what can be proven from the document alone — file shape,
//! XML well-formedness, XSD conformance, `SdI` content rules, identifier checksums —
//! and explicitly reports what it could not check.
//!
//! It never returns a boolean "valid": the result is a layered report. The design
//! plan lives in `.planning/PLAN.md` in the repository.
//!
//! # Architecture
//!
//! `fattura-core` is I/O-free, network-free and clock-injected so it can be
//! embedded in WASM, C ABI and language bindings without conditional compilation
//! of business logic. Callers inject the validation date and all bytes; the core
//! never opens a file and never reads the clock.
//!
//! The pipeline is one streaming pass over borrowed bytes:
//!
//! ```text
//! bytes → L0 sniff → L1 reader → typed model / schema walk / L3 rules → report
//! ```
//!
//! No DOM, no runtime XSD interpretation, no runtime regex, no locks, no globals.
//!
//! # Status
//!
//! The crate is under active construction. The public `validate` entry point
//! arrives with E9.1; the foundation milestones provide the diagnostics,
//! arithmetic and ingestion primitives it will compose.

#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]
#![deny(clippy::float_cmp)]

pub mod diag;
pub mod file;
pub mod id;
pub mod model;
pub mod num;
pub mod rules;
pub mod schema;
pub mod sign;
pub mod tables;
pub mod xml;

/// Crate version, as recorded in report metadata.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The JSON report contract version.
///
/// Independently versioned from the crate: a semver-minor crate release may
/// carry a schema bump when the report contract changes. Bumping it requires a
/// `CHANGELOG.md` note and refreshed `insta` snapshots.
pub const REPORT_SCHEMA_VERSION: &str = "1";
