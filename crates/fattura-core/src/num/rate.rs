//! Unsigned exact percentage rate.
//!
//! [`Rate`] stores hundredths of a percent in a `u32`, matching `RateType`
//! (`P1 §7.1`): `[0-9]{1,3}\.[0-9]{2}` with `maxInclusive="100.00"`. The minus
//! sign is not part of the pattern.

use core::fmt;
use core::str::FromStr;

use super::error::{ArithmeticError, ParseError};
use super::parse::{accumulate_digits, parse_decimal};

/// One hundred percent expressed in hundredths of a percent.
const MAX_RAW: u32 = 10_000;

/// Maximum integer digits accepted by `RateType`.
const MAX_INTEGER_DIGITS: usize = 3;

/// An unsigned percentage with exactly two decimal places.
///
/// The representation is hundredths of one percent: `8.50%` is stored as
/// `850`. The valid range is `0.00..=100.00`; `100.01` and larger are rejected,
/// while `1.00` is accepted (`P3 §8` item 13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Rate(u32);

impl Rate {
    /// The `0.00%` rate.
    pub const ZERO: Self = Self(0);

    /// The `100.00%` rate.
    pub const MAX: Self = Self(MAX_RAW);

    /// Builds a rate from hundredths of a percent.
    ///
    /// Rejects values above `100.00` (`RateType` `maxInclusive`, `P1 §7.1`)
    /// with [`ArithmeticError::OutOfRange`].
    pub fn from_raw(raw: u32) -> Result<Self, ArithmeticError> {
        if raw > MAX_RAW {
            return Err(ArithmeticError::OutOfRange);
        }
        Ok(Self(raw))
    }

    /// Returns the rate in hundredths of a percent (`RateType`, `P1 §7.1`).
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Parses `RateType`, `[0-9]{1,3}\.[0-9]{2}` with `maxInclusive="100.00"`
    /// (`P1 §7.1`).
    ///
    /// Exactly two fractional digits are required and no leading `-` or `+` is
    /// accepted.
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let parts = parse_decimal(input, false, MAX_INTEGER_DIGITS, 2, 2)?;
        let integer = u32::try_from(accumulate_digits(parts.integer)).unwrap_or(MAX_RAW);
        let fraction = u32::try_from(accumulate_digits(parts.fraction)).unwrap_or(0);
        let raw = integer.saturating_mul(100).saturating_add(fraction);
        Self::from_raw(raw).map_err(|_| ParseError::RateOutOfRange)
    }

    /// Converts the percentage to an exact rational fraction.
    ///
    /// `8.50%` becomes `850 / 10000`, reduced to `17 / 200`. The conversion is
    /// infallible because [`Rate`] is capped at `100.00` by `RateType`
    /// (`P1 §7.1`).
    pub fn as_rational(self) -> super::Rational {
        super::Rational::from_hundredths_of_percent(self.0)
    }
}

impl FromStr for Rate {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse(input)
    }
}

impl fmt::Display for Rate {
    /// Renders exactly two fractional digits, as required by `RateType`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let integer = self.0 / 100;
        let fraction = self.0 % 100;
        write!(f, "{integer}.{fraction:02}")
    }
}
