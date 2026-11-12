//! Identifier validation: Partita IVA, Codice Fiscale, IBAN, BIC, CIG/CUP.
//!
//! Only offline-provable facts are checked — shape and checksums. Existence and
//! status in the Anagrafe tributaria are L4 and are disclosed as not checked.
//!
//! # Stability
//!
//! Internal crate API. Checksum functions are pure and deterministic.

// Filled in by E6.1–E6.5.
