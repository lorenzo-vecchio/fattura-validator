//! Signed exact monetary amount.
//!
//! [`Amount`] stores an `i128` scaled by `10^8`, so every value representable by
//! `Amount8DecimalType` and `Amount2DecimalType` (`P1 §7.1`) is exact. No
//! binary floating point is involved at any point.

use core::fmt;
use core::str::FromStr;

use super::error::{ArithmeticError, ParseError};
use super::parse::{accumulate_digits, parse_decimal};
use super::power::pow10;
use super::rate::Rate;
use super::rational::Rational;

/// Number of decimal places stored by [`Amount`].
pub const SCALE: u8 = 8;

/// `10^SCALE`, the number of scaled units in one euro.
pub const SCALE_FACTOR: i128 = 100_000_000;

/// Number of scaled units in one euro, as an unsigned value for formatting.
const SCALE_FACTOR_U128: u128 = 100_000_000;

/// Maximum number of integer digits accepted by the XSD amount patterns.
const MAX_INTEGER_DIGITS: usize = 11;

/// A signed exact amount with eight decimal places.
///
/// The internal unit is `10^-8`; for example `12.34` is stored as
/// `1_234_000_000`. The value is exact for both `Amount8DecimalType`
/// (`[\-]?[0-9]{1,11}\.[0-9]{2,8}`) and `Amount2DecimalType`
/// (`[\-]?[0-9]{1,11}\.[0-9]{2}`) from `P1 §7.1`; parsing keeps the XSD
/// integer-digit limit, while arithmetic uses the full `i128` range and reports
/// overflow through [`ArithmeticError::Overflow`].
///
/// `Amount` is `Copy`, `Eq` and `Hash`: aggregation keys use the exact scaled
/// integer returned by [`Amount::scaled`], not a formatted string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Amount(i128);

impl Amount {
    /// The zero amount.
    pub const ZERO: Self = Self(0);

    /// Builds an amount from its `10^-8` scaled representation.
    ///
    /// This constructor is intentionally unrestricted so aggregation code can
    /// represent intermediate totals beyond the XSD lexical range. Use
    /// [`Amount::parse_amount8`] or [`Amount::parse_amount2`] for input that
    /// must obey the eleven-integer-digit limit.
    #[must_use]
    pub const fn from_scaled(scaled: i128) -> Self {
        Self(scaled)
    }

    /// Returns the `10^-8` scaled representation.
    #[must_use]
    pub const fn scaled(self) -> i128 {
        self.0
    }

    /// Parses `Amount8DecimalType`, `[\-]?[0-9]{1,11}\.[0-9]{2,8}`
    /// (`P1 §7.1`).
    ///
    /// The separator is a dot; a comma, exponent, thousands separator or any
    /// surrounding whitespace is rejected. Leading zeros are accepted. `-0.00`
    /// is accepted and normalises to zero.
    pub fn parse_amount8(input: &str) -> Result<Self, ParseError> {
        Self::parse(input, 2, 8)
    }

    /// Parses `Amount2DecimalType`, `[\-]?[0-9]{1,11}\.[0-9]{2}`
    /// (`P1 §7.1`).
    ///
    /// Exactly two fractional digits are required; three or more are rejected
    /// even though [`Amount::parse_amount8`] would accept them.
    pub fn parse_amount2(input: &str) -> Result<Self, ParseError> {
        Self::parse(input, 2, 2)
    }

    fn parse(input: &str, min_fraction: usize, max_fraction: usize) -> Result<Self, ParseError> {
        let parts = parse_decimal(input, true, MAX_INTEGER_DIGITS, min_fraction, max_fraction)?;
        let integer = accumulate_digits(parts.integer);
        let fraction = accumulate_digits(parts.fraction);
        let padding = SCALE.saturating_sub(u8::try_from(parts.fraction.len()).unwrap_or(0));
        let magnitude = integer
            .saturating_mul(SCALE_FACTOR)
            .saturating_add(fraction.saturating_mul(pow10(padding)));
        // `padding <= 8` and the combined digit run is at most 19 digits, so
        // `10^8` is the largest power used. The `saturating_*` calls above can
        // only saturate for inputs the length checks already rejected.
        let scaled = if parts.negative {
            magnitude.saturating_neg()
        } else {
            magnitude
        };
        Ok(Self(scaled))
    }

    /// Checked addition for the `00422`/`00423` aggregations (`P3 §3.4`–`§3.5`).
    pub fn checked_add(self, rhs: Self) -> Result<Self, ArithmeticError> {
        self.0
            .checked_add(rhs.0)
            .map(Self)
            .ok_or(ArithmeticError::Overflow)
    }

    /// Checked subtraction for invoice totals (`P3 §3.4`–`§3.5`).
    pub fn checked_sub(self, rhs: Self) -> Result<Self, ArithmeticError> {
        self.0
            .checked_sub(rhs.0)
            .map(Self)
            .ok_or(ArithmeticError::Overflow)
    }

    /// Checked negation. `-(-i128::MIN)` is reported as overflow.
    ///
    /// Used when sign-adjusting `Arrotondamento` in `00422` (`P3 §3.4`).
    pub fn checked_neg(self) -> Result<Self, ArithmeticError> {
        self.0
            .checked_neg()
            .map(Self)
            .ok_or(ArithmeticError::Overflow)
    }

    /// Checked multiplication by an integer factor.
    ///
    /// The factor is an exact integer, not a fixed-point amount: multiplying
    /// `500.00` by `100` yields `50000.00` exactly, as in the `00423` quantity
    /// example (`P3 §3.5`).
    pub fn checked_mul_i128(self, factor: i128) -> Result<Self, ArithmeticError> {
        self.0
            .checked_mul(factor)
            .map(Self)
            .ok_or(ArithmeticError::Overflow)
    }

    /// Checked absolute value, used by the `00421`–`00423` tolerance tests
    /// (`P3 §3.3`–`§3.5`).
    pub fn checked_abs(self) -> Result<Self, ArithmeticError> {
        self.0
            .checked_abs()
            .map(Self)
            .ok_or(ArithmeticError::Overflow)
    }

    /// Absolute difference between two amounts (`P3 §3.3`–`§3.5`).
    pub fn abs_diff(self, other: Self) -> Result<Self, ArithmeticError> {
        self.checked_sub(other)?.checked_abs()
    }

    /// Returns `true` when the difference is strictly below `tolerance`.
    ///
    /// This is the comparison for `00421`/`00423` (`|difference| >= 0.01` is an
    /// error, `P3 §3.3`, `P3 §3.5`) and for `00422` (`>= 1.00`, `P3 §3.4`).
    /// The tolerance is supplied by each call site; this module defines no
    /// global epsilon (`P3 §8` item 11).
    ///
    /// A negative tolerance makes every comparison `false`.
    pub fn within_tolerance(self, other: Self, tolerance: Self) -> Result<bool, ArithmeticError> {
        Ok(self.abs_diff(other)?.0 < tolerance.0)
    }

    /// Returns `true` when the difference is at or above `tolerance`.
    ///
    /// This is the error predicate used by `00421`, `00422` and `00423`
    /// (`P3 §3.3`–`§3.5`).
    pub fn exceeds_tolerance(self, other: Self, tolerance: Self) -> Result<bool, ArithmeticError> {
        Ok(self.abs_diff(other)?.0 >= tolerance.0)
    }

    /// Rounds half-up to `decimals` decimal places.
    ///
    /// Half-up means half away from zero on the magnitude: `1500.2458` becomes
    /// `1500.25`, `1500.2448` becomes `1500.24`, and `-0.005` becomes `-0.01`
    /// (`P3 §3.3`). `decimals` must be in `0..=8`.
    // The divisor is a validated non-zero power of ten and the remainder is
    // bounded by it, so these divisions cannot trap.
    #[allow(clippy::arithmetic_side_effects)]
    pub fn round_to(self, decimals: u8) -> Result<Self, ArithmeticError> {
        if decimals > SCALE {
            return Err(ArithmeticError::InvalidScale { decimals });
        }
        let dropped = SCALE.saturating_sub(decimals);
        if dropped == 0 {
            return Ok(self);
        }
        let factor = pow10(dropped);
        let quotient = self.0 / factor;
        let remainder = self.0 % factor;
        let half = u128::try_from(factor / 2).unwrap_or(u128::MAX);
        // `factor >= 10`, so `|quotient| <= i128::MAX / 10`; the saturation
        // below is therefore unreachable and only avoids an arithmetic lint.
        let rounded = if remainder.unsigned_abs() >= half {
            // The remainder is non-zero here (`half >= 5`), so `signum` is
            // exactly ±1 and moves the magnitude away from zero.
            quotient.saturating_add(remainder.signum())
        } else {
            quotient
        };
        rounded
            .checked_mul(factor)
            .map(Self)
            .ok_or(ArithmeticError::Overflow)
    }

    /// Converts the amount to an exact rational number of euros.
    pub fn as_rational(self) -> Result<Rational, ArithmeticError> {
        Rational::new(self.0, SCALE_FACTOR)
    }

    /// Applies a percentage exactly, without rounding.
    ///
    /// `500.00` at `8.50%` yields the rational `42.5`; rounding is a separate,
    /// caller-chosen step via [`Rational::round_to`] (`P3 §3.3`).
    pub fn apply_percentage(self, rate: Rate) -> Result<Rational, ArithmeticError> {
        self.as_rational()?.checked_mul(rate.as_rational())
    }

    /// Applies percentage discounts in cascade, exactly and without rounding.
    ///
    /// Each percentage applies to the price already discounted by the previous
    /// block, as required for `00423` (`P3 §3.5`, rule 2). This computes the
    /// discounted unit price; multiply by the quantity with [`Rational`]
    /// arithmetic and round once at the caller's chosen point.
    pub fn apply_cascade(self, rates: &[Rate]) -> Result<Rational, ArithmeticError> {
        self.as_rational()?.apply_cascade(rates)
    }

    /// Builds a display adapter that renders exactly `decimals` decimal places.
    ///
    /// The value is first rounded half-up to the requested scale; the adapter
    /// can therefore never hide non-zero digits. `decimals` must be in
    /// `0..=8`.
    pub fn display_with(self, decimals: u8) -> Result<AmountDisplay, ArithmeticError> {
        Ok(AmountDisplay {
            value: self.round_to(decimals)?,
            decimals,
        })
    }
}

impl FromStr for Amount {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse_amount8(input)
    }
}

impl fmt::Display for Amount {
    /// Renders the canonical amount: at least two and at most eight fractional
    /// digits, with trailing zeros beyond the second removed.
    ///
    /// Canonical values round-trip through [`Amount::parse_amount8`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_scaled(f, self.0, None)
    }
}

/// A display adapter that renders an [`Amount`] with an exact number of
/// decimal places.
///
/// Created by [`Amount::display_with`]; the value has already been rounded
/// half-up to the requested scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AmountDisplay {
    value: Amount,
    decimals: u8,
}

impl AmountDisplay {
    /// Returns the rounded value being displayed (`P1 §7.1` amount types).
    #[must_use]
    pub const fn amount(self) -> Amount {
        self.value
    }

    /// Returns the number of decimal places rendered (`P1 §7.1`: 2 to 8 for
    /// amounts).
    #[must_use]
    pub const fn decimals(self) -> u8 {
        self.decimals
    }
}

impl fmt::Display for AmountDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_scaled(f, self.value.0, Some(self.decimals))
    }
}

/// Writes a scaled amount, either canonically (`decimals == None`) or with an
/// exact number of fractional digits.
// The divisors are validated non-zero powers of ten; the integer part is
// bounded by the scaled magnitude. None of these divisions can trap.
#[allow(clippy::arithmetic_side_effects)]
fn write_scaled(f: &mut fmt::Formatter<'_>, scaled: i128, decimals: Option<u8>) -> fmt::Result {
    let magnitude = scaled.unsigned_abs();
    let integer = magnitude / SCALE_FACTOR_U128;
    let fraction = magnitude % SCALE_FACTOR_U128;
    let sign = if scaled < 0 { "-" } else { "" };
    match decimals {
        Some(0) => write!(f, "{sign}{integer}"),
        Some(digits) => {
            let divisor = 10u128.pow(u32::from(SCALE.saturating_sub(digits)));
            let fraction = fraction / divisor;
            write!(
                f,
                "{sign}{integer}.{fraction:0width$}",
                width = usize::from(digits)
            )
        }
        None => {
            let mut fraction = fraction;
            let mut digits = SCALE;
            loop {
                if digits <= 2 {
                    break;
                }
                if fraction.checked_rem(10) != Some(0) {
                    break;
                }
                fraction /= 10;
                digits = digits.saturating_sub(1);
            }
            write!(
                f,
                "{sign}{integer}.{fraction:0width$}",
                width = usize::from(digits)
            )
        }
    }
}
