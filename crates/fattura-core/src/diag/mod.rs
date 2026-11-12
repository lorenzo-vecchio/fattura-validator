//! Diagnostic vocabulary shared by every validation layer.
//!
//! This module owns the stable types of the public contract: `Finding`,
//! `SdiCode`, `Level`, `Severity`, `RuleId`, `ElementPath` and the layered
//! `Report`. Every other module speaks these types; none may invent a private
//! error vocabulary.
//!
//! # Stability
//!
//! The JSON projection of `Report` is the cross-language contract. The Rust
//! types may gain fields in a minor release only when the schema version is
//! bumped and snapshots are updated.

// Filled in by E1.2.
