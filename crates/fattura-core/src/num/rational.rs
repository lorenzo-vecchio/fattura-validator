//! Exact rational arithmetic for percentages and cascading discounts.
//!
//! Percentages are kept as integer numerator/denominator pairs until the caller
//! chooses a rounding point (`P3 §3.5`, `PLAN §10` R8). This is the reference
//! model that the property tests compare against `BigRational`.
//!
//! # Rounding
//!
//! [`Rational::round_to`] rounds **half away from zero on the magnitude**:
//! a value exactly halfway between two representable amounts moves away from
//! zero, so `-0.005` rounded to two decimals is `-0.01`. All rounding is
//! explicit; no operation rounds implicitly.

use super::amount::{Amount, SCALE};
use super::error::ArithmeticError;
use super::power::pow10;
use super::rate::Rate;

/// One hundred percent expressed in hundredths of a percent.
const RATE_DENOMINATOR: i128 = 10_000;

/// An exact rational number with a positive denominator.
///
/// The value is always reduced to lowest terms and the denominator is always
/// positive. Both fields are `i128`; [`Rational::new`] reports
/// [`ArithmeticError::Overflow`] when either magnitude is `i128::MIN`, because
/// that magnitude has no positive `i128` representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational {
    num: i128,
    den: i128,
}

impl Rational {
    /// Builds `num / den`, reduced to lowest terms.
    ///
    /// Returns [`ArithmeticError::DivisionByZero`] for a zero denominator and
    /// [`ArithmeticError::Overflow`] when normalisation cannot be represented.
    // `divisor` is the positive gcd of two positive magnitudes, so these
    // divisions cannot trap.
    #[allow(clippy::arithmetic_side_effects)]
    pub fn new(num: i128, den: i128) -> Result<Self, ArithmeticError> {
        if den == 0 {
            return Err(ArithmeticError::DivisionByZero);
        }
        if num == 0 {
            return Ok(Self { num: 0, den: 1 });
        }
        let numerator = num.checked_abs().ok_or(ArithmeticError::Overflow)?;
        let denominator = den.checked_abs().ok_or(ArithmeticError::Overflow)?;
        let divisor = gcd(numerator, denominator);
        let numerator = numerator / divisor;
        let denominator = denominator / divisor;
        let negative = (num < 0) ^ (den < 0);
        // `numerator` is a positive magnitude no larger than `i128::MAX`, so
        // negating it cannot overflow.
        let numerator = if negative {
            numerator.saturating_neg()
        } else {
            numerator
        };
        Ok(Self {
            num: numerator,
            den: denominator,
        })
    }

    /// Builds the exact integer `value` (`P1 §7.1` integer quantities).
    #[must_use]
    pub const fn from_integer(value: i128) -> Self {
        Self { num: value, den: 1 }
    }

    /// Builds `raw / 10000` for `raw <= 10000`, reduced to lowest terms.
    ///
    /// This is the representation of a `RateType` percentage; the bound is
    /// enforced by [`Rate`], so no error path is needed.
    #[allow(clippy::arithmetic_side_effects)]
    pub(super) fn from_hundredths_of_percent(raw: u32) -> Self {
        if raw == 0 {
            return Self { num: 0, den: 1 };
        }
        let numerator = i128::from(raw);
        let divisor = gcd(numerator, RATE_DENOMINATOR);
        Self {
            num: numerator / divisor,
            den: RATE_DENOMINATOR / divisor,
        }
    }

    /// Returns the numerator. It carries the sign of the value (`P3 §3.5`).
    #[must_use]
    pub const fn numerator(self) -> i128 {
        self.num
    }

    /// Returns the denominator. It is always strictly positive (`P3 §3.5`).
    #[must_use]
    pub const fn denominator(self) -> i128 {
        self.den
    }

    /// Returns `true` for zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.num == 0
    }

    /// Returns `true` for negative values.
    #[must_use]
    pub const fn is_negative(self) -> bool {
        self.num < 0
    }

    /// Checked addition for exact percentage arithmetic (`P3 §3.5`).
    pub fn checked_add(self, rhs: Self) -> Result<Self, ArithmeticError> {
        let left = self
            .num
            .checked_mul(rhs.den)
            .ok_or(ArithmeticError::Overflow)?;
        let right = rhs
            .num
            .checked_mul(self.den)
            .ok_or(ArithmeticError::Overflow)?;
        let numerator = left.checked_add(right).ok_or(ArithmeticError::Overflow)?;
        let denominator = self
            .den
            .checked_mul(rhs.den)
            .ok_or(ArithmeticError::Overflow)?;
        Self::new(numerator, denominator)
    }

    /// Checked subtraction for exact percentage arithmetic (`P3 §3.5`).
    pub fn checked_sub(self, rhs: Self) -> Result<Self, ArithmeticError> {
        let left = self
            .num
            .checked_mul(rhs.den)
            .ok_or(ArithmeticError::Overflow)?;
        let right = rhs
            .num
            .checked_mul(self.den)
            .ok_or(ArithmeticError::Overflow)?;
        let numerator = left.checked_sub(right).ok_or(ArithmeticError::Overflow)?;
        let denominator = self
            .den
            .checked_mul(rhs.den)
            .ok_or(ArithmeticError::Overflow)?;
        Self::new(numerator, denominator)
    }

    /// Checked multiplication for exact percentage arithmetic (`P3 §3.5`).
    pub fn checked_mul(self, rhs: Self) -> Result<Self, ArithmeticError> {
        let numerator = self
            .num
            .checked_mul(rhs.num)
            .ok_or(ArithmeticError::Overflow)?;
        let denominator = self
            .den
            .checked_mul(rhs.den)
            .ok_or(ArithmeticError::Overflow)?;
        Self::new(numerator, denominator)
    }

    /// Checked division for exact percentage arithmetic (`P3 §3.5`).
    pub fn checked_div(self, rhs: Self) -> Result<Self, ArithmeticError> {
        if rhs.num == 0 {
            return Err(ArithmeticError::DivisionByZero);
        }
        let numerator = self
            .num
            .checked_mul(rhs.den)
            .ok_or(ArithmeticError::Overflow)?;
        let denominator = self
            .den
            .checked_mul(rhs.num)
            .ok_or(ArithmeticError::Overflow)?;
        Self::new(numerator, denominator)
    }

    /// Checked negation for signed discounts and adjustments (`P3 §3.5`).
    pub fn checked_neg(self) -> Result<Self, ArithmeticError> {
        self.num
            .checked_neg()
            .map(|num| Self { num, den: self.den })
            .ok_or(ArithmeticError::Overflow)
    }

    /// Checked absolute value (`P3 §3.3`–`§3.5`).
    pub fn checked_abs(self) -> Result<Self, ArithmeticError> {
        self.num
            .checked_abs()
            .map(|num| Self { num, den: self.den })
            .ok_or(ArithmeticError::Overflow)
    }

    /// Applies percentage discounts in cascade, exactly.
    ///
    /// For rates `r1, r2, …` the result is
    /// `value × (1 − r1/100) × (1 − r2/100) × …`, which is the `00423` rule
    /// for multiple percentage discounts (`P3 §3.5`, rule 2).
    pub fn apply_cascade(self, rates: &[Rate]) -> Result<Self, ArithmeticError> {
        let mut value = self;
        for rate in rates {
            let complement = 10_000u32.saturating_sub(rate.raw());
            let factor = Self::from_hundredths_of_percent(complement);
            value = value.checked_mul(factor)?;
        }
        Ok(value)
    }

    /// Rounds half-up to `decimals` decimal places and returns an [`Amount`].
    ///
    /// The result is stored at the amount scale of eight decimals. Half-up
    /// moves away from zero, so `-0.005` at two decimals is `-0.01`
    /// (`P3 §3.3`). `decimals` must be in `0..=8`.
    // The divisor is the rational's positive denominator, so these divisions
    // cannot trap.
    #[allow(clippy::arithmetic_side_effects)]
    pub fn round_to(self, decimals: u8) -> Result<Amount, ArithmeticError> {
        if decimals > SCALE {
            return Err(ArithmeticError::InvalidScale { decimals });
        }
        let factor = pow10(decimals);
        let scaled = self
            .num
            .checked_mul(factor)
            .ok_or(ArithmeticError::Overflow)?;
        let quotient = scaled / self.den;
        let remainder = scaled % self.den;
        let half = (self.den >> 1).saturating_add(self.den & 1);
        // `|quotient| <= |scaled|`, so the saturation is unreachable and only
        // avoids an arithmetic lint.
        let rounded = if remainder.unsigned_abs() >= half.unsigned_abs() {
            // The remainder is non-zero here, so `signum` is exactly ±1 and
            // moves the magnitude away from zero.
            quotient.saturating_add(remainder.signum())
        } else {
            quotient
        };
        let scale_back = pow10(SCALE.saturating_sub(decimals));
        rounded
            .checked_mul(scale_back)
            .map(Amount::from_scaled)
            .ok_or(ArithmeticError::Overflow)
    }
}

/// Greatest common divisor of two strictly positive integers.
// Both arguments are strictly positive on entry; the Euclidean remainder is
// bounded by the divisor, so the division cannot trap.
#[allow(clippy::arithmetic_side_effects)]
fn gcd(mut left: i128, mut right: i128) -> i128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}
