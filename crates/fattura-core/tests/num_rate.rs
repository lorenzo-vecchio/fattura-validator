//! Lexical and range tests for [`Rate`].

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]

use core::str::FromStr;

use fattura_core::num::{ArithmeticError, ParseError, Rate};

fn rate(input: &str) -> Rate {
    Rate::parse(input).unwrap()
}

#[test]
fn num_accepts_the_official_rate_range() {
    assert_eq!(rate("0.00"), Rate::ZERO);
    assert_eq!(rate("0.00").raw(), 0);
    assert_eq!(rate("1.00").raw(), 100);
    assert_eq!(rate("8.50").raw(), 850);
    assert_eq!(rate("100.00"), Rate::MAX);
    assert_eq!(rate("100.00").raw(), 10_000);
    assert_eq!(rate("001.00"), rate("1.00"));
    assert_eq!(rate("99.99").raw(), 9_999);
    assert_eq!(Rate::from_str("8.50").unwrap(), rate("8.50"));
}

#[test]
fn num_rejects_rates_above_one_hundred() {
    assert_eq!(Rate::parse("100.01"), Err(ParseError::RateOutOfRange));
    assert_eq!(Rate::parse("999.99"), Err(ParseError::RateOutOfRange));
}

#[test]
fn num_rejects_non_conforming_lexical_forms() {
    assert!(matches!(
        Rate::parse("100.0"),
        Err(ParseError::FractionTooFew {
            digits: 1,
            minimum: 2
        })
    ));
    assert!(matches!(
        Rate::parse("1.000"),
        Err(ParseError::FractionTooMany {
            digits: 3,
            maximum: 2
        })
    ));
    assert!(matches!(
        Rate::parse("1000.00"),
        Err(ParseError::IntegerPartTooLong {
            digits: 4,
            maximum: 3
        })
    ));
    assert_eq!(Rate::parse("-1.00"), Err(ParseError::NegativeNotAllowed));
    assert_eq!(Rate::parse("-0.00"), Err(ParseError::NegativeNotAllowed));
    assert_eq!(Rate::parse("+1.00"), Err(ParseError::PlusSignNotAllowed));
    assert_eq!(Rate::parse("1,00"), Err(ParseError::MissingDecimalPoint));
    assert_eq!(Rate::parse("1e2"), Err(ParseError::MissingDecimalPoint));
    assert!(matches!(
        Rate::parse(" 1.00"),
        Err(ParseError::InvalidCharacter { index: 0, .. })
    ));
    assert_eq!(Rate::parse(""), Err(ParseError::Empty));
}

#[test]
fn num_formats_with_exactly_two_decimals() {
    assert_eq!(Rate::ZERO.to_string(), "0.00");
    assert_eq!(rate("8.50").to_string(), "8.50");
    assert_eq!(rate("100.00").to_string(), "100.00");
    assert_eq!(rate("8.50").to_string(), "8.50");
    for input in ["0.00", "0.01", "1.00", "8.50", "100.00", "099.99"] {
        let parsed = rate(input);
        assert_eq!(rate(&parsed.to_string()), parsed, "{input}");
    }
}

#[test]
fn num_raw_construction_enforces_the_ceiling() {
    assert_eq!(Rate::from_raw(0).unwrap(), Rate::ZERO);
    assert_eq!(Rate::from_raw(10_000).unwrap(), Rate::MAX);
    assert_eq!(Rate::from_raw(10_001), Err(ArithmeticError::OutOfRange));
    assert_eq!(Rate::from_raw(u32::MAX), Err(ArithmeticError::OutOfRange));
}

#[test]
fn num_converts_to_an_exact_rational_fraction() {
    let half = rate("8.50").as_rational();
    assert_eq!(half.numerator(), 17);
    assert_eq!(half.denominator(), 200);

    let zero = Rate::ZERO.as_rational();
    assert!(zero.is_zero());
    assert_eq!(zero.numerator(), 0);
    assert_eq!(zero.denominator(), 1);
}
