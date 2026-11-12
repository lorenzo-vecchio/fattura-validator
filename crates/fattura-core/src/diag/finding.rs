//! A single diagnostic finding.
//!
//! A [`Finding`] is the unit every validation layer emits. It always carries the
//! rule id, the layer and the element path; the official `SdI` code is optional
//! because recommended (`E`-series) checks have none (`PLAN §4`, `P3 §4`).

use serde::Serialize;

use super::code::SdiCode;
use super::level::Level;
use super::path::ElementPath;
use super::rule::RuleId;
use super::severity::Severity;

/// One diagnostic produced by a validation rule.
///
/// Field names and JSON shapes are part of the report contract and are covered
/// by snapshots. Build one with [`Finding::new`] and attach an optional hint
/// with [`Finding::with_hint`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Finding {
    /// Stable rule identifier, for example `ade.00423` or `rec.e3`.
    pub rule: RuleId,
    /// Official `SdI` control code, when the rule maps to one.
    ///
    /// Recommended checks and informational advice have no official code.
    pub code: Option<SdiCode>,
    /// Effective severity after the report's [`SeverityProfile`] has been applied.
    ///
    /// [`SeverityProfile`]: crate::diag::SeverityProfile
    pub severity: Severity,
    /// Validation layer that produced the finding.
    pub level: Level,
    /// Path to the element responsible for the finding.
    pub path: ElementPath,
    /// Human-readable description. The `SdI` would use this text in its rejection.
    pub message: String,
    /// Optional remediation or source hint; never required for machine use.
    pub hint: Option<String>,
}

impl Finding {
    /// Creates a finding without a hint.
    ///
    /// `code` is `None` for recommended and informational checks. The declared
    /// severity is the rule's default; [`ReportBuilder`](crate::diag::ReportBuilder)
    /// applies the active severity profile when it assembles the report.
    #[must_use]
    pub fn new(
        rule: RuleId,
        code: Option<SdiCode>,
        severity: Severity,
        level: Level,
        path: ElementPath,
        message: impl Into<String>,
    ) -> Self {
        Self {
            rule,
            code,
            severity,
            level,
            path,
            message: message.into(),
            hint: None,
        }
    }

    /// Attaches a remediation or source hint, consuming the finding.
    #[must_use]
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
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

    use super::Finding;
    use crate::diag::{ElementPath, Level, RuleId, SdiCode, Severity};

    #[test]
    fn finding_serializes_stable_fields_without_hint() {
        let finding = Finding::new(
            RuleId::new("rec.e3"),
            None,
            Severity::Warning,
            Level::L3,
            ElementPath::root()
                .with_name("Body")
                .with_index("DatiPagamento", 1),
            "payment terms do not match the number of instalments",
        );

        assert_eq!(
            serde_json::to_value(&finding).unwrap(),
            json!({
                "rule": "rec.e3",
                "code": null,
                "severity": "warning",
                "level": "L3",
                "path": "Body/DatiPagamento[1]",
                "message": "payment terms do not match the number of instalments",
                "hint": null
            })
        );
    }

    #[test]
    fn finding_with_hint_and_code_serializes_both() {
        let finding = Finding::new(
            RuleId::new("ade.00423"),
            Some(SdiCode::WRONG_LINE_TOTAL),
            Severity::Error,
            Level::L3,
            ElementPath::root().with_index("Body", 2),
            "wrong line total",
        )
        .with_hint("check cascading discounts");

        assert_eq!(finding.code, Some(SdiCode::WRONG_LINE_TOTAL));
        assert_eq!(finding.severity, Severity::Error);
        assert_eq!(finding.level, Level::L3);
        assert_eq!(finding.message, "wrong line total");
        assert_eq!(finding.hint.as_deref(), Some("check cascading discounts"));
        assert_eq!(
            serde_json::to_value(&finding).unwrap(),
            json!({
                "rule": "ade.00423",
                "code": "00423",
                "severity": "error",
                "level": "L3",
                "path": "Body[2]",
                "message": "wrong line total",
                "hint": "check cascading discounts"
            })
        );
    }
}
