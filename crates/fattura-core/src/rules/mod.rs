//! L3 — offline `SdI` content rules (`004xx`).
//!
//! One module per rule group (`amounts`, `natura`, `riepilogo`, `document`,
//! `withholding`, `transmission`, `duplicates`, `recommended`). Every rule cites
//! its official code and source section in rustdoc.
//!
//! Findings marked `[?]` in the research are warnings, never errors.
//!
//! # Stability
//!
//! Internal crate API. Rule ids (`ade.00423`, `rec.e3`) are stable once released.

#![deny(clippy::float_arithmetic)]
#![deny(clippy::float_cmp)]

// Filled in by E5.1–E5.9.
