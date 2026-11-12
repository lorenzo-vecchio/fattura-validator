//! Severity and the severity policy hook.
//!
//! Severities are defined by `PLAN §1.3`: `error` means the `SdI` would reject
//! the file according to the rules encoded here, `warning` means likely
//! rejection, a deprecated construct or ambiguity, and `info` is domain advice or
//! a disclosure. A rule never hard-codes policy decisions in its emission path:
//! it declares a default severity and a [`SeverityProfile`] can cap, promote or
//! demote it without touching the rule (`PLAN §3`, risk `R6` in `PLAN §10`).

use core::cmp::Ordering;
use core::fmt;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::finding::Finding;
use super::rule::RuleId;

/// Severity of a finding.
///
/// Ordering is semantic: `Info < Warning < Error`, so `Error` is the maximum.
/// The `SdI` contract serializes the lowercase form.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Domain advice or a statement about what was skipped; never blocking.
    Info,
    /// Likely rejection, deprecated construct or ambiguity; must be reviewed.
    Warning,
    /// The `SdI` would reject the file according to the rules encoded here.
    Error,
}

impl Severity {
    /// Returns the semantic rank, with `Error` highest.
    #[must_use]
    pub const fn rank(self) -> u8 {
        match self {
            Self::Info => 0,
            Self::Warning => 1,
            Self::Error => 2,
        }
    }
}

impl Ord for Severity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl PartialOrd for Severity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Severity {
    /// Writes `info`, `warning` or `error` without allocating.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        };
        formatter.write_str(text)
    }
}

/// Named severity policy selected by the caller.
///
/// * [`SeverityMode::Sdi`] — the default: rule-declared severities are kept.
/// * [`SeverityMode::Recommended`] — checks without an official `SdI` code are
///   capped at `warning`, so recommended advice can never become a false error.
/// * [`SeverityMode::Permissive`] — every finding is capped at `warning`.
///
/// Per-rule overrides are applied after the mode and take precedence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SeverityMode {
    /// Keep severities as declared by the rules.
    #[default]
    Sdi,
    /// Cap findings without an official code at `warning`.
    Recommended,
    /// Cap every finding at `warning`.
    Permissive,
}

impl fmt::Display for SeverityMode {
    /// Writes `sdi`, `recommended` or `permissive` without allocating.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Sdi => "sdi",
            Self::Recommended => "recommended",
            Self::Permissive => "permissive",
        };
        formatter.write_str(text)
    }
}

/// Severity policy applied while assembling a [`Report`](crate::diag::Report).
///
/// The policy is a value owned by the report builder — there are no globals, so
/// tests and embedders can run different policies concurrently. It supports the
/// three modes of `PLAN §3` plus exact per-rule overrides for promoting or
/// demoting a single rule without changing that rule's code.
///
/// ```
/// use fattura_core::diag::{RuleId, Severity, SeverityProfile};
///
/// let profile = SeverityProfile::permissive()
///     .with_override(RuleId::new("ade.00423"), Severity::Error);
/// assert_eq!(
///     profile.override_for(RuleId::new("ade.00423")),
///     Some(Severity::Error)
/// );
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SeverityProfile {
    mode: SeverityMode,
    overrides: BTreeMap<RuleId, Severity>,
}

impl SeverityProfile {
    /// Creates an empty profile in the given mode.
    #[must_use]
    pub fn new(mode: SeverityMode) -> Self {
        Self {
            mode,
            overrides: BTreeMap::new(),
        }
    }

    /// Creates the default `SdI` profile.
    // Equivalent to `Self::default()`: the derived `Default` is `SdI` mode with
    // no overrides. The mutant is indistinguishable by any test, so it is
    // explicitly skipped rather than covered by a meaningless assertion.
    #[cfg_attr(test, mutants::skip)]
    #[must_use]
    pub fn sdi() -> Self {
        Self::new(SeverityMode::Sdi)
    }

    /// Creates a profile that caps uncoded findings at `warning`.
    #[must_use]
    pub fn recommended() -> Self {
        Self::new(SeverityMode::Recommended)
    }

    /// Creates a profile that caps every finding at `warning`.
    #[must_use]
    pub fn permissive() -> Self {
        Self::new(SeverityMode::Permissive)
    }

    /// Returns the active mode.
    #[must_use]
    pub const fn mode(&self) -> SeverityMode {
        self.mode
    }

    /// Replaces the mode, consuming the profile.
    #[must_use]
    pub fn with_mode(mut self, mode: SeverityMode) -> Self {
        self.mode = mode;
        self
    }

    /// Adds an exact per-rule override, consuming the profile.
    #[must_use]
    pub fn with_override(mut self, rule: RuleId, severity: Severity) -> Self {
        self.set_override(rule, severity);
        self
    }

    /// Adds or replaces a per-rule override.
    pub fn set_override(&mut self, rule: RuleId, severity: Severity) {
        self.overrides.insert(rule, severity);
    }

    /// Removes a per-rule override, returning the previous value.
    pub fn clear_override(&mut self, rule: RuleId) -> Option<Severity> {
        self.overrides.remove(&rule)
    }

    /// Returns the override configured for a rule, if any.
    #[must_use]
    pub fn override_for(&self, rule: RuleId) -> Option<Severity> {
        self.overrides.get(&rule).copied()
    }

    /// Computes the effective severity for a finding.
    ///
    /// The mode caps the declared severity first; an override for the finding's
    /// rule then replaces the result exactly.
    #[must_use]
    pub fn apply(&self, finding: &Finding) -> Severity {
        let declared = finding.severity;
        let cap = match self.mode {
            SeverityMode::Sdi => false,
            SeverityMode::Recommended => finding.code.is_none(),
            SeverityMode::Permissive => true,
        };
        let effective = if cap {
            declared.min(Severity::Warning)
        } else {
            declared
        };
        self.overrides
            .get(&finding.rule)
            .copied()
            .unwrap_or(effective)
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

    use super::{Severity, SeverityMode, SeverityProfile};
    use crate::diag::{ElementPath, Finding, Level, RuleId, SdiCode};

    fn finding(rule: &'static str, code: Option<SdiCode>, severity: Severity) -> Finding {
        Finding::new(
            RuleId::new(rule),
            code,
            severity,
            Level::L3,
            ElementPath::root(),
            "message",
        )
    }

    #[test]
    fn severity_display_serde_and_ordering_cover_every_variant() {
        let variants = [
            (Severity::Info, "info", 0),
            (Severity::Warning, "warning", 1),
            (Severity::Error, "error", 2),
        ];

        for (severity, label, rank) in variants {
            assert_eq!(severity.to_string(), label);
            assert_eq!(severity.rank(), rank);
            assert_eq!(serde_json::to_value(severity).unwrap(), json!(label));
            assert_eq!(
                serde_json::from_value::<Severity>(json!(label)).unwrap(),
                severity
            );
        }

        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error > Severity::Info);
        assert_eq!(Severity::Info.max(Severity::Error), Severity::Error);
        assert_eq!(Severity::Error.min(Severity::Warning), Severity::Warning);
    }

    #[test]
    fn severity_mode_display_covers_every_variant() {
        assert_eq!(SeverityMode::Sdi.to_string(), "sdi");
        assert_eq!(SeverityMode::Recommended.to_string(), "recommended");
        assert_eq!(SeverityMode::Permissive.to_string(), "permissive");
    }

    #[test]
    fn severity_profile_constructors_and_accessors() {
        assert_eq!(SeverityProfile::sdi().mode(), SeverityMode::Sdi);
        assert_eq!(
            SeverityProfile::recommended().mode(),
            SeverityMode::Recommended
        );
        assert_eq!(
            SeverityProfile::permissive().mode(),
            SeverityMode::Permissive
        );
        assert_eq!(
            SeverityProfile::new(SeverityMode::Sdi),
            SeverityProfile::sdi()
        );
        assert_eq!(
            SeverityProfile::sdi().with_mode(SeverityMode::Permissive),
            SeverityProfile::permissive()
        );
    }

    #[test]
    fn sdi_profile_keeps_declared_severity() {
        let profile = SeverityProfile::sdi();
        assert_eq!(
            profile.apply(&finding("rec.e1", None, Severity::Error)),
            Severity::Error
        );
        assert_eq!(
            profile.apply(&finding(
                "ade.00423",
                Some(SdiCode::WRONG_LINE_TOTAL),
                Severity::Warning
            )),
            Severity::Warning
        );
        assert_eq!(
            profile.apply(&finding("rec.e2", None, Severity::Info)),
            Severity::Info
        );
    }

    #[test]
    fn recommended_profile_caps_only_uncoded_findings() {
        let profile = SeverityProfile::recommended();
        assert_eq!(
            profile.apply(&finding("rec.e1", None, Severity::Error)),
            Severity::Warning
        );
        assert_eq!(
            profile.apply(&finding("rec.e2", None, Severity::Info)),
            Severity::Info
        );
        assert_eq!(
            profile.apply(&finding(
                "ade.00423",
                Some(SdiCode::WRONG_LINE_TOTAL),
                Severity::Error
            )),
            Severity::Error
        );
    }

    #[test]
    fn permissive_profile_caps_every_finding() {
        let profile = SeverityProfile::permissive();
        assert_eq!(
            profile.apply(&finding(
                "ade.00423",
                Some(SdiCode::WRONG_LINE_TOTAL),
                Severity::Error
            )),
            Severity::Warning
        );
        assert_eq!(
            profile.apply(&finding("rec.e1", None, Severity::Error)),
            Severity::Warning
        );
        assert_eq!(
            profile.apply(&finding("rec.e2", None, Severity::Info)),
            Severity::Info
        );
    }

    #[test]
    fn overrides_promote_and_demote_after_the_mode() {
        let rule = RuleId::new("ade.00423");
        let mut profile = SeverityProfile::permissive().with_override(rule, Severity::Error);

        assert_eq!(profile.override_for(rule), Some(Severity::Error));
        assert_eq!(
            profile.apply(&finding(
                "ade.00423",
                Some(SdiCode::WRONG_LINE_TOTAL),
                Severity::Error
            )),
            Severity::Error,
            "the override wins over the permissive cap"
        );

        profile.set_override(RuleId::new("rec.e3"), Severity::Info);
        assert_eq!(
            profile.apply(&finding("rec.e3", None, Severity::Error)),
            Severity::Info
        );

        assert_eq!(
            profile.clear_override(RuleId::new("ade.00423")),
            Some(Severity::Error)
        );
        assert_eq!(profile.override_for(rule), None);
        assert_eq!(profile.clear_override(rule), None);
    }
}
