//! The layered report and its deterministic builder.
//!
//! [`Report`] is the public result of validation. It follows `PLAN §4`: a
//! schema version, per-layer summaries, findings and explicit disclosures for
//! what could not be checked (`PLAN §5.2`, `P3 §7`). The JSON projection of this
//! type is the cross-language contract.
//!
//! [`ReportBuilder`] owns all ordering and deduplication so callers cannot
//! produce a non-deterministic report: findings are sorted by rendered element
//! path, then official code, then rule id, and identical findings are collapsed.

use core::cmp::Ordering;
use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::REPORT_SCHEMA_VERSION;

use super::code::SdiCode;
use super::finding::Finding;
use super::level::Level;
use super::severity::SeverityProfile;

/// Aggregate outcome of one validation layer.
///
/// The three states mirror `PLAN §4`:
///
/// * `Checked` — the layer was evaluated but this report makes no pass/fail
///   claim (used by partially assembled reports).
/// * `Passed` — the layer was evaluated and emitted no findings.
/// * `Findings(n)` — the layer was evaluated and emitted `n` findings.
///
/// [`ReportBuilder`] emits `Passed` and `Findings`; `Checked` is available for
/// callers that record a layer without an aggregate count.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", content = "count", rename_all = "snake_case")]
pub enum LevelSummary {
    /// The layer was evaluated without asserting a pass/fail aggregate.
    Checked,
    /// The layer was evaluated and emitted no findings.
    Passed,
    /// The layer was evaluated and emitted the given number of findings.
    Findings(usize),
}

impl fmt::Display for LevelSummary {
    /// Writes `checked`, `passed` or `findings(n)` without allocating.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Checked => formatter.write_str("checked"),
            Self::Passed => formatter.write_str("passed"),
            Self::Findings(count) => write!(formatter, "findings({count})"),
        }
    }
}

/// A check that the offline validator did not perform.
///
/// Every entry names the official code when one exists and states the reason the
/// check needs external data (`PLAN §5.2`, `P3 §7`). An entry without a code
/// covers a whole lifecycle area, such as delivery notifications.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NotChecked {
    /// Official `SdI` code, or `None` when the skipped check has no single code.
    pub code: Option<SdiCode>,
    /// Why the check could not be performed offline.
    pub reason: String,
}

impl NotChecked {
    /// Creates a disclosure entry.
    #[must_use]
    pub fn new(code: Option<SdiCode>, reason: impl Into<String>) -> Self {
        Self {
            code,
            reason: reason.into(),
        }
    }
}

/// The layered result of a validation run.
///
/// The `schema_version` field always equals
/// [`REPORT_SCHEMA_VERSION`](crate::REPORT_SCHEMA_VERSION); a contract change
/// increments that constant and refreshes the snapshots. Field order and names
/// are stable and snapshot-tested.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Report {
    /// JSON contract version, from
    /// [`REPORT_SCHEMA_VERSION`](crate::REPORT_SCHEMA_VERSION).
    pub schema_version: &'static str,
    /// Outcome of every layer that was evaluated, ordered `L0` through `L5`.
    pub levels: BTreeMap<Level, LevelSummary>,
    /// Findings after severity policy, deterministic sorting and deduplication.
    pub findings: Vec<Finding>,
    /// Checks that could not be performed offline, deterministically sorted.
    pub not_checked: Vec<NotChecked>,
}

impl Report {
    /// Starts a new report builder with the default `SdI` severity profile.
    // Equivalent to `ReportBuilder::default()`; the mutation cannot change any
    // observable behaviour, so it is explicitly skipped.
    #[cfg_attr(test, mutants::skip)]
    #[must_use]
    pub fn builder() -> ReportBuilder {
        ReportBuilder::new()
    }
}

/// Deterministic assembler for a [`Report`].
///
/// The builder applies the configured [`SeverityProfile`], sorts findings by
/// `(rendered path, code, rule, severity, level, message, hint)`, removes
/// identical findings, sorts and deduplicates the not-checked disclosures, and
/// derives one [`LevelSummary`] per evaluated layer.
///
/// ```
/// use fattura_core::diag::{
///     ElementPath, Finding, Level, Report, RuleId, Severity, SdiCode,
/// };
///
/// let mut builder = Report::builder();
/// builder.mark_checked(Level::L1);
/// builder.push(Finding::new(
///     RuleId::new("ade.00423"),
///     Some(SdiCode::WRONG_LINE_TOTAL),
///     Severity::Error,
///     Level::L3,
///     ElementPath::root().with_index("Body", 1),
///     "wrong line total",
/// ));
///
/// let report = builder.build();
/// assert_eq!(report.levels.get(&Level::L3).copied(), Some(
///     fattura_core::diag::LevelSummary::Findings(1)
/// ));
/// ```
#[derive(Clone, Debug)]
pub struct ReportBuilder {
    profile: SeverityProfile,
    findings: Vec<Finding>,
    not_checked: Vec<NotChecked>,
    checked_levels: BTreeSet<Level>,
}

impl ReportBuilder {
    /// Creates a builder with the default `SdI` profile.
    #[must_use]
    pub fn new() -> Self {
        Self::with_profile(SeverityProfile::sdi())
    }

    /// Creates a builder with the given severity policy.
    #[must_use]
    pub fn with_profile(profile: SeverityProfile) -> Self {
        Self {
            profile,
            findings: Vec::new(),
            not_checked: Vec::new(),
            checked_levels: BTreeSet::new(),
        }
    }

    /// Returns the active severity policy.
    #[must_use]
    pub fn profile(&self) -> &SeverityProfile {
        &self.profile
    }

    /// Replaces the severity policy.
    pub fn set_profile(&mut self, profile: SeverityProfile) -> &mut Self {
        self.profile = profile;
        self
    }

    /// Adds one finding and records its layer as evaluated.
    pub fn push(&mut self, finding: Finding) -> &mut Self {
        self.checked_levels.insert(finding.level);
        self.findings.push(finding);
        self
    }

    /// Adds the skipped-check disclosure to the report.
    pub fn push_not_checked(&mut self, entry: NotChecked) -> &mut Self {
        self.not_checked.push(entry);
        self
    }

    /// Records a layer as evaluated even when it produced no findings.
    pub fn mark_checked(&mut self, level: Level) -> &mut Self {
        self.checked_levels.insert(level);
        self
    }

    /// Applies policy, sorts, deduplicates and assembles the report.
    ///
    /// The JSON contract version comes from
    /// [`REPORT_SCHEMA_VERSION`](crate::REPORT_SCHEMA_VERSION). A caller that
    /// never marked a layer and never pushed a finding gets an empty `levels`
    /// map: the report does not claim that any layer passed.
    #[must_use]
    pub fn build(mut self) -> Report {
        for finding in &mut self.findings {
            finding.severity = self.profile.apply(finding);
        }

        self.findings.sort_by(compare_findings);
        self.findings.dedup();

        self.not_checked.sort();
        self.not_checked.dedup();

        let mut level_counts: BTreeMap<Level, usize> = self
            .checked_levels
            .into_iter()
            .map(|level| (level, 0))
            .collect();
        for finding in &self.findings {
            let count = level_counts.entry(finding.level).or_insert(0);
            *count = count.saturating_add(1);
        }

        let levels = level_counts
            .into_iter()
            .map(|(level, count)| {
                let summary = if count == 0 {
                    LevelSummary::Passed
                } else {
                    LevelSummary::Findings(count)
                };
                (level, summary)
            })
            .collect();

        Report {
            schema_version: REPORT_SCHEMA_VERSION,
            levels,
            findings: self.findings,
            not_checked: self.not_checked,
        }
    }
}

impl Default for ReportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Extend<Finding> for ReportBuilder {
    fn extend<T>(&mut self, findings: T)
    where
        T: IntoIterator<Item = Finding>,
    {
        for finding in findings {
            self.push(finding);
        }
    }
}

/// Total order for findings in a report.
///
/// The rendered path comes first because that is what a human reader scans;
/// the `SdI` code and rule id break ties between different diagnostics on the same
/// element. The remaining fields make the order total so deduplication is exact.
fn compare_findings(left: &Finding, right: &Finding) -> Ordering {
    left.path
        .to_string()
        .cmp(&right.path.to_string())
        .then_with(|| left.code.cmp(&right.code))
        .then_with(|| left.rule.cmp(&right.rule))
        .then_with(|| left.severity.cmp(&right.severity))
        .then_with(|| left.level.cmp(&right.level))
        .then_with(|| left.message.cmp(&right.message))
        .then_with(|| left.hint.cmp(&right.hint))
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

    use core::cmp::Ordering;

    use serde_json::json;

    use super::{LevelSummary, NotChecked, Report, ReportBuilder, compare_findings};
    use crate::REPORT_SCHEMA_VERSION;
    use crate::diag::{
        ElementPath, Finding, Level, RuleId, SdiCode, Severity, SeverityMode, SeverityProfile,
    };

    fn finding(
        rule: &'static str,
        code: Option<SdiCode>,
        severity: Severity,
        level: Level,
        path: ElementPath,
        message: &str,
        hint: Option<&str>,
    ) -> Finding {
        let finding = Finding::new(RuleId::new(rule), code, severity, level, path, message);
        match hint {
            Some(hint) => finding.with_hint(hint),
            None => finding,
        }
    }

    #[test]
    fn empty_report_claims_nothing() {
        let report = Report::builder().build();

        assert_eq!(report.schema_version, REPORT_SCHEMA_VERSION);
        assert_eq!(report.levels.len(), 0);
        assert_eq!(report.findings.len(), 0);
        assert_eq!(report.not_checked.len(), 0);
        assert_eq!(report, ReportBuilder::default().build());
    }

    #[test]
    fn level_summary_display_and_serde_cover_every_variant() {
        assert_eq!(LevelSummary::Checked.to_string(), "checked");
        assert_eq!(LevelSummary::Passed.to_string(), "passed");
        assert_eq!(LevelSummary::Findings(3).to_string(), "findings(3)");

        assert_eq!(
            serde_json::to_value(LevelSummary::Checked).unwrap(),
            json!({"state": "checked"})
        );
        assert_eq!(
            serde_json::to_value(LevelSummary::Passed).unwrap(),
            json!({"state": "passed"})
        );
        assert_eq!(
            serde_json::to_value(LevelSummary::Findings(3)).unwrap(),
            json!({"state": "findings", "count": 3})
        );
        assert_eq!(
            serde_json::from_value::<LevelSummary>(json!({"state": "findings", "count": 3}))
                .unwrap(),
            LevelSummary::Findings(3)
        );
    }

    #[test]
    fn not_checked_serializes_code_and_reason() {
        let coded = NotChecked::new(
            Some(SdiCode::REVOKED_SIGNING_CERTIFICATE),
            "requires CRL/OCSP",
        );
        let uncoded = NotChecked::new(None, "requires the SdI archive");

        assert_eq!(coded.code, Some(SdiCode::REVOKED_SIGNING_CERTIFICATE));
        assert_eq!(coded.reason, "requires CRL/OCSP");
        assert_eq!(
            serde_json::to_value(&coded).unwrap(),
            json!({"code": "00101", "reason": "requires CRL/OCSP"})
        );
        assert_eq!(
            serde_json::to_value(&uncoded).unwrap(),
            json!({"code": null, "reason": "requires the SdI archive"})
        );
        assert!(
            serde_json::from_value::<NotChecked>(
                json!({"code": "00101", "reason": "requires CRL/OCSP"})
            )
            .is_ok()
        );
    }

    #[test]
    fn builder_marks_pushed_layers_and_explicitly_checked_layers() {
        let mut builder = Report::builder();
        builder
            .mark_checked(Level::L1)
            .push(finding(
                "ade.00001",
                Some(SdiCode::INVALID_FILE_NAME),
                Severity::Error,
                Level::L0,
                ElementPath::root().with_name("File").with_name("Nome"),
                "bad file name",
                None,
            ))
            .push(finding(
                "rec.e3",
                None,
                Severity::Warning,
                Level::L3,
                ElementPath::root().with_name("Body"),
                "installment mismatch",
                None,
            ));

        let report = builder.build();

        assert_eq!(
            report.levels.get(&Level::L0),
            Some(&LevelSummary::Findings(1))
        );
        assert_eq!(report.levels.get(&Level::L1), Some(&LevelSummary::Passed));
        assert_eq!(
            report.levels.get(&Level::L3),
            Some(&LevelSummary::Findings(1))
        );
        assert!(!report.levels.contains_key(&Level::L2));
        assert_eq!(report.findings.len(), 2);
    }

    #[test]
    fn builder_sorts_by_rendered_path_then_code_then_rule() {
        let mut builder = Report::builder();
        builder
            .push(finding(
                "rec.e9",
                None,
                Severity::Info,
                Level::L3,
                ElementPath::root().with_name("Zeta"),
                "zeta",
                None,
            ))
            .push(finding(
                "rec.e3",
                None,
                Severity::Info,
                Level::L3,
                ElementPath::root()
                    .with_name("Alfa")
                    .with_index("Linea", 10),
                "line ten",
                None,
            ))
            .push(finding(
                "ade.00421",
                Some(SdiCode::WRONG_VAT_AMOUNT),
                Severity::Error,
                Level::L3,
                ElementPath::root().with_name("Alfa").with_index("Linea", 2),
                "vat",
                None,
            ))
            .push(finding(
                "rec.e1",
                None,
                Severity::Info,
                Level::L3,
                ElementPath::root().with_name("Alfa").with_index("Linea", 2),
                "uncoded first",
                None,
            ));

        let report = builder.build();
        let order: Vec<&str> = report
            .findings
            .iter()
            .map(|finding| finding.message.as_str())
            .collect();

        assert_eq!(
            order,
            vec!["line ten", "uncoded first", "vat", "zeta"],
            "rendered path string order, then None code before Some"
        );
    }

    #[test]
    fn compare_findings_is_total_and_covers_every_tie_breaker() {
        let base = finding(
            "ade.00423",
            Some(SdiCode::WRONG_LINE_TOTAL),
            Severity::Warning,
            Level::L3,
            ElementPath::root().with_name("Body"),
            "message",
            Some("hint"),
        );

        assert_eq!(compare_findings(&base, &base), Ordering::Equal);

        let mut other = base.clone();
        other.path = ElementPath::root().with_name("Altro");
        assert_eq!(compare_findings(&base, &other), Ordering::Greater);

        let mut other = base.clone();
        other.code = Some(SdiCode::WRONG_VAT_AMOUNT);
        assert_eq!(compare_findings(&base, &other), Ordering::Greater);

        let mut other = base.clone();
        other.rule = RuleId::new("rec.e3");
        assert_eq!(compare_findings(&base, &other), Ordering::Less);

        let mut other = base.clone();
        other.severity = Severity::Error;
        assert_eq!(compare_findings(&base, &other), Ordering::Less);

        let mut other = base.clone();
        other.level = Level::L2;
        assert_eq!(compare_findings(&base, &other), Ordering::Greater);

        let mut other = base.clone();
        other.message = "other".to_owned();
        assert_eq!(compare_findings(&base, &other), Ordering::Less);

        let mut other = base.clone();
        other.hint = None;
        assert_eq!(compare_findings(&base, &other), Ordering::Greater);
    }

    #[test]
    fn builder_deduplicates_identical_findings_only() {
        let repeated = finding(
            "ade.00421",
            Some(SdiCode::WRONG_VAT_AMOUNT),
            Severity::Error,
            Level::L3,
            ElementPath::root().with_name("Body").with_name("Riepilogo"),
            "vat mismatch",
            Some("check rounding"),
        );
        let distinct = finding(
            "ade.00421",
            Some(SdiCode::WRONG_VAT_AMOUNT),
            Severity::Error,
            Level::L3,
            ElementPath::root().with_name("Body").with_name("Riepilogo"),
            "vat mismatch with another amount",
            Some("check rounding"),
        );

        let mut builder = Report::builder();
        builder
            .push(repeated.clone())
            .push(distinct.clone())
            .push(repeated)
            .push(distinct);

        let report = builder.build();
        assert_eq!(report.findings.len(), 2);
        assert_eq!(report.findings[0].message, "vat mismatch");
        assert_eq!(
            report.findings[1].message,
            "vat mismatch with another amount"
        );
    }

    #[test]
    fn builder_applies_severity_profile_and_overrides() {
        let error = finding(
            "ade.00423",
            Some(SdiCode::WRONG_LINE_TOTAL),
            Severity::Error,
            Level::L3,
            ElementPath::root().with_name("Body"),
            "wrong total",
            None,
        );
        let warning = finding(
            "rec.e1",
            None,
            Severity::Warning,
            Level::L3,
            ElementPath::root().with_name("Body").with_name("Riepilogo"),
            "total mismatch",
            None,
        );

        let mut builder = ReportBuilder::with_profile(SeverityProfile::permissive());
        assert_eq!(builder.profile().mode(), SeverityMode::Permissive);
        builder.push(error.clone()).push(warning.clone());

        let report = builder.build();
        assert_eq!(report.findings[0].severity, Severity::Warning);
        assert_eq!(report.findings[1].severity, Severity::Warning);

        let mut builder = Report::builder();
        builder.set_profile(
            SeverityProfile::permissive().with_override(RuleId::new("ade.00423"), Severity::Error),
        );
        builder.push(error);
        let report = builder.build();
        assert_eq!(report.findings[0].severity, Severity::Error);

        let mut builder = Report::builder();
        builder.set_profile(SeverityProfile::recommended());
        builder.push(warning);
        let report = builder.build();
        assert_eq!(report.findings[0].severity, Severity::Warning);
    }

    #[test]
    fn builder_sorts_and_deduplicates_not_checked_entries() {
        let mut builder = Report::builder();
        builder
            .push_not_checked(NotChecked::new(
                Some(SdiCode::DUPLICATE_INVOICE),
                "requires the SdI archive",
            ))
            .push_not_checked(NotChecked::new(None, "requires delivery notifications"))
            .push_not_checked(NotChecked::new(
                Some(SdiCode::REVOKED_SIGNING_CERTIFICATE),
                "requires CRL/OCSP",
            ))
            .push_not_checked(NotChecked::new(
                Some(SdiCode::DUPLICATE_INVOICE),
                "requires the SdI archive",
            ));

        let report = builder.build();
        let rendered: Vec<(Option<u16>, &str)> = report
            .not_checked
            .iter()
            .map(|entry| (entry.code.map(SdiCode::value), entry.reason.as_str()))
            .collect();

        assert_eq!(
            rendered,
            vec![
                (None, "requires delivery notifications"),
                (Some(101), "requires CRL/OCSP"),
                (Some(404), "requires the SdI archive"),
            ]
        );
    }

    #[test]
    fn builder_is_deterministic_across_insertion_order() {
        let first = finding(
            "ade.00421",
            Some(SdiCode::WRONG_VAT_AMOUNT),
            Severity::Error,
            Level::L3,
            ElementPath::root().with_name("Body"),
            "vat",
            None,
        );
        let second = finding(
            "rec.e3",
            None,
            Severity::Info,
            Level::L3,
            ElementPath::root().with_name("Alfa"),
            "installments",
            None,
        );
        let third = finding(
            "ade.00001",
            Some(SdiCode::INVALID_FILE_NAME),
            Severity::Error,
            Level::L0,
            ElementPath::root().with_name("File"),
            "name",
            None,
        );

        let mut forward = Report::builder();
        forward
            .push(first.clone())
            .push(second.clone())
            .push(third.clone());
        let mut backward = Report::builder();
        backward.push(third).push(second).push(first);

        let forward = forward.build();
        let backward = backward.build();
        assert_eq!(forward, backward);
        assert_eq!(
            serde_json::to_string(&forward).unwrap(),
            serde_json::to_string(&backward).unwrap()
        );
    }

    #[test]
    fn builder_extend_and_checked_entries_are_supported() {
        let mut builder = Report::builder();
        builder.extend([
            finding(
                "rec.e2",
                None,
                Severity::Info,
                Level::L3,
                ElementPath::root().with_name("Body"),
                "second",
                None,
            ),
            finding(
                "rec.e1",
                None,
                Severity::Info,
                Level::L2,
                ElementPath::root().with_name("Body"),
                "first",
                None,
            ),
        ]);
        builder.mark_checked(Level::L0);

        let report = builder.build();
        assert_eq!(report.findings.len(), 2);
        assert_eq!(report.levels.get(&Level::L0), Some(&LevelSummary::Passed));
        assert_eq!(
            report.levels.get(&Level::L2),
            Some(&LevelSummary::Findings(1))
        );
        assert_eq!(
            report.levels.get(&Level::L3),
            Some(&LevelSummary::Findings(1))
        );
    }
}
