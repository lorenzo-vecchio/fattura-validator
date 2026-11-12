//! Exact money and percentage arithmetic.
//!
//! `Amount` is a signed scale-8 `i128` fixed-point value; `Rate` is an
//! unsigned scale-2 percentage. Parsing is strict — dot decimal separator, no
//! exponent, no thousands separators, no whitespace — and mirrors the XSD
//! lexical forms (`P1 §7.1`). Rounding is explicit and half-up; there is no
//! global epsilon and no floating point anywhere.
//!
//! # Types
//!
//! * [`Amount`] — `Amount8DecimalType` / `Amount2DecimalType` values, stored as
//!   `i128` scaled by `10^8`.
//! * [`Rate`] — `RateType`, exactly two decimals in `0.00..=100.00`.
//! * [`Rational`] — exact numerator/denominator arithmetic for percentage
//!   application and cascading discounts, rounded only when the caller asks.
//!
//! # Error handling
//!
//! Parsing returns [`ParseError`]; checked arithmetic returns
//! [`ArithmeticError`]. Both are allocation-free and implement
//! [`core::error::Error`]. No operation panics on user input.
//!
//! # Stability
//!
//! Numeric semantics are part of the validated result: changing rounding or
//! tolerance behaviour requires a changelog note and new property tests.

#![deny(clippy::float_arithmetic)]
#![deny(clippy::float_cmp)]

mod amount;
mod error;
mod parse;
mod power;
mod rate;
mod rational;

pub use amount::{Amount, AmountDisplay, SCALE, SCALE_FACTOR};
pub use error::{ArithmeticError, ParseError};
pub use rate::Rate;
pub use rational::Rational;
