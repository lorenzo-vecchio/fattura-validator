//! Integration tests for the L1 streaming scanner over fixture documents and
//! adversarial inputs.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::cast_possible_truncation,
    clippy::assert_is_empty
)]

use fattura_core::diag::{Level, Severity};
use fattura_core::xml::{Limits, RULE_DEPTH_LIMIT, RULE_TEXT_LIMIT, RootKind, scan};

const FPR12: &str = include_str!("data/xml/fpr12_mixed_prefix.xml");
const FSM10: &str = include_str!("data/xml/fsm10_default_prefix.xml");

#[test]
fn scans_official_like_fpr12_with_mixed_prefixes_and_schema_location() {
    let result = scan(FPR12.as_bytes(), Limits::default());

    assert!(result.completed, "{:?}", result.findings);
    assert!(result.is_well_formed());
    assert!(result.findings.is_empty(), "{:?}", result.findings);
    assert_eq!(result.max_depth, 6);

    let root = result.root.expect("root");
    assert_eq!(root.local_name(), b"FatturaElettronica");
    assert_eq!(root.prefix(), Some(b"p".as_slice()));
    assert_eq!(root.kind(), Some(RootKind::Invoice));
    assert_eq!(root.version(), Some("FPR12"));
    assert_eq!(
        root.namespace(),
        Some("http://ivaservizi.agenziaentrate.gov.it/docs/xsd/fatture/v1.2")
    );
}

#[test]
fn scans_simplified_default_namespace_fixture() {
    let result = scan(FSM10.as_bytes(), Limits::default());

    assert!(result.completed);
    assert!(result.findings.is_empty(), "{:?}", result.findings);

    let root = result.root.expect("root");
    assert_eq!(root.local_name(), b"FatturaElettronicaSemplificata");
    assert_eq!(root.prefix(), None);
    assert_eq!(root.kind(), Some(RootKind::Simplified));
    assert_eq!(root.version(), Some("FSM10"));
    assert_eq!(
        root.namespace(),
        Some("http://ivaservizi.agenziaentrate.gov.it/docs/xsd/fatture/v1.0")
    );
}

#[test]
fn every_truncation_is_reported_and_never_panics() {
    let bytes = FPR12.as_bytes();
    for cut in 0..=bytes.len() {
        let result = scan(&bytes[..cut], Limits::default());
        if result.completed {
            if result.root.is_some() {
                assert!(
                    result.is_well_formed(),
                    "cut at {cut} completed with errors: {:?}",
                    result.findings
                );
            } else {
                assert!(
                    !result.is_well_formed(),
                    "cut at {cut} completed without a root and without a finding"
                );
            }
        } else {
            assert!(
                !result.findings.is_empty(),
                "cut at {cut} stopped without a finding"
            );
            assert_eq!(result.findings[0].level, Level::L1);
        }
    }
}

#[test]
fn injected_bytes_are_reported_and_never_panic() {
    let base = FPR12.as_bytes();
    for value in 0..=u8::MAX {
        for position in [0usize, 1, 64, 512, base.len().saturating_sub(1)] {
            let mut bytes = base.to_vec();
            bytes[position] = value;
            let result = scan(&bytes, Limits::default());
            if result.completed {
                assert!(result.is_well_formed(), "byte {value:#04x} at {position}");
            } else {
                assert!(!result.findings.is_empty());
            }
        }
    }
}

#[test]
fn deep_nesting_hits_the_guard_instead_of_overflowing() {
    let mut document = String::from("<r>");
    for _ in 0..10_000 {
        document.push_str("<a>");
    }
    document.push_str("</a></r>");

    let result = scan(document.as_bytes(), Limits::default());
    assert!(!result.completed);
    assert_eq!(result.findings.len(), 1);
    let finding = &result.findings[0];
    assert_eq!(finding.rule, RULE_DEPTH_LIMIT);
    assert_eq!(finding.severity, Severity::Warning);
    assert_eq!(finding.level, Level::L1);
    assert_eq!(finding.code, None);
    assert!(finding.message.contains("64"));
}

#[test]
fn long_text_hits_the_guard_instead_of_allocating_a_model() {
    let mut document = String::with_capacity(2 * 1_048_576 + 8);
    document.push_str("<r>");
    document.push_str(&"x".repeat(1_048_577));
    document.push_str("</r>");

    let result = scan(document.as_bytes(), Limits::default());
    assert!(!result.completed);
    assert_eq!(result.findings.len(), 1);
    let finding = &result.findings[0];
    assert_eq!(finding.rule, RULE_TEXT_LIMIT);
    assert_eq!(finding.severity, Severity::Warning);
}

#[test]
fn text_whitespace_is_preserved_and_never_trimmed() {
    let mut reader = fattura_core::xml::Reader::new(
        b"<r><Descrizione>  due spazi  </Descrizione></r>",
        Limits::default(),
    );
    let mut texts = Vec::new();
    loop {
        match reader.read_event().unwrap() {
            fattura_core::xml::Event::Text(text) => texts.push(text.raw().to_vec()),
            fattura_core::xml::Event::Eof => break,
            _ => {}
        }
    }
    assert_eq!(texts, vec![b"  due spazi  ".to_vec()]);
}
