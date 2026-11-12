//! L2 — static XSD conformance.
//!
//! The official XSDs are compiled ahead of time into static tables: element
//! order, cardinality, sequence/choice, enumerations and simple-type facets.
//! There is no runtime XSD engine and no libxml2 (`ADR-0002`).
//!
//! # Stability
//!
//! Internal crate API. Generated data is versioned with the vendored XSDs.

// Filled in by E3.1–E3.5.
