//! Exact money and percentage arithmetic.
//!
//! `Amount` is a signed scale-8 `i128` fixed-point value; `Rate` is an
//! unsigned scale-2 percentage. Parsing is strict — dot decimal separator, no
//! exponent, no thousands separators, no whitespace — and mirrors the XSD
//! lexical forms (`P1 §7.1`). Rounding is explicit and half-up; there is no
//! global epsilon and no floating point anywhere.
//!
//! # Stability
//!
//! Numeric semantics are part of the validated result: changing rounding or
//! tolerance behaviour requires a changelog note and new property tests.

#![deny(clippy::float_arithmetic)]
#![deny(clippy::float_cmp)]

// Filled in by E1.3.
