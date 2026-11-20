//! Calendar dates without time zones.
//!
//! Invoice fields typed `xs:date` (`Data`, `DataDDT`, `DataScadenzaPagamento`,
//! …) are represented by [`CivilDate`]. The parser is pure integer arithmetic:
//! no `chrono`, no floating point, no system clock. The lexical form follows
//! `xs:date` (`P1 §6`, `P1 §7.1`): `YYYY-MM-DD`, optionally followed by `Z` or a
//! numeric time-zone offset. The offset is validated and then discarded because
//! an invoice date is a civil date, not an instant.
//!
//! The supported year range is `0000..=9999`. Invoice dates are additionally
//! constrained by `DataFatturaType` (`minInclusive="1970-01-01"`); that lower
//! bound belongs to the schema layer (E3.3), not to the model.

use core::fmt;
use core::str::FromStr;

/// A calendar date with no time or time-zone component.
///
/// Field order is `year`, `month`, `day` so the derived ordering is
/// chronological. The value is only constructed through [`CivilDate::parse`] or
/// [`CivilDate::from_ymd`], both of which reject impossible calendar dates such
/// as `2023-02-29`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CivilDate {
    year: u16,
    month: u8,
    day: u8,
}

impl CivilDate {
    /// Builds a date from its components.
    ///
    /// Returns [`DateError::MonthOutOfRange`] or
    /// [`DateError::DayOutOfRange`] for impossible dates. Leap years follow the
    /// Gregorian rule (divisible by 4, except centuries not divisible by 400).
    pub fn from_ymd(year: u16, month: u8, day: u8) -> Result<Self, DateError> {
        if !(1..=12).contains(&month) {
            return Err(DateError::MonthOutOfRange { month });
        }
        let maximum = days_in_month(year, month);
        if day == 0 || day > maximum {
            return Err(DateError::DayOutOfRange { year, month, day });
        }
        Ok(Self { year, month, day })
    }

    /// Parses the `xs:date` lexical form.
    ///
    /// The parser requires a four-digit year, `MM` and `DD`, rejects
    /// non-calendar dates, and accepts an optional `Z` or `±HH:MM` time-zone
    /// suffix. The suffix is validated and discarded: the model stores the
    /// civil date. `P1 §7` leaves time-zone handling of `xs:date` open, so a
    /// syntactically valid offset is tolerated instead of producing a false
    /// positive.
    pub fn parse(input: &str) -> Result<Self, DateError> {
        // `get(..10)` fails only for short input or a non-char-boundary split;
        // both are lexical failures. The suffix starts at a known boundary.
        let date = input.get(..10).ok_or(DateError::InvalidFormat)?;
        let suffix = input.get(10..).unwrap_or_default();
        validate_timezone(suffix)?;

        let bytes = date.as_bytes();
        if bytes.get(4) != Some(&b'-') || bytes.get(7) != Some(&b'-') {
            return Err(DateError::InvalidFormat);
        }
        // The separators are at 4 and 7, so these slices are in range; a
        // non-boundary split degrades to an empty string and fails the digit
        // check instead of panicking.
        let year_text = date.get(..4).unwrap_or_default();
        let month_text = date.get(5..7).unwrap_or_default();
        let day_text = date.get(8..10).unwrap_or_default();

        let year = parse_digits(year_text, 4)?;
        let month = parse_small_digits(month_text)?;
        let day = parse_small_digits(day_text)?;
        Self::from_ymd(year, month, day)
    }

    /// Returns the year (`0000..=9999`).
    #[must_use]
    pub const fn year(self) -> u16 {
        self.year
    }

    /// Returns the month (`1..=12`).
    #[must_use]
    pub const fn month(self) -> u8 {
        self.month
    }

    /// Returns the day of the month (`1..=31`).
    #[must_use]
    pub const fn day(self) -> u8 {
        self.day
    }

    /// Returns `true` when `year` is a Gregorian leap year.
    ///
    /// Exposed because date-window rules (E4.1, E4.3) need the same calendar
    /// arithmetic as the parser.
    #[must_use]
    // The divisors are non-zero constants and `year` is a `u16`; there is no
    // overflow and no possibility of division by zero.
    #[allow(clippy::arithmetic_side_effects)]
    pub const fn is_leap_year(year: u16) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }
}

/// Number of days in `month` of `year`.
///
/// `month` must be in `1..=12`; callers reach this through
/// [`CivilDate::from_ymd`] after validating the month.
// The match covers every valid month; the fallback is unreachable for validated
// input and returns zero rather than panicking.
const fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if CivilDate::is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

// The accumulator is bounded by `10^4` and the loop only runs over ASCII
// digits, so neither the multiplication nor the addition can overflow.
#[allow(clippy::arithmetic_side_effects)]
fn parse_digits(text: &str, width: usize) -> Result<u16, DateError> {
    if text.len() != width {
        return Err(DateError::InvalidFormat);
    }
    let mut value = 0_u16;
    for byte in text.bytes() {
        if !byte.is_ascii_digit() {
            return Err(DateError::InvalidFormat);
        }
        value = value * 10 + u16::from(byte - b'0');
    }
    Ok(value)
}

/// Parses an exactly two-digit month or day into a `u8`.
///
/// The value is at most `99`, so it always fits; the accumulator arithmetic is
/// bounded and the digit check precedes it.
#[allow(clippy::arithmetic_side_effects)]
fn parse_small_digits(text: &str) -> Result<u8, DateError> {
    if text.len() != 2 {
        return Err(DateError::InvalidFormat);
    }
    let mut value = 0_u8;
    for byte in text.bytes() {
        if !byte.is_ascii_digit() {
            return Err(DateError::InvalidFormat);
        }
        value = value * 10 + (byte - b'0');
    }
    Ok(value)
}

/// Validates and discards an `xs:date` time-zone suffix.
fn validate_timezone(suffix: &str) -> Result<(), DateError> {
    if suffix.is_empty() || suffix == "Z" {
        return Ok(());
    }
    if suffix.len() != 6 {
        return Err(DateError::InvalidTimezone);
    }
    let sign = suffix.as_bytes().first().copied();
    if !matches!(sign, Some(b'+' | b'-')) {
        return Err(DateError::InvalidTimezone);
    }
    // `suffix` is six bytes and starts with a sign, so these slices are in
    // range; the digit checks below reject malformed bodies.
    let body = suffix.get(1..).unwrap_or_default();
    let hour_text = body.get(..2).unwrap_or_default();
    let colon = body.as_bytes().get(2).copied();
    let minute_text = body.get(3..).unwrap_or_default();
    if colon != Some(b':') {
        return Err(DateError::InvalidTimezone);
    }
    let hour = parse_digits(hour_text, 2).map_err(|_| DateError::InvalidTimezone)?;
    let minute = parse_digits(minute_text, 2).map_err(|_| DateError::InvalidTimezone)?;
    // `xs:date` allows offsets up to ±14:00, and 14:00 has no minutes.
    if hour > 14 || minute > 59 || (hour == 14 && minute != 0) {
        return Err(DateError::InvalidTimezone);
    }
    Ok(())
}

impl FromStr for CivilDate {
    type Err = DateError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse(input)
    }
}

impl fmt::Display for CivilDate {
    /// Renders the canonical `YYYY-MM-DD` form without a time zone.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        )
    }
}

/// A malformed or impossible calendar date.
///
/// The variants are allocation-free and stable enough for diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateError {
    /// The input is not `YYYY-MM-DD` (or the optional time zone is malformed).
    InvalidFormat,
    /// The numeric time-zone offset is outside `-14:00..=+14:00`.
    InvalidTimezone,
    /// The month is not in `1..=12`.
    MonthOutOfRange {
        /// The rejected month.
        month: u8,
    },
    /// The day is zero or exceeds the number of days in that month.
    DayOutOfRange {
        /// The year of the rejected date.
        year: u16,
        /// The month of the rejected date.
        month: u8,
        /// The rejected day.
        day: u8,
    },
}

impl fmt::Display for DateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => {
                formatter.write_str("date does not match the xs:date lexical form")
            }
            Self::InvalidTimezone => formatter.write_str("invalid xs:date time-zone offset"),
            Self::MonthOutOfRange { month } => {
                write!(formatter, "month {month} is outside 1..=12")
            }
            Self::DayOutOfRange { year, month, day } => {
                write!(
                    formatter,
                    "{day} is not a valid day in {year:04}-{month:02}"
                )
            }
        }
    }
}

impl std::error::Error for DateError {}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        clippy::assert_is_empty
    )]

    use super::{CivilDate, DateError};

    #[test]
    fn parses_canonical_dates() {
        for (text, year, month, day) in [
            ("1970-01-01", 1970, 1, 1),
            ("2000-02-29", 2000, 2, 29),
            ("2023-02-28", 2023, 2, 28),
            ("2024-12-31", 2024, 12, 31),
            ("0000-01-01", 0, 1, 1),
            ("9999-12-31", 9999, 12, 31),
        ] {
            let date = CivilDate::parse(text).unwrap();
            assert_eq!((date.year(), date.month(), date.day()), (year, month, day));
            assert_eq!(date.to_string(), text);
        }
    }

    #[test]
    fn accepts_and_validates_the_optional_timezone() {
        for text in [
            "2024-01-02Z",
            "2024-01-02+02:00",
            "2024-01-02-14:00",
            "2024-01-02+00:00",
            "2024-01-02+02:59",
        ] {
            assert_eq!(CivilDate::parse(text).unwrap().to_string(), "2024-01-02");
        }
        for text in [
            "2024-01-02x",
            // Six-byte suffix that does not start with a sign.
            "2024-01-02X02:00",
            "2024-01-02+2:00",
            "2024-01-02+02-00",
            "2024-01-02+ab:00",
            "2024-01-02+15:00",
            "2024-01-02+02:60",
            "2024-01-02+02:ab",
            "2024-01-02+14:01",
            // `get(1..)` cannot fail for a 6-byte suffix that starts with a
            // sign, but the missing-colon and short-body checks are still
            // exercised by the cases above.
        ] {
            assert_eq!(CivilDate::parse(text), Err(DateError::InvalidTimezone));
        }
    }

    #[test]
    fn rejects_len_short_and_extra_components() {
        for text in [
            "",
            "2024",
            "2024-01",
            "2024-01-0",
            "2024-1-02",
            // Ten characters, but the year is five digits.
            "12345-1-01",
            // Ten characters, but the month is one digit.
            "2024-1-002",
        ] {
            assert_eq!(CivilDate::parse(text), Err(DateError::InvalidFormat));
        }
        assert_eq!(
            CivilDate::parse("2024-01-02-03"),
            Err(DateError::InvalidTimezone)
        );
        assert_eq!(
            CivilDate::parse("2024-01-02+02:00:00"),
            Err(DateError::InvalidTimezone)
        );
    }

    #[test]
    fn rejects_impossible_calendar_dates() {
        assert_eq!(
            CivilDate::parse("2023-02-29"),
            Err(DateError::DayOutOfRange {
                year: 2023,
                month: 2,
                day: 29
            })
        );
        assert_eq!(
            CivilDate::parse("2024-04-31"),
            Err(DateError::DayOutOfRange {
                year: 2024,
                month: 4,
                day: 31
            })
        );
        assert_eq!(
            CivilDate::parse("2024-00-01"),
            Err(DateError::MonthOutOfRange { month: 0 })
        );
        assert_eq!(
            CivilDate::parse("2024-13-01"),
            Err(DateError::MonthOutOfRange { month: 13 })
        );
        assert_eq!(
            CivilDate::parse("2024-01-00"),
            Err(DateError::DayOutOfRange {
                year: 2024,
                month: 1,
                day: 0
            })
        );
    }

    #[test]
    fn rejects_non_digit_year_month_and_day() {
        for text in ["20x4-01-01", "2024-0a-01", "2024-01-0b", "2024--1-01"] {
            assert_eq!(CivilDate::parse(text), Err(DateError::InvalidFormat));
        }
    }

    #[test]
    fn from_ymd_enforces_the_calendar() {
        assert!(CivilDate::from_ymd(2024, 2, 29).is_ok());
        assert_eq!(
            CivilDate::from_ymd(2024, 2, 30),
            Err(DateError::DayOutOfRange {
                year: 2024,
                month: 2,
                day: 30
            })
        );
        assert_eq!(
            CivilDate::from_ymd(2024, 13, 1),
            Err(DateError::MonthOutOfRange { month: 13 })
        );
        assert_eq!(
            CivilDate::from_ymd(2024, 1, 0),
            Err(DateError::DayOutOfRange {
                year: 2024,
                month: 1,
                day: 0
            })
        );
    }

    #[test]
    fn helper_digit_parsers_reject_wrong_widths() {
        assert_eq!(super::parse_digits("123", 4), Err(DateError::InvalidFormat));
        assert_eq!(
            super::parse_small_digits("1"),
            Err(DateError::InvalidFormat)
        );
    }

    #[test]
    fn dash_positions_are_checked_before_slicing() {
        assert_eq!(
            CivilDate::parse("2024010102"),
            Err(DateError::InvalidFormat)
        );
        assert_eq!(
            CivilDate::parse("2024-01010"),
            Err(DateError::InvalidFormat)
        );
        assert_eq!(
            CivilDate::parse("2024_01_02"),
            Err(DateError::InvalidFormat)
        );
    }

    #[test]
    fn month_table_has_a_total_fallback() {
        assert_eq!(super::days_in_month(2024, 0), 0);
        assert_eq!(super::days_in_month(2024, 13), 0);
        assert_eq!(super::days_in_month(2024, 2), 29);
        assert_eq!(super::days_in_month(2023, 2), 28);
    }

    #[test]
    fn leap_year_rule_is_gregorian() {
        assert!(CivilDate::is_leap_year(2000));
        assert!(!CivilDate::is_leap_year(1900));
        assert!(!CivilDate::is_leap_year(2023));
        assert!(CivilDate::is_leap_year(2024));
        assert_eq!(
            CivilDate::from_ymd(1900, 2, 29),
            Err(DateError::DayOutOfRange {
                year: 1900,
                month: 2,
                day: 29
            })
        );
    }

    #[test]
    fn ordering_is_chronological() {
        let earlier = CivilDate::parse("2023-12-31").unwrap();
        let later = CivilDate::parse("2024-01-01").unwrap();
        assert!(earlier < later);
        assert_eq!(
            "2024-01-01".parse::<CivilDate>().unwrap(),
            CivilDate::from_ymd(2024, 1, 1).unwrap()
        );
    }

    #[test]
    fn errors_render_stable_messages() {
        assert!(!DateError::InvalidFormat.to_string().is_empty());
        assert!(!DateError::InvalidTimezone.to_string().is_empty());
        assert!(
            DateError::MonthOutOfRange { month: 13 }
                .to_string()
                .contains("13")
        );
        assert!(
            DateError::DayOutOfRange {
                year: 2024,
                month: 2,
                day: 30
            }
            .to_string()
            .contains("2024-02")
        );
        let error: &dyn std::error::Error = &DateError::InvalidFormat;
        assert!(error.source().is_none());
    }
}
