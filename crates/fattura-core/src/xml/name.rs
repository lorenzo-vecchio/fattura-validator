//! Zero-copy element and attribute names.
//!
//! The official `FatturaPA` schema declares `elementFormDefault` explicitly as
//! `unqualified`: only the root element is qualified by the target namespace and
//! every child lives in no namespace (`P1 §5.1`). Matching therefore must be
//! performed on the **local name**, namespace-agnostically, or by raw qualified
//! name when a prefix is reported for diagnostics (`P1 §5.1`, `P3 §8` plumbing
//! item 6).
//!
//! [`Name`] borrows the bytes that are already in the input. It never allocates
//! and never performs namespace resolution: prefix and local part are split at
//! the first colon only.

use core::fmt;

/// A qualified XML name borrowed from the input.
///
/// `raw` is the name exactly as written (`p:FatturaElettronica`), `prefix` is
/// the text before the first colon when present, and `local` is the remainder.
/// XML names are case-sensitive; no case folding is applied.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Name<'a> {
    raw: &'a [u8],
    local: &'a [u8],
    prefix: Option<&'a [u8]>,
}

impl<'a> Name<'a> {
    /// Decomposes a raw qualified name.
    ///
    /// A leading colon produces an empty prefix; a trailing colon produces an
    /// empty local name. Both are malformed XML, but this type is deliberately
    /// total so malformed input can be reported instead of panicking.
    #[must_use]
    pub fn new(raw: &'a [u8]) -> Self {
        match raw.iter().position(|byte| *byte == b':') {
            Some(colon) => Self {
                raw,
                local: raw.get(colon.saturating_add(1)..).unwrap_or_default(),
                prefix: raw.get(..colon),
            },
            None => Self {
                raw,
                local: raw,
                prefix: None,
            },
        }
    }

    /// Returns the name exactly as written.
    #[must_use]
    pub const fn raw(self) -> &'a [u8] {
        self.raw
    }

    /// Returns the local part, without any prefix.
    #[must_use]
    pub const fn local(self) -> &'a [u8] {
        self.local
    }

    /// Returns the prefix, or `None` when the name is unprefixed.
    #[must_use]
    pub const fn prefix(self) -> Option<&'a [u8]> {
        self.prefix
    }

    /// Returns the local part as UTF-8, or `None` when the bytes are not valid
    /// UTF-8.
    ///
    /// UTF-8 validation is deferred to this accessor so byte-level dispatch is
    /// free of decoding costs; encoding findings are owned by E2.5.
    #[must_use]
    pub fn local_str(self) -> Option<&'a str> {
        core::str::from_utf8(self.local).ok()
    }

    /// Returns the prefix as UTF-8, or `None` when absent or not valid UTF-8.
    #[must_use]
    pub fn prefix_str(self) -> Option<&'a str> {
        core::str::from_utf8(self.prefix?).ok()
    }

    /// Returns `true` when the local part matches the given bytes exactly.
    #[must_use]
    pub fn is_local(self, local: &[u8]) -> bool {
        self.local == local
    }
}

impl fmt::Debug for Name<'_> {
    /// Writes the raw name with ASCII escaping so diagnostics cannot break the
    /// surrounding line.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Name(")?;
        for &byte in self.raw {
            if byte.is_ascii_graphic() {
                fmt::Write::write_char(formatter, char::from(byte))?;
            } else {
                write!(formatter, "\\x{byte:02x}")?;
            }
        }
        formatter.write_str(")")
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )]

    use super::Name;

    #[test]
    fn name_without_prefix_keeps_raw_and_local_equal() {
        let name = Name::new(b"FatturaElettronica");
        assert_eq!(name.raw(), b"FatturaElettronica");
        assert_eq!(name.local(), b"FatturaElettronica");
        assert_eq!(name.prefix(), None);
        assert_eq!(name.prefix_str(), None);
        assert_eq!(name.local_str(), Some("FatturaElettronica"));
        assert!(name.is_local(b"FatturaElettronica"));
        assert!(!name.is_local(b"fatturaelettronica"));
        assert_eq!(format!("{name:?}"), "Name(FatturaElettronica)");
    }

    #[test]
    fn name_with_prefix_splits_at_the_first_colon() {
        let name = Name::new(b"p:FatturaElettronica");
        assert_eq!(name.raw(), b"p:FatturaElettronica");
        assert_eq!(name.local(), b"FatturaElettronica");
        assert_eq!(name.prefix(), Some(b"p".as_slice()));
        assert_eq!(name.prefix_str(), Some("p"));
        assert_eq!(name.local_str(), Some("FatturaElettronica"));
        assert!(name.is_local(b"FatturaElettronica"));

        let multi = Name::new(b"a:b:c");
        assert_eq!(multi.local(), b"b:c");
        assert_eq!(multi.prefix(), Some(b"a".as_slice()));
    }

    #[test]
    fn name_handles_empty_prefix_and_local_without_panicking() {
        let empty_local = Name::new(b"p:");
        assert_eq!(empty_local.prefix(), Some(b"p".as_slice()));
        assert_eq!(empty_local.local(), b"");
        assert_eq!(empty_local.local_str(), Some(""));

        let empty_prefix = Name::new(b":name");
        assert_eq!(empty_prefix.prefix(), Some(b"".as_slice()));
        assert_eq!(empty_prefix.local(), b"name");

        let empty = Name::new(b"");
        assert_eq!(empty.raw(), b"");
        assert_eq!(empty.local(), b"");
        assert_eq!(empty.prefix(), None);
    }

    #[test]
    fn name_utf8_accessors_return_none_for_invalid_bytes() {
        let name = Name::new(&[0xff, b':', 0xfe]);
        assert_eq!(name.prefix_str(), None);
        assert_eq!(name.local_str(), None);
        assert!(format!("{name:?}").contains("\\xff"));
        assert!(format!("{name:?}").contains("\\xfe"));
    }

    /// Fails after a fixed number of successful writes, exercising every
    /// formatter-error branch without a panic.
    struct BudgetWriter {
        remaining: usize,
    }

    impl core::fmt::Write for BudgetWriter {
        fn write_str(&mut self, _text: &str) -> core::fmt::Result {
            if self.remaining == 0 {
                return Err(core::fmt::Error);
            }
            self.remaining -= 1;
            Ok(())
        }

        fn write_char(&mut self, _value: char) -> core::fmt::Result {
            if self.remaining == 0 {
                return Err(core::fmt::Error);
            }
            self.remaining -= 1;
            Ok(())
        }
    }

    fn render_with_budget(name: &Name<'_>, budget: usize) -> core::fmt::Result {
        let mut writer = BudgetWriter { remaining: budget };
        core::fmt::write(&mut writer, format_args!("{name:?}"))
    }

    #[test]
    fn name_debug_propagates_formatter_errors() {
        let graphic = Name::new(b"ab");
        let escaped = Name::new(&[0x01]);
        for name in [graphic, escaped] {
            let success = (0..32)
                .find(|budget| render_with_budget(&name, *budget).is_ok())
                .expect("the formatter succeeds with a sufficient budget");
            assert!(success > 0, "a zero budget must fail");
            for budget in 0..success {
                assert!(render_with_budget(&name, budget).is_err());
            }
        }
    }
}
