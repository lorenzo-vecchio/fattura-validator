//! Typed errors for the numeric core.
//!
//! Both enums are allocation-free: they carry only integers and implement
//! [`core::fmt::Display`] and [`core::error::Error`]. Callers can therefore use
//! them in diagnostics without formatting a message first.

use core::error::Error;
use core::fmt;

/// A lexical or range failure while parsing an XSD decimal.
///
/// The variants mirror the constraints of `Amount8DecimalType`,
/// `Amount2DecimalType` and `RateType` (`P1 §7.1`): dot separator only, no
/// exponent, no thousands separators, no surrounding whitespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// The input is empty.
    Empty,
    /// A leading `+` was found; the XSD patterns have no plus sign.
    PlusSignNotAllowed,
    /// A leading `-` was found in a type whose pattern has no minus sign.
    NegativeNotAllowed,
    /// No digit precedes the decimal point.
    MissingIntegerPart,
    /// The integer part has more digits than the XSD pattern allows.
    IntegerPartTooLong {
        /// Number of digits found.
        digits: usize,
        /// Maximum allowed by the pattern.
        maximum: usize,
    },
    /// The input contains no decimal point.
    MissingDecimalPoint,
    /// The input contains more than one decimal point.
    MultipleDecimalPoints,
    /// A byte that is not an ASCII digit or the decimal point was found.
    InvalidCharacter {
        /// Byte offset in the original input.
        index: usize,
        /// The offending byte.
        byte: u8,
    },
    /// Fewer fractional digits than the pattern allows.
    FractionTooFew {
        /// Number of fractional digits found.
        digits: usize,
        /// Minimum allowed by the pattern.
        minimum: usize,
    },
    /// More fractional digits than the pattern allows.
    FractionTooMany {
        /// Number of fractional digits found.
        digits: usize,
        /// Maximum allowed by the pattern.
        maximum: usize,
    },
    /// The rate is numerically above `100.00` (`RateType` `maxInclusive`).
    RateOutOfRange,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty decimal"),
            Self::PlusSignNotAllowed => f.write_str("a leading '+' is not allowed"),
            Self::NegativeNotAllowed => f.write_str("a leading '-' is not allowed"),
            Self::MissingIntegerPart => f.write_str("missing integer part"),
            Self::IntegerPartTooLong { digits, maximum } => {
                write!(f, "integer part has {digits} digits, maximum is {maximum}")
            }
            Self::MissingDecimalPoint => f.write_str("missing decimal point"),
            Self::MultipleDecimalPoints => f.write_str("multiple decimal points"),
            Self::InvalidCharacter { index, byte } => {
                write!(f, "invalid byte 0x{byte:02x} at index {index}")
            }
            Self::FractionTooFew { digits, minimum } => {
                write!(f, "fraction has {digits} digits, minimum is {minimum}")
            }
            Self::FractionTooMany { digits, maximum } => {
                write!(f, "fraction has {digits} digits, maximum is {maximum}")
            }
            Self::RateOutOfRange => f.write_str("rate is above 100.00"),
        }
    }
}

impl Error for ParseError {}

/// A failure in exact arithmetic.
///
/// Every operation in this module is checked; overflow is reported here instead
/// of wrapping or panicking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticError {
    /// The result does not fit the `i128` representation.
    Overflow,
    /// A rational operation had a zero denominator.
    DivisionByZero,
    /// The requested number of decimal places is outside `0..=8`.
    InvalidScale {
        /// The requested decimal count.
        decimals: u8,
    },
    /// A value is outside the range accepted by its type.
    OutOfRange,
}

impl fmt::Display for ArithmeticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow => f.write_str("numeric overflow"),
            Self::DivisionByZero => f.write_str("division by zero"),
            Self::InvalidScale { decimals } => {
                write!(f, "invalid decimal scale {decimals}; expected 0..=8")
            }
            Self::OutOfRange => f.write_str("numeric value out of range"),
        }
    }
}

impl Error for ArithmeticError {}
