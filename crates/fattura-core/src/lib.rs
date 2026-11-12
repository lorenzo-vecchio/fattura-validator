//! Offline validator for Italian electronic invoices (*fatture elettroniche*).
//!
//! The crate validates what can be proven from the document alone — file shape,
//! XML well-formedness, XSD conformance, `SdI` content rules, identifier checksums —
//! and explicitly reports what it could not check.
//!
//! # Layered report, never a boolean
//!
//! Validation is organised in layers, and each layer is reported separately:
//!
//! | Level | What it covers |
//! | --- | --- |
//! | **L0** | File: name shape, size, archive integrity, encoding, `.p7m` envelope |
//! | **L1** | XML well-formedness and declared-vs-actual encoding |
//! | **L2** | XSD conformance: order, cardinality, enumerations, patterns, ranges |
//! | **L3** | Offline `SdI` content rules (`004xx`) and documented warnings |
//! | **L4** | External registries — **not checked** (P.IVA/CF existence, IPA, VIES, …) |
//! | **L5** | Lifecycle — **not checked** (historical duplicates, delivery, notifications) |
//!
//! The result is a structured report, not a boolean "valid". It states which levels were
//! checked, which findings were produced, and which checks were skipped; skipped checks
//! are named by their official code. Every finding carries an official code when one
//! exists, a severity (`error`/`warning`/`info`), a level, an element path, a stable rule
//! id and a human message. Items marked `[?]` in the research sources are never emitted
//! as `error` until verified against a primary source.
//!
//! The library never repairs input and never claims more than local evidence supports.
//! In particular, signature status is `Unverifiable` — never `Valid` — when the caller has
//! not injected the trust and revocation data needed to prove otherwise.
//!
//! # Example
//!
//! ```
//! assert_eq!(fattura_core::VERSION, env!("CARGO_PKG_VERSION"));
//! assert_eq!(fattura_core::REPORT_SCHEMA_VERSION, "1");
//! ```
//!
//! # Target API
//!
//! The public `validate` entry point arrives with E9.1. The following sketch is
//! illustrative only and is **not compiled**:
//!
//! ```ignore
//! use fattura_core::{validate, Options};
//!
//! let report = validate(&bytes, &Options::default());
//!
//! for finding in &report.findings {
//!     eprintln!("{:?} {}: {}", finding.severity, finding.code, finding.message);
//! }
//!
//! // L4/L5 are disclosed, not silently treated as passing.
//! for skipped in &report.not_checked {
//!     eprintln!("not checked: {}", skipped.code);
//! }
//! ```
//!
//! # Architecture
//!
//! `fattura-core` is I/O-free, network-free and clock-injected so it can be
//! embedded in WASM, C ABI and language bindings without conditional compilation
//! of business logic. Callers inject the validation date and all bytes; the core
//! never opens a file, never reads the clock and never performs network I/O.
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
//! The crate is under active construction (milestone M1). The public `validate` entry
//! point arrives with E9.1; the foundation milestones provide the diagnostics,
//! arithmetic and ingestion primitives it will compose. The design record and work
//! breakdown live in `.planning/PLAN.md` in the repository.

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
