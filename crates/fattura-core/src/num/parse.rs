//! Byte-level lexical parser shared by [`Amount`](super::Amount) and
//! [`Rate`](super::Rate).
//!
//! The matcher is hand-written and allocation-free: it implements the XSD
//! patterns directly instead of compiling a regular expression at runtime
//! (`PLAN §2.4`, `P1 §7.1`). It never indexes past the input and never panics.

use super::error::ParseError;

/// Validated parts of a decimal lexical form.
pub(super) struct DecimalParts<'a> {
    /// Whether the form carried a leading `-`.
    pub(super) negative: bool,
    /// ASCII digits before the point.
    pub(super) integer: &'a [u8],
    /// ASCII digits after the point.
    pub(super) fraction: &'a [u8],
}

/// Parses the lexical shape shared by the XSD decimal types.
///
/// * `allow_negative` — whether the pattern starts with `[\-]?`.
/// * `max_integer_digits` — width of `[0-9]{1,n}` before the point.
/// * `min_fraction` / `max_fraction` — bounds of the post-point digit run.
///
/// Digits, the leading sign and the single point are validated; the caller
/// remains responsible for numeric range checks (`RateType` ≤ `100.00`).
pub(super) fn parse_decimal(
    input: &str,
    allow_negative: bool,
    max_integer_digits: usize,
    min_fraction: usize,
    max_fraction: usize,
) -> Result<DecimalParts<'_>, ParseError> {
    let bytes = input.as_bytes();
    if bytes.is_empty() {
        return Err(ParseError::Empty);
    }

    let (negative, offset, body) = match bytes.first() {
        Some(b'-') => {
            if !allow_negative {
                return Err(ParseError::NegativeNotAllowed);
            }
            (true, 1usize, bytes.get(1..).unwrap_or_default())
        }
        Some(b'+') => return Err(ParseError::PlusSignNotAllowed),
        _ => (false, 0usize, bytes),
    };

    if body.is_empty() || body.first() == Some(&b'.') {
        return Err(ParseError::MissingIntegerPart);
    }

    let Some(point) = body.iter().position(|byte| *byte == b'.') else {
        return Err(ParseError::MissingDecimalPoint);
    };
    let (integer, tail) = body.split_at(point);
    // `tail` starts with the point; `get(1..)` can only fail if the input is
    // malformed in a way `split_at` already excluded.
    let fraction = tail.get(1..).unwrap_or_default();
    if fraction.contains(&b'.') {
        return Err(ParseError::MultipleDecimalPoints);
    }

    if integer.len() > max_integer_digits {
        return Err(ParseError::IntegerPartTooLong {
            digits: integer.len(),
            maximum: max_integer_digits,
        });
    }
    for (position, byte) in integer.iter().enumerate() {
        if !byte.is_ascii_digit() {
            return Err(ParseError::InvalidCharacter {
                index: offset.saturating_add(position),
                byte: *byte,
            });
        }
    }

    for (position, byte) in fraction.iter().enumerate() {
        if !byte.is_ascii_digit() {
            return Err(ParseError::InvalidCharacter {
                index: offset
                    .saturating_add(integer.len())
                    .saturating_add(1)
                    .saturating_add(position),
                byte: *byte,
            });
        }
    }
    if fraction.len() < min_fraction {
        return Err(ParseError::FractionTooFew {
            digits: fraction.len(),
            minimum: min_fraction,
        });
    }
    if fraction.len() > max_fraction {
        return Err(ParseError::FractionTooMany {
            digits: fraction.len(),
            maximum: max_fraction,
        });
    }

    Ok(DecimalParts {
        negative,
        integer,
        fraction,
    })
}

/// Converts a run of previously validated ASCII digits to an integer.
///
/// The caller guarantees at most 19 digits (11 integer + 8 fractional), which
/// fits `i128` with ample headroom. The unchecked operators are therefore safe
/// by construction; there is no overflow branch to leave untested.
#[allow(clippy::arithmetic_side_effects)]
pub(super) fn accumulate_digits(digits: &[u8]) -> i128 {
    let mut value = 0i128;
    for byte in digits {
        // `byte` is an ASCII digit; the low nibble is its numeric value.
        value = value * 10 + i128::from(*byte & 0x0f);
    }
    value
}
