//! Property tests against a `BigRational` reference model (`PLAN §10` R8).
//!
//! The proptest RNG is seeded explicitly so runs are reproducible; no other
//! randomness is used anywhere in the numeric core.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use fattura_core::num::{Amount, ArithmeticError, Rate, Rational, SCALE_FACTOR};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, ToPrimitive, Zero};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;

fn amount_value(amount: Amount) -> BigRational {
    BigRational::new(BigInt::from(amount.scaled()), BigInt::from(SCALE_FACTOR))
}

fn rational_value(rational: Rational) -> BigRational {
    BigRational::new(
        BigInt::from(rational.numerator()),
        BigInt::from(rational.denominator()),
    )
}

/// Half away from zero, expressed with exact big integers.
fn reference_round(value: &BigRational, decimals: u32) -> BigInt {
    let factor = BigInt::from(10u8).pow(decimals);
    let scaled = value * BigRational::from_integer(factor);
    let numerator = scaled.numer();
    let denominator = scaled.denom();
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    if remainder.abs() * BigInt::from(2u8) >= *denominator {
        if remainder.is_negative() {
            quotient - 1
        } else {
            quotient + 1
        }
    } else {
        quotient
    }
}

/// A canonical `Amount8DecimalType` string plus its exact value.
fn amount_strategy() -> impl Strategy<Value = (String, BigRational)> {
    (
        any::<bool>(),
        prop::collection::vec(0u8..=9, 1..=11),
        prop::collection::vec(0u8..=9, 2..=8),
    )
        .prop_map(|(negative, integer_digits, fraction_digits)| {
            let integer: String = integer_digits
                .iter()
                .map(|digit| char::from(b'0' + digit))
                .collect();
            let fraction: String = fraction_digits
                .iter()
                .map(|digit| char::from(b'0' + digit))
                .collect();
            let mut text = String::new();
            if negative {
                text.push('-');
            }
            text.push_str(&integer);
            text.push('.');
            text.push_str(&fraction);

            let scale = BigInt::from(10u8).pow(fraction_digits.len() as u32);
            let integer_value: BigInt = integer.parse().unwrap();
            let fraction_value: BigInt = fraction.parse().unwrap();
            let value = BigRational::new(integer_value * &scale + fraction_value, scale);
            let value = if negative { -value } else { value };
            (text, value)
        })
}

fn arbitrary_amount() -> impl Strategy<Value = (Amount, BigRational)> {
    any::<i128>().prop_map(|raw| {
        (
            Amount::from_scaled(raw),
            BigRational::new(BigInt::from(raw), BigInt::from(SCALE_FACTOR)),
        )
    })
}

fn bounded_amount() -> impl Strategy<Value = (Amount, BigRational)> {
    (-9_999_999_999_999_999_999i128..=9_999_999_999_999_999_999i128).prop_map(|raw| {
        (
            Amount::from_scaled(raw),
            BigRational::new(BigInt::from(raw), BigInt::from(SCALE_FACTOR)),
        )
    })
}

fn rate_strategy() -> impl Strategy<Value = (Rate, BigRational)> {
    (0u32..=10_000).prop_map(|raw| {
        (
            Rate::from_raw(raw).unwrap(),
            BigRational::new(BigInt::from(raw), BigInt::from(10_000u32)),
        )
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 128,
        rng_seed: RngSeed::Fixed(13),
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    #[test]
    fn num_parse_and_canonical_format_round_trip((text, expected) in amount_strategy()) {
        let parsed = Amount::parse_amount8(&text).unwrap();
        prop_assert_eq!(amount_value(parsed), expected.clone());

        let formatted = parsed.to_string();
        let reparsed = Amount::parse_amount8(&formatted).unwrap();
        prop_assert_eq!(reparsed, parsed);
        prop_assert_eq!(amount_value(reparsed), expected);
    }

    #[test]
    fn num_checked_add_matches_reference(
        (left, left_value) in arbitrary_amount(),
        (right, right_value) in arbitrary_amount(),
    ) {
        match left.checked_add(right) {
            Ok(sum) => prop_assert_eq!(amount_value(sum), left_value + right_value),
            Err(error) => {
                prop_assert_eq!(error, ArithmeticError::Overflow);
                let raw = BigInt::from(left.scaled()) + BigInt::from(right.scaled());
                prop_assert!(raw.to_i128().is_none());
            }
        }
    }

    #[test]
    fn num_checked_sub_matches_reference(
        (left, left_value) in arbitrary_amount(),
        (right, right_value) in arbitrary_amount(),
    ) {
        match left.checked_sub(right) {
            Ok(difference) => prop_assert_eq!(amount_value(difference), left_value - right_value),
            Err(error) => {
                prop_assert_eq!(error, ArithmeticError::Overflow);
                let raw = BigInt::from(left.scaled()) - BigInt::from(right.scaled());
                prop_assert!(raw.to_i128().is_none());
            }
        }
    }

    #[test]
    fn num_percentage_application_matches_reference(
        (amount, amount_exact) in bounded_amount(),
        (rate, rate_exact) in rate_strategy(),
    ) {
        let applied = amount.apply_percentage(rate).unwrap();
        let expected = amount_exact * rate_exact;
        prop_assert_eq!(rational_value(applied), expected.clone());

        for decimals in [2u8, 8] {
            let rounded = applied.round_to(decimals).unwrap();
            let scale_back = BigInt::from(10u8).pow(8 - u32::from(decimals));
            let expected_scaled = reference_round(&expected, u32::from(decimals)) * scale_back;
            prop_assert_eq!(BigInt::from(rounded.scaled()), expected_scaled);
        }
    }

    #[test]
    fn num_cascading_discounts_match_reference(
        (amount, amount_exact) in bounded_amount(),
        raw_rates in prop::collection::vec(0u32..=10_000, 0..=3),
    ) {
        let rates: Vec<Rate> = raw_rates
            .iter()
            .map(|raw| Rate::from_raw(*raw).unwrap())
            .collect();
        let applied = amount.apply_cascade(&rates).unwrap();

        let mut expected = amount_exact;
        for raw in raw_rates {
            let rate = BigRational::new(BigInt::from(raw), BigInt::from(10_000u32));
            expected *= BigRational::from_integer(BigInt::from(1u8)) - rate;
        }
        prop_assert_eq!(rational_value(applied), expected);
    }

    #[test]
    fn num_rational_rounding_matches_reference(
        numerator in -1_000_000_000i128..=1_000_000_000i128,
        denominator in 1i128..=1_000_000_000i128,
        decimals in 0u8..=8,
    ) {
        let value = Rational::new(numerator, denominator).unwrap();
        let rounded = value.round_to(decimals).unwrap();
        let expected = BigRational::new(BigInt::from(numerator), BigInt::from(denominator));
        let scale_back = BigInt::from(10u8).pow(8 - u32::from(decimals));
        let expected_scaled = reference_round(&expected, u32::from(decimals)) * &scale_back;
        prop_assert_eq!(BigInt::from(rounded.scaled()), expected_scaled.clone());
        prop_assert_eq!(BigInt::from(rounded.scaled()) % scale_back, BigInt::zero());
    }
}
