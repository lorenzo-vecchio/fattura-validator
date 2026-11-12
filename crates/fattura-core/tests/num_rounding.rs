//! Rounding, percentage and cascading-discount tests using the official
//! examples from `P3 §3.3` and `P3 §3.5`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]

use fattura_core::num::{Amount, ArithmeticError, Rate, Rational};

fn amount(input: &str) -> Amount {
    Amount::parse_amount8(input).unwrap()
}

fn rate(input: &str) -> Rate {
    Rate::parse(input).unwrap()
}

#[test]
fn num_official_00421_rounding_examples() {
    assert_eq!(amount("1500.2448").round_to(2).unwrap(), amount("1500.24"));
    assert_eq!(amount("1500.2458").round_to(2).unwrap(), amount("1500.25"));
    assert_eq!(
        Rational::new(1_500_244_800, 1_000_000)
            .unwrap()
            .round_to(2)
            .unwrap(),
        amount("1500.24")
    );
    assert_eq!(
        Rational::new(1_500_245_800, 1_000_000)
            .unwrap()
            .round_to(2)
            .unwrap(),
        amount("1500.25")
    );
}

#[test]
fn num_half_up_moves_away_from_zero_on_the_magnitude() {
    assert_eq!(amount("0.005").round_to(2).unwrap(), amount("0.01"));
    assert_eq!(amount("-0.005").round_to(2).unwrap(), amount("-0.01"));
    assert_eq!(amount("0.0049").round_to(2).unwrap(), amount("0.00"));
    assert_eq!(amount("-0.0049").round_to(2).unwrap(), amount("0.00"));
    assert_eq!(
        Rational::new(-5, 1_000).unwrap().round_to(2).unwrap(),
        amount("-0.01")
    );
    assert_eq!(
        Rational::new(5, 1_000).unwrap().round_to(2).unwrap(),
        amount("0.01")
    );
    assert_eq!(
        Rational::new(-5, 1_000)
            .unwrap()
            .round_to(2)
            .unwrap()
            .to_string(),
        "-0.01"
    );
    assert!(matches!(
        Rational::from_integer(1).round_to(9),
        Err(ArithmeticError::InvalidScale { decimals: 9 })
    ));
}

#[test]
fn num_official_00423_cascading_percentage_example() {
    let unit_price = amount("500.00");
    let quantity = amount("100.00");
    let rates = [rate("8.50"), rate("5.15"), rate("3.00")];

    let discounted = unit_price.apply_cascade(&rates).unwrap();
    let total = discounted
        .checked_mul(quantity.as_rational().unwrap())
        .unwrap();

    // Exact value mandated by the official example: 42092.05875.
    assert_eq!(discounted.round_to(8).unwrap(), amount("420.9205875"));
    assert_eq!(total, Rational::new(4_209_205_875, 100_000).unwrap());
    assert_eq!(total.round_to(8).unwrap(), amount("42092.05875"));
    assert_eq!(total.round_to(8).unwrap().to_string(), "42092.05875");
    assert_eq!(total.round_to(2).unwrap().to_string(), "42092.06");

    // The same cascade expressed through the exact fraction.
    let reference = Rational::new(500, 1)
        .unwrap()
        .checked_mul(Rational::new(915, 1_000).unwrap())
        .unwrap()
        .checked_mul(Rational::new(9_485, 10_000).unwrap())
        .unwrap()
        .checked_mul(Rational::new(97, 100).unwrap())
        .unwrap()
        .checked_mul(Rational::new(100, 1).unwrap())
        .unwrap();
    assert_eq!(total, reference);
}

#[test]
fn num_official_00423_amount_discount_example() {
    let unit_price = amount("500.00");
    let quantity = amount("100.00");
    let mut unit = unit_price;
    for discount in ["10.05", "5.25", "4.70"] {
        unit = unit.checked_sub(amount(discount)).unwrap();
    }
    assert_eq!(unit, amount("480.00"));
    let total = unit
        .as_rational()
        .unwrap()
        .checked_mul(quantity.as_rational().unwrap())
        .unwrap()
        .round_to(8)
        .unwrap();
    assert_eq!(total, amount("48000.00"));
    assert_eq!(total.to_string(), "48000.00");
}

#[test]
fn num_percentage_application_is_exact_before_rounding() {
    let applied = amount("500.00").apply_percentage(rate("8.50")).unwrap();
    assert_eq!(applied, Rational::new(85, 2).unwrap());
    assert_eq!(applied.round_to(2).unwrap(), amount("42.50"));

    // A rate with three decimals is not a RateType value at all.
    assert!(Rate::parse("42.505").is_err());
}

#[test]
fn num_every_rational_overflow_branch_is_reported() {
    let maximum = Rational::from_integer(i128::MAX);
    let minimum = Rational::from_integer(i128::MIN);
    let half = Rational::new(1, 2).unwrap();
    let large_denominator = Rational::new(1, i128::MAX).unwrap();
    let negative_large_denominator = Rational::new(-1, i128::MAX).unwrap();

    // checked_add: left product, right product, numerator, denominator.
    assert_eq!(maximum.checked_add(half), Err(ArithmeticError::Overflow));
    assert_eq!(half.checked_add(maximum), Err(ArithmeticError::Overflow));
    assert_eq!(maximum.checked_add(maximum), Err(ArithmeticError::Overflow));
    assert_eq!(
        large_denominator.checked_add(negative_large_denominator),
        Err(ArithmeticError::Overflow)
    );

    // checked_sub: left product, right product, numerator, denominator.
    assert_eq!(maximum.checked_sub(half), Err(ArithmeticError::Overflow));
    assert_eq!(half.checked_sub(maximum), Err(ArithmeticError::Overflow));
    assert_eq!(maximum.checked_sub(minimum), Err(ArithmeticError::Overflow));
    assert_eq!(
        large_denominator.checked_sub(large_denominator),
        Err(ArithmeticError::Overflow)
    );

    // checked_mul: numerator and denominator.
    assert_eq!(
        maximum.checked_mul(Rational::from_integer(2)),
        Err(ArithmeticError::Overflow)
    );
    assert_eq!(
        large_denominator.checked_mul(large_denominator),
        Err(ArithmeticError::Overflow)
    );

    // checked_div: numerator and denominator.
    assert_eq!(maximum.checked_div(half), Err(ArithmeticError::Overflow));
    assert_eq!(
        large_denominator.checked_div(maximum),
        Err(ArithmeticError::Overflow)
    );

    // round_to and the cascade multiply.
    assert_eq!(maximum.round_to(2), Err(ArithmeticError::Overflow));
    assert_eq!(
        maximum.apply_cascade(&[rate("0.01")]),
        Err(ArithmeticError::Overflow)
    );

    // Normalisation rejects an unrepresentable denominator magnitude.
    assert_eq!(Rational::new(1, i128::MIN), Err(ArithmeticError::Overflow));
}

#[test]
fn num_rational_operations_reduce_and_stay_exact() {
    let third = Rational::new(1, 3).unwrap();
    let sixth = Rational::new(1, 6).unwrap();
    let half = third.checked_add(sixth).unwrap();
    assert_eq!(half, Rational::new(1, 2).unwrap());
    assert_eq!(half.numerator(), 1);
    assert_eq!(half.denominator(), 2);
    assert_eq!(
        third.checked_sub(sixth).unwrap(),
        Rational::new(1, 6).unwrap()
    );
    assert_eq!(
        Rational::new(2, 3)
            .unwrap()
            .checked_mul(Rational::new(3, 4).unwrap())
            .unwrap(),
        Rational::new(1, 2).unwrap()
    );
    assert_eq!(
        Rational::new(1, 2)
            .unwrap()
            .checked_div(Rational::new(1, 4).unwrap())
            .unwrap(),
        Rational::from_integer(2)
    );
    assert_eq!(
        Rational::new(-1, 2).unwrap().checked_neg().unwrap(),
        Rational::new(1, 2).unwrap()
    );
    assert_eq!(
        Rational::new(-1, 2).unwrap().checked_abs().unwrap(),
        Rational::new(1, 2).unwrap()
    );
    assert_eq!(Rational::new(6, 3).unwrap(), Rational::from_integer(2));
    assert!(Rational::new(1, -2).unwrap().is_negative());
    assert!(!Rational::new(1, -2).unwrap().is_zero());
    assert!(!Rational::from_integer(0).is_negative());
    assert!(!Rational::new(1, 2).unwrap().is_negative());
    assert_eq!(Rational::new(1, -2).unwrap().denominator(), 2);

    assert_eq!(Rational::new(1, 0), Err(ArithmeticError::DivisionByZero));
    assert_eq!(
        Rational::from_integer(1).checked_div(Rational::from_integer(0)),
        Err(ArithmeticError::DivisionByZero)
    );
    assert_eq!(Rational::new(i128::MIN, 1), Err(ArithmeticError::Overflow));
    assert_eq!(
        Rational::from_integer(i128::MIN).checked_neg(),
        Err(ArithmeticError::Overflow)
    );
    assert_eq!(
        Rational::from_integer(i128::MIN).checked_abs(),
        Err(ArithmeticError::Overflow)
    );
    assert_eq!(
        Rational::from_integer(i128::MAX).checked_add(Rational::from_integer(1)),
        Err(ArithmeticError::Overflow)
    );
    assert_eq!(
        Rational::from_integer(i128::MIN).checked_sub(Rational::from_integer(1)),
        Err(ArithmeticError::Overflow)
    );
    assert_eq!(
        Rational::from_integer(i128::MAX).checked_mul(Rational::from_integer(2)),
        Err(ArithmeticError::Overflow)
    );
    assert_eq!(
        Rational::from_integer(i128::MAX)
            .checked_sub(Rational::from_integer(i128::MAX))
            .unwrap(),
        Rational::from_integer(0)
    );
}
