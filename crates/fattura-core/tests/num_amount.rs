//! Lexical, range and arithmetic tests for [`Amount`].

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]

use core::str::FromStr;

use fattura_core::num::{Amount, ArithmeticError, ParseError, SCALE, SCALE_FACTOR};

fn amount(input: &str) -> Amount {
    Amount::parse_amount8(input).unwrap()
}

#[test]
fn num_parses_zero_and_negative_zero() {
    for input in ["0.00", "-0.00", "00.00", "-00.00000000"] {
        let parsed = Amount::parse_amount8(input).unwrap();
        assert_eq!(parsed, Amount::ZERO, "{input}");
        assert_eq!(parsed.to_string(), "0.00", "{input}");
    }
    assert_eq!(Amount::ZERO.scaled(), 0);
    assert_eq!(Amount::ZERO, Amount::from_scaled(0));
    assert_eq!(Amount::from_str("0.00").unwrap(), Amount::ZERO);
    assert_eq!(Amount::from_str("12.34").unwrap(), amount("12.34"));
    assert!("not-an-amount".parse::<Amount>().is_err());
    assert!("1,00".parse::<Amount>().is_err());
    assert_eq!(SCALE, 8);
    assert_eq!(SCALE_FACTOR, 100_000_000);
}

#[test]
fn num_parses_both_official_amount_types() {
    assert_eq!(amount("12.34").scaled(), 1_234_000_000);
    assert_eq!(amount("12.34"), Amount::parse_amount2("12.34").unwrap());
    assert_eq!(
        Amount::parse_amount8("12.3400").unwrap().scaled(),
        1_234_000_000
    );
    assert_eq!(
        Amount::parse_amount2("12.34").unwrap().scaled(),
        1_234_000_000
    );
    assert_eq!(
        Amount::parse_amount8("1.12345678").unwrap().scaled(),
        112_345_678
    );

    assert!(matches!(
        Amount::parse_amount2("1.000"),
        Err(ParseError::FractionTooMany {
            digits: 3,
            maximum: 2
        })
    ));
    assert!(matches!(
        Amount::parse_amount8("1.000000001"),
        Err(ParseError::FractionTooMany {
            digits: 9,
            maximum: 8
        })
    ));
    assert!(matches!(
        Amount::parse_amount8("1.0"),
        Err(ParseError::FractionTooFew {
            digits: 1,
            minimum: 2
        })
    ));
    assert!(matches!(
        Amount::parse_amount2("1.0"),
        Err(ParseError::FractionTooFew {
            digits: 1,
            minimum: 2
        })
    ));
}

#[test]
fn num_accepts_leading_zeros_within_the_width_limit() {
    assert_eq!(amount("01.00"), amount("1.00"));
    assert_eq!(amount("00000000001.00"), amount("1.00"));
    assert_eq!(amount("00000000000.00"), Amount::ZERO);
    assert!(matches!(
        amount_result("000000000001.00"),
        Err(ParseError::IntegerPartTooLong {
            digits: 12,
            maximum: 11
        })
    ));
}

fn amount_result(input: &str) -> Result<Amount, ParseError> {
    Amount::parse_amount8(input)
}

#[test]
fn num_accepts_exact_lexical_boundaries() {
    let maximum = amount("99999999999.99999999");
    assert_eq!(maximum.scaled(), 9_999_999_999_999_999_999);
    assert_eq!(maximum.to_string(), "99999999999.99999999");

    let minimum = amount("-99999999999.99999999");
    assert_eq!(minimum.scaled(), -9_999_999_999_999_999_999);
    assert_eq!(minimum.to_string(), "-99999999999.99999999");

    assert!(matches!(
        amount_result("100000000000.00"),
        Err(ParseError::IntegerPartTooLong {
            digits: 12,
            maximum: 11
        })
    ));
    assert!(matches!(
        amount_result("-100000000000.00"),
        Err(ParseError::IntegerPartTooLong {
            digits: 12,
            maximum: 11
        })
    ));
}

#[test]
fn num_rejects_non_conforming_separators_exponents_and_whitespace() {
    assert!(matches!(
        amount_result("1,00"),
        Err(ParseError::MissingDecimalPoint)
    ));
    assert!(matches!(
        amount_result("1.00,00"),
        Err(ParseError::InvalidCharacter { index: 4, .. })
    ));
    assert!(matches!(
        amount_result("1e2"),
        Err(ParseError::MissingDecimalPoint)
    ));
    assert!(matches!(
        amount_result("1.00e2"),
        Err(ParseError::InvalidCharacter { index: 4, .. })
    ));
    assert!(matches!(
        amount_result(" 1.00"),
        Err(ParseError::InvalidCharacter { index: 0, .. })
    ));
    assert!(matches!(
        amount_result("1.00 "),
        Err(ParseError::InvalidCharacter { index: 4, .. })
    ));
    assert!(matches!(amount_result(""), Err(ParseError::Empty)));
}

#[test]
fn num_rejects_signs_missing_parts_and_extra_points() {
    assert!(matches!(
        amount_result("+1.00"),
        Err(ParseError::PlusSignNotAllowed)
    ));
    assert!(matches!(
        amount_result("-"),
        Err(ParseError::MissingIntegerPart)
    ));
    assert!(matches!(
        amount_result(".50"),
        Err(ParseError::MissingIntegerPart)
    ));
    assert!(matches!(
        amount_result("1"),
        Err(ParseError::MissingDecimalPoint)
    ));
    assert!(matches!(
        amount_result("1.2.3"),
        Err(ParseError::MultipleDecimalPoints)
    ));
    assert!(matches!(
        amount_result("a1.00"),
        Err(ParseError::InvalidCharacter { index: 0, .. })
    ));
    assert!(matches!(
        amount_result("1a.00"),
        Err(ParseError::InvalidCharacter { index: 1, .. })
    ));
}

#[test]
fn num_every_parse_error_has_a_message() {
    let errors = [
        ParseError::Empty,
        ParseError::PlusSignNotAllowed,
        ParseError::NegativeNotAllowed,
        ParseError::MissingIntegerPart,
        ParseError::IntegerPartTooLong {
            digits: 12,
            maximum: 11,
        },
        ParseError::MissingDecimalPoint,
        ParseError::MultipleDecimalPoints,
        ParseError::InvalidCharacter {
            index: 3,
            byte: b'x',
        },
        ParseError::FractionTooFew {
            digits: 1,
            minimum: 2,
        },
        ParseError::FractionTooMany {
            digits: 9,
            maximum: 8,
        },
        ParseError::RateOutOfRange,
    ];
    for error in errors {
        let message = error.to_string();
        assert!(!message.is_empty(), "empty message for {error:?}");
        assert!(core::error::Error::source(&error).is_none());
    }
}

#[test]
fn num_canonical_format_round_trips() {
    for input in [
        "0.00",
        "0.10",
        "1.00",
        "12.34",
        "12.3400",
        "0.00000001",
        "99999999999.99999999",
        "-99999999999.99999999",
        "-0.005",
        "500.00",
        "42092.05875",
    ] {
        let parsed = amount(input);
        let formatted = parsed.to_string();
        assert_eq!(amount(&formatted), parsed, "{input} -> {formatted}");
    }
    assert_eq!(amount("12.3400").to_string(), "12.34");
    assert_eq!(amount("0.00000001").to_string(), "0.00000001");
}

#[test]
fn num_formats_with_exactly_the_requested_decimals() {
    let value = amount("1.235");
    let two = value.display_with(2).unwrap();
    assert_eq!(two.to_string(), "1.24");
    assert_eq!(two.decimals(), 2);
    assert_eq!(two.amount(), amount("1.24"));

    assert_eq!(value.display_with(8).unwrap().to_string(), "1.23500000");
    assert_eq!(value.display_with(0).unwrap().to_string(), "1");
    assert_eq!(
        amount("500.00").display_with(2).unwrap().to_string(),
        "500.00"
    );
    assert_eq!(
        amount("-1.235").display_with(2).unwrap().to_string(),
        "-1.24"
    );
    assert!(matches!(
        value.display_with(9),
        Err(ArithmeticError::InvalidScale { decimals: 9 })
    ));
}

#[test]
fn num_checked_add_sub_neg_and_mul_are_exact() {
    let a = amount("12.34");
    let b = amount("0.66");
    assert_eq!(a.checked_add(b).unwrap(), amount("13.00"));
    assert_eq!(a.checked_sub(b).unwrap(), amount("11.68"));
    assert_eq!(a.checked_neg().unwrap(), amount("-12.34"));
    assert_eq!(a.checked_mul_i128(100).unwrap(), amount("1234.00"));
    assert_eq!(a.checked_abs().unwrap(), a);
    assert_eq!(amount("-12.34").checked_abs().unwrap(), a);
    assert_eq!(a.abs_diff(b).unwrap(), amount("11.68"));
    assert_eq!(
        amount("1.00").checked_add(amount("-1.00")).unwrap(),
        Amount::ZERO
    );
}

#[test]
fn num_every_arithmetic_overflow_is_reported() {
    let maximum = Amount::from_scaled(i128::MAX);
    let minimum = Amount::from_scaled(i128::MIN);

    assert_eq!(
        maximum.checked_add(Amount::from_scaled(1)),
        Err(ArithmeticError::Overflow)
    );
    assert_eq!(
        minimum.checked_sub(Amount::from_scaled(1)),
        Err(ArithmeticError::Overflow)
    );
    assert_eq!(minimum.checked_neg(), Err(ArithmeticError::Overflow));
    assert_eq!(minimum.checked_abs(), Err(ArithmeticError::Overflow));
    assert_eq!(maximum.checked_mul_i128(2), Err(ArithmeticError::Overflow));
    assert_eq!(maximum.abs_diff(minimum), Err(ArithmeticError::Overflow));
    assert_eq!(minimum.as_rational(), Err(ArithmeticError::Overflow));
    assert_eq!(
        maximum.within_tolerance(minimum, Amount::ZERO),
        Err(ArithmeticError::Overflow)
    );
    assert_eq!(
        maximum.exceeds_tolerance(minimum, Amount::ZERO),
        Err(ArithmeticError::Overflow)
    );
    assert!(maximum.checked_add(Amount::ZERO).is_ok());
    assert!(minimum.checked_sub(Amount::ZERO).is_ok());

    // Rounding i128::MAX up at seven decimals overflows the scale-back.
    assert_eq!(maximum.round_to(7), Err(ArithmeticError::Overflow));
}

#[test]
fn num_percentage_helpers_report_rational_overflow() {
    let minimum = Amount::from_scaled(i128::MIN);
    assert_eq!(
        minimum.apply_percentage(fattura_core::num::Rate::ZERO),
        Err(ArithmeticError::Overflow)
    );
    assert_eq!(minimum.apply_cascade(&[]), Err(ArithmeticError::Overflow));
}

#[test]
fn num_every_arithmetic_error_has_a_message() {
    for error in [
        ArithmeticError::Overflow,
        ArithmeticError::DivisionByZero,
        ArithmeticError::InvalidScale { decimals: 9 },
        ArithmeticError::OutOfRange,
    ] {
        let message = error.to_string();
        assert!(!message.is_empty(), "empty message for {error:?}");
        assert!(core::error::Error::source(&error).is_none());
    }
}

#[test]
fn num_tolerance_edges_are_per_call_site() {
    // 00421 / 00423: |difference| >= 0.01 is an error.
    let cent = amount("0.01");
    let below = amount("0.009");
    let at = amount("0.010");
    let above = amount("0.011");

    assert!(Amount::ZERO.within_tolerance(below, cent).unwrap());
    assert!(!Amount::ZERO.within_tolerance(at, cent).unwrap());
    assert!(!Amount::ZERO.within_tolerance(above, cent).unwrap());

    assert!(!Amount::ZERO.exceeds_tolerance(below, cent).unwrap());
    assert!(Amount::ZERO.exceeds_tolerance(at, cent).unwrap());
    assert!(Amount::ZERO.exceeds_tolerance(above, cent).unwrap());

    // 00422: |difference| >= 1.00 is an error.
    let euro = amount("1.00");
    assert!(Amount::ZERO.within_tolerance(amount("0.99"), euro).unwrap());
    assert!(!Amount::ZERO.within_tolerance(euro, euro).unwrap());
    assert!(Amount::ZERO.exceeds_tolerance(euro, euro).unwrap());

    // A negative tolerance is never satisfied.
    assert!(
        !Amount::ZERO
            .within_tolerance(amount("0.001"), amount("-0.01"))
            .unwrap()
    );
}
