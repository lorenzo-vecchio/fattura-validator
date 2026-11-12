//! Stable rule identifiers.
//!
//! Every finding names the rule that produced it, independently of the official
//! `SdI` code: `ade.00423` for a `00423` control, `rec.e3` for recommended check
//! `E3` (`PLAN §4`, `P3 §4`). Rule ids are part of the JSON contract; renaming a
//! rule is a contract change.

use core::fmt;

use serde::Serialize;

/// Stable identifier of a validation rule.
///
/// The text is a `'static` literal owned by the rule module, so emitting a
/// finding never allocates to name its rule. The format is
/// `ade.<code>` for `SdI` controls and `rec.e<n>` for recommended checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct RuleId(&'static str);

impl RuleId {
    /// Creates a rule id from a static literal.
    ///
    /// The constructor deliberately does not parse or validate the text: rule
    /// ids are authored in source code, not derived from user input.
    #[must_use]
    pub const fn new(id: &'static str) -> Self {
        Self(id)
    }

    /// Returns the rule id as written in the report contract.
    ///
    /// No allocation is performed.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl From<&'static str> for RuleId {
    fn from(value: &'static str) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for RuleId {
    /// Writes the rule id without allocating.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl Serialize for RuleId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
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

    use serde_json::json;

    use super::RuleId;

    #[test]
    fn rule_id_renders_serializes_and_orders() {
        let ade = RuleId::new("ade.00423");
        let rec = RuleId::from("rec.e3");

        assert_eq!(ade.as_str(), "ade.00423");
        assert_eq!(ade.to_string(), "ade.00423");
        assert_eq!(rec.to_string(), "rec.e3");
        assert_eq!(serde_json::to_value(ade).unwrap(), json!("ade.00423"));
        assert_eq!(serde_json::to_value(rec).unwrap(), json!("rec.e3"));
        assert!(ade < rec);
        assert_eq!(ade, RuleId::new("ade.00423"));
    }
}
