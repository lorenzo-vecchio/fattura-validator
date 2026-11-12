//! JSON contract snapshots and determinism tests for the whole diagnostic
//! report (`PLAN §4`, `PLAN §7`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]

use fattura_core::REPORT_SCHEMA_VERSION;
use fattura_core::diag::{
    ElementPath, Finding, Level, LevelSummary, NotChecked, Report, RuleId, SdiCode, Severity,
    SeverityProfile,
};

fn sample_report() -> Report {
    let mut builder = Report::builder();
    builder.mark_checked(Level::L0);
    builder.mark_checked(Level::L1);
    builder.mark_checked(Level::L2);
    builder.mark_checked(Level::L3);

    builder.push(
        Finding::new(
            RuleId::new("rec.e11"),
            None,
            Severity::Info,
            Level::L3,
            ElementPath::root()
                .with_index("Body", 2)
                .with_name("DatiTrasmissione")
                .with_name("PECDestinatario"),
            "recipient PEC uses a permissive RFC-style form",
        )
        .with_hint("P3 §5.6: delivery is not verifiable offline"),
    );

    builder.push(Finding::new(
        RuleId::new("ade.00423"),
        Some(SdiCode::WRONG_LINE_TOTAL),
        Severity::Error,
        Level::L3,
        ElementPath::root()
            .with_index("Body", 2)
            .with_name("DatiBeniServizi")
            .with_index("DettaglioLinee", 3)
            .with_name("PrezzoTotale"),
        "PrezzoTotale does not equal quantity times unit price",
    ));

    builder.push(Finding::new(
        RuleId::new("ade.00200"),
        Some(SdiCode::SCHEMA_VIOLATION),
        Severity::Warning,
        Level::L2,
        ElementPath::root()
            .with_index("Body", 2)
            .with_name("DatiGenerali")
            .with_name("DatiGeneraliDocumento")
            .with_name("Data"),
        "date does not match the xs:date lexical form",
    ));

    builder.push(Finding::new(
        RuleId::new("ade.00001"),
        Some(SdiCode::INVALID_FILE_NAME),
        Severity::Error,
        Level::L0,
        ElementPath::root().with_name("File").with_name("Nome"),
        "file name does not match the required shape",
    ));

    builder.push_not_checked(NotChecked::new(
        Some(SdiCode::REVOKED_SIGNING_CERTIFICATE),
        "requires CRL/OCSP revocation data",
    ));
    builder.push_not_checked(NotChecked::new(
        Some(SdiCode::DUPLICATE_INVOICE),
        "requires the SdI invoice archive",
    ));
    builder.push_not_checked(NotChecked::new(
        None,
        "L5 delivery notifications and recipient outcomes are outside offline scope",
    ));

    builder.build()
}

#[test]
fn diag_report_json_contract_snapshot() {
    let report = sample_report();

    assert_eq!(report.schema_version, REPORT_SCHEMA_VERSION);
    assert_eq!(
        report.levels.get(&Level::L1).copied(),
        Some(LevelSummary::Passed)
    );
    assert_eq!(
        report.levels.get(&Level::L3).copied(),
        Some(LevelSummary::Findings(2))
    );

    insta::assert_json_snapshot!("report_contract", report);
}

#[test]
fn diag_report_json_is_stable_for_reordered_input() {
    let report = sample_report();

    let mut reversed = Report::builder();
    reversed.mark_checked(Level::L3);
    reversed.mark_checked(Level::L2);
    reversed.mark_checked(Level::L1);
    reversed.mark_checked(Level::L0);
    for finding in report.findings.iter().rev() {
        reversed.push(finding.clone());
    }
    for entry in report.not_checked.iter().rev() {
        reversed.push_not_checked(entry.clone());
    }

    let reversed = reversed.build();
    assert_eq!(reversed, report);
    assert_eq!(
        serde_json::to_string_pretty(&reversed).unwrap(),
        serde_json::to_string_pretty(&report).unwrap()
    );
}

#[test]
fn diag_report_applies_severity_profile_without_touching_rules() {
    let mut builder = Report::builder();
    builder.set_profile(
        SeverityProfile::permissive().with_override(RuleId::new("ade.00423"), Severity::Error),
    );
    builder.push(Finding::new(
        RuleId::new("ade.00423"),
        Some(SdiCode::WRONG_LINE_TOTAL),
        Severity::Error,
        Level::L3,
        ElementPath::root().with_name("Body"),
        "wrong total",
    ));
    builder.push(Finding::new(
        RuleId::new("ade.00200"),
        Some(SdiCode::SCHEMA_VIOLATION),
        Severity::Error,
        Level::L2,
        ElementPath::root().with_name("Body"),
        "schema",
    ));

    let report = builder.build();
    let severities: Vec<Severity> = report
        .findings
        .iter()
        .map(|finding| finding.severity)
        .collect();

    assert_eq!(
        severities,
        vec![Severity::Warning, Severity::Error],
        "permissive caps uncoded/other errors; the explicit override wins"
    );
}
