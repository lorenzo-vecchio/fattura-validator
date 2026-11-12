//! Signature verification (feature `signature`).
//!
//! `XAdES` and `CAdES` integrity, certificate validity windows and a local,
//! versioned trust store. Without revocation data the status is `Unverifiable`,
//! never `Valid`.
//!
//! # Stability
//!
//! Internal crate API, feature-gated; the status model is public via the report.

// Filled in by E8.1–E8.4.
