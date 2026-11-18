//! One streaming structural pass with L1 findings.
//!
//! [`scan`] consumes [`Reader`] events without building a model. It proves
//! well-formedness, records the root prefix/namespace for reporting, counts
//! elements and depth, and converts reader failures into deterministic
//! [`Finding`] values. The typed model of E2.2 reuses the same reader.
//!
//! Findings use `00106` — file or archive empty or corrupt — for malformed XML,
//! matching the plan's mapping of the L1 reader to that code (`PLAN §5.1`,
//! Appendix A; `P3 §2.1`). Resource-guard breaches carry no official code and
//! are warnings: they are local policy, not a claim about the `SdI` verdict.

use crate::diag::{ElementPath, Finding, Level, RuleId, SdiCode, Severity};
use crate::xml::error::{Position, ReadError};
use crate::xml::event::{Element, EntityRef, Event};
use crate::xml::local_name::{RootKind, root_kind};
use crate::xml::reader::{Limits, Reader};

/// Rule id for malformed XML and other L1 well-formedness failures.
pub const RULE_WELL_FORMED: RuleId = RuleId::new("xml.well-formedness");

/// Rule id for a rejected `DOCTYPE` declaration.
pub const RULE_FORBIDDEN_DOCTYPE: RuleId = RuleId::new("xml.forbidden-doctype");

/// Rule id for a reference to an entity that no declaration defines.
pub const RULE_UNDEFINED_ENTITY: RuleId = RuleId::new("xml.undefined-entity");

/// Rule id for the nesting guard.
pub const RULE_DEPTH_LIMIT: RuleId = RuleId::new("xml.depth-limit");

/// Rule id for the text-run guard.
pub const RULE_TEXT_LIMIT: RuleId = RuleId::new("xml.text-limit");

/// The root element and the evidence needed to report it.
///
/// The element borrows the input, so its name is zero-copy; the resolved
/// namespace and `@versione` are copied once because a report may outlive the
/// parse. Namespace resolution is used **only for reporting**, never for
/// matching (`P1 §5.1`, `P3 §8` plumbing item 6).
#[derive(Clone, Debug)]
pub struct RootElement<'a> {
    element: Element<'a>,
    namespace: Option<String>,
    version: Option<String>,
    kind: Option<RootKind>,
}

impl<'a> RootElement<'a> {
    fn new(element: Element<'a>) -> Self {
        let name = element.name();
        let root_prefix = name.prefix();
        let kind = root_kind(name.local());
        let mut namespace = None;
        let mut version = None;

        for attribute in element.attributes() {
            let Ok(attribute) = attribute else {
                // The reader validates every attribute before yielding the
                // element; this arm is unreachable for reader-produced events
                // but keeps the function total for hand-built events.
                continue;
            };
            let key = attribute.name();

            let is_namespace = match key.prefix() {
                Some(prefix) => prefix == b"xmlns" && Some(key.local()) == root_prefix,
                None => key.local() == b"xmlns" && root_prefix.is_none(),
            };
            if is_namespace {
                namespace = core::str::from_utf8(attribute.value())
                    .ok()
                    .map(str::to_owned);
                continue;
            }

            if key.prefix().is_none() && key.local() == b"versione" {
                version = core::str::from_utf8(attribute.value())
                    .ok()
                    .map(str::to_owned);
            }
        }

        Self {
            element,
            namespace,
            version,
            kind,
        }
    }

    /// Returns the qualified root name as written in the input.
    #[must_use]
    pub fn name(&self) -> crate::xml::Name<'_> {
        self.element.name()
    }

    /// Returns the root local name.
    #[must_use]
    pub fn local_name(&self) -> &[u8] {
        self.element.name().local()
    }

    /// Returns the prefix declared on the root element, if any.
    #[must_use]
    pub fn prefix(&self) -> Option<&[u8]> {
        // The element outlives the borrow used by this accessor.
        let name = self.element.name();
        match name.prefix() {
            Some(prefix) => Some(prefix),
            None => None,
        }
    }

    /// Returns the namespace URI bound to the root prefix (or to the default
    /// namespace when the root is unprefixed), if it could be resolved from the
    /// root's own attributes.
    #[must_use]
    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    /// Returns the raw `@versione` value of the root element.
    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// Returns the official root kind, or `None` for an unknown root.
    #[must_use]
    pub const fn kind(&self) -> Option<RootKind> {
        self.kind
    }
}

/// The result of one structural pass.
#[derive(Debug)]
pub struct Scan<'a> {
    /// Root element evidence, when the document had a root.
    pub root: Option<RootElement<'a>>,
    /// L1 findings in discovery order.
    pub findings: Vec<Finding>,
    /// Number of opening and self-closing elements seen.
    pub elements: usize,
    /// Deepest element depth seen.
    pub max_depth: usize,
    /// `true` when the reader reached `Eof` without a fatal error or guard.
    pub completed: bool,
}

impl Scan<'_> {
    /// Returns `true` when the pass reached `Eof` and emitted no error finding.
    ///
    /// A guard warning still makes the scan incomplete when it stopped the
    /// reader; `completed` is the precise signal.
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        self.completed
            && !self
                .findings
                .iter()
                .any(|finding| finding.severity == Severity::Error)
    }
}

/// Runs one structural pass over the bytes.
///
/// The pass never panics and never allocates for element names or text. It
/// allocates only for findings, for the two root strings kept for reporting, and
/// for `quick-xml`'s internal open-tag stack.
#[must_use]
pub fn scan(input: &[u8], limits: Limits) -> Scan<'_> {
    let mut reader = Reader::new(input, limits);
    let mut findings = Vec::new();
    let mut root = None;
    let mut elements = 0usize;
    let mut max_depth = 0usize;
    let mut completed = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(element) | Event::Empty(element)) => {
                let depth = element.depth();
                elements = elements.saturating_add(1);
                max_depth = max_depth.max(depth);

                if depth == 1 {
                    if root.is_some() {
                        findings.push(simple_finding(
                            RULE_WELL_FORMED,
                            Some(SdiCode::EMPTY_OR_CORRUPT_FILE),
                            Severity::Error,
                            format!(
                                "multiple root elements; the second starts at {}",
                                Position::at(input, element.offset())
                            ),
                        ));
                        break;
                    }
                    root = Some(RootElement::new(element));
                }
            }
            Ok(Event::End(_)) => {}
            Ok(Event::Text(text)) => {
                if text.depth() == 0 && !is_xml_whitespace(text.raw()) {
                    findings.push(simple_finding(
                        RULE_WELL_FORMED,
                        Some(SdiCode::EMPTY_OR_CORRUPT_FILE),
                        Severity::Error,
                        format!(
                            "character data outside the root element at {}",
                            Position::at(input, text.offset())
                        ),
                    ));
                    break;
                }
            }
            Ok(Event::EntityRef(reference)) => {
                if let Some(finding) = entity_finding(input, &reference) {
                    findings.push(finding);
                    break;
                }
            }
            Ok(Event::Eof) => {
                completed = true;
                break;
            }
            Err(error) => {
                findings.push(error_finding(&error));
                break;
            }
        }
    }

    if completed && root.is_none() {
        findings.push(simple_finding(
            RULE_WELL_FORMED,
            Some(SdiCode::EMPTY_OR_CORRUPT_FILE),
            Severity::Error,
            "the document contains no root element".to_owned(),
        ));
    }

    Scan {
        root,
        findings,
        elements,
        max_depth,
        completed,
    }
}

fn entity_finding(input: &[u8], reference: &EntityRef<'_>) -> Option<Finding> {
    if reference.is_char_ref() {
        let error = reference.resolve_char_ref().err()?;
        let position = Position::at(input, reference.offset());
        return Some(simple_finding(
            RULE_WELL_FORMED,
            Some(SdiCode::EMPTY_OR_CORRUPT_FILE),
            Severity::Error,
            format!("invalid character reference at {position}: {error}"),
        ));
    }

    if reference.is_predefined() {
        return None;
    }

    let position = Position::at(input, reference.offset());
    let name = String::from_utf8_lossy(reference.raw());
    Some(
        simple_finding(
            RULE_UNDEFINED_ENTITY,
            Some(SdiCode::EMPTY_OR_CORRUPT_FILE),
            Severity::Error,
            format!("undefined entity `&{name};` at {position}"),
        )
        .with_hint("DTDs are not allowed; use one of the five predefined entities"),
    )
}

fn error_finding(error: &ReadError) -> Finding {
    match error {
        ReadError::Xml { .. } | ReadError::Attribute { .. } | ReadError::UnexpectedEof { .. } => {
            simple_finding(
                RULE_WELL_FORMED,
                Some(SdiCode::EMPTY_OR_CORRUPT_FILE),
                Severity::Error,
                error.to_string(),
            )
        }
        ReadError::ForbiddenDoctype { .. } => simple_finding(
            RULE_FORBIDDEN_DOCTYPE,
            Some(SdiCode::EMPTY_OR_CORRUPT_FILE),
            Severity::Error,
            error.to_string(),
        )
        .with_hint("the official invoices contain no DTD; entities are not resolved"),
        ReadError::DepthLimit { .. } => {
            simple_finding(RULE_DEPTH_LIMIT, None, Severity::Warning, error.to_string())
        }
        ReadError::TextLimit { .. } => {
            simple_finding(RULE_TEXT_LIMIT, None, Severity::Warning, error.to_string())
        }
    }
}

fn simple_finding(
    rule: RuleId,
    code: Option<SdiCode>,
    severity: Severity,
    message: String,
) -> Finding {
    Finding::new(
        rule,
        code,
        severity,
        Level::L1,
        ElementPath::root(),
        message,
    )
}

fn is_xml_whitespace(bytes: &[u8]) -> bool {
    bytes
        .iter()
        .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
}

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

    use quick_xml::events::BytesStart;

    use super::{RULE_DEPTH_LIMIT, RULE_TEXT_LIMIT, RootElement, is_xml_whitespace, scan};
    use crate::diag::{Level, SdiCode, Severity};
    use crate::xml::event::Element;
    use crate::xml::reader::Limits;
    use crate::xml::{Name, RootKind};

    #[test]
    fn whitespace_predicate_matches_only_xml_whitespace() {
        assert!(is_xml_whitespace(b" \t\r\n"));
        assert!(is_xml_whitespace(b""));
        assert!(!is_xml_whitespace(b" x "));
        assert!(!is_xml_whitespace(b"\0"));
    }

    #[test]
    fn scan_records_root_prefix_namespace_version_and_counts() {
        let input = br#"<?xml version="1.0"?>
<!-- comment -->
<p:FatturaElettronica versione="FPR12"
   xmlns:p="http://ivaservizi.agenziaentrate.gov.it/docs/xsd/fatture/v1.2"
   xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
   xsi:schemaLocation="urn:schema Schema_VFPR12_v1.2.3.xsd">
  <FatturaElettronicaHeader/>
  <FatturaElettronicaBody/>
</p:FatturaElettronica>"#;
        let result = scan(input, Limits::default());
        assert!(result.completed);
        assert!(result.is_well_formed());
        assert!(result.findings.is_empty());
        assert_eq!(result.elements, 3);
        assert_eq!(result.max_depth, 2);

        let root = result.root.expect("root");
        assert_eq!(root.local_name(), b"FatturaElettronica");
        assert_eq!(root.prefix(), Some(b"p".as_slice()));
        assert_eq!(root.version(), Some("FPR12"));
        assert_eq!(
            root.namespace(),
            Some("http://ivaservizi.agenziaentrate.gov.it/docs/xsd/fatture/v1.2")
        );
        assert_eq!(root.kind(), Some(RootKind::Invoice));
        assert!(format!("{root:?}").contains("FatturaElettronica"));
    }

    #[test]
    fn scan_resolves_default_namespace_for_unprefixed_root() {
        let input = br#"<FatturaElettronicaSemplificata versione="FSM10" xmlns="http://ivaservizi.agenziaentrate.gov.it/docs/xsd/fatture/v1.0"/>"#;
        let result = scan(input, Limits::default());
        let root = result.root.expect("root");
        assert_eq!(root.prefix(), None);
        assert_eq!(
            root.namespace(),
            Some("http://ivaservizi.agenziaentrate.gov.it/docs/xsd/fatture/v1.0")
        );
        assert_eq!(root.kind(), Some(RootKind::Simplified));
        assert_eq!(root.version(), Some("FSM10"));
    }

    #[test]
    fn scan_records_missing_namespace_version_and_unknown_root() {
        let result = scan(b"<Unknown/>", Limits::default());
        let root = result.root.expect("root");
        assert_eq!(root.namespace(), None);
        assert_eq!(root.version(), None);
        assert_eq!(root.kind(), None);
        assert_eq!(root.prefix(), None);
        assert!(result.findings.is_empty());
    }

    #[test]
    fn scan_accepts_whitespace_and_comments_outside_root() {
        let input = b"\n<!-- before -->\n<r/>\n<!-- after -->\n";
        let result = scan(input, Limits::default());
        assert!(result.completed);
        assert!(result.findings.is_empty());
        assert!(result.root.is_some());
    }

    #[test]
    fn scan_flags_empty_documents_and_whitespace_only_input() {
        for input in [
            b"".as_slice(),
            b" \n\t".as_slice(),
            b"<!-- only -->".as_slice(),
        ] {
            let result = scan(input, Limits::default());
            assert!(result.completed);
            assert!(!result.is_well_formed());
            assert!(result.root.is_none());
            assert_eq!(result.findings.len(), 1, "{input:?}");
            let finding = &result.findings[0];
            assert_eq!(finding.code, Some(SdiCode::EMPTY_OR_CORRUPT_FILE));
            assert_eq!(finding.severity, Severity::Error);
            assert_eq!(finding.level, Level::L1);
            assert!(finding.message.contains("no root element"));
        }
    }

    #[test]
    fn scan_flags_multiple_roots_and_text_outside_root() {
        let multiple = scan(b"<a/><b/>", Limits::default());
        assert!(!multiple.completed);
        assert_eq!(multiple.findings.len(), 1);
        assert!(multiple.findings[0].message.contains("multiple root"));

        let text = scan(b"oops<r/>", Limits::default());
        assert!(!text.completed);
        assert_eq!(text.findings.len(), 1);
        assert!(text.findings[0].message.contains("outside the root"));

        let nested = scan(b"<r>text</r>", Limits::default());
        assert!(nested.completed);
        assert!(nested.findings.is_empty());
    }

    #[test]
    fn scan_rejects_doctype_and_undefined_entities() {
        let doctype = scan(b"<!DOCTYPE r><r/>", Limits::default());
        assert!(!doctype.completed);
        assert_eq!(doctype.findings.len(), 1);
        assert_eq!(doctype.findings[0].rule.as_str(), "xml.forbidden-doctype");
        assert!(doctype.findings[0].hint.is_some());

        let undefined = scan(b"<r>&xxe;</r>", Limits::default());
        assert!(!undefined.completed);
        assert_eq!(undefined.findings.len(), 1);
        assert_eq!(undefined.findings[0].rule.as_str(), "xml.undefined-entity");
        assert!(undefined.findings[0].hint.is_some());

        let predefined = scan(b"<r>&amp;&#65;&#x42;</r>", Limits::default());
        assert!(predefined.completed);
        assert!(predefined.findings.is_empty());

        let invalid_char_ref = scan(b"<r>&#0;</r>", Limits::default());
        assert!(!invalid_char_ref.completed);
        assert_eq!(invalid_char_ref.findings.len(), 1);
        assert!(
            invalid_char_ref.findings[0]
                .message
                .contains("invalid character reference")
        );
    }

    #[test]
    fn scan_maps_syntax_errors_to_l1_findings_with_position() {
        let result = scan(b"<r>\n  <a>\n</r>", Limits::default());
        assert!(!result.completed);
        assert_eq!(result.findings.len(), 1);
        let finding = &result.findings[0];
        assert_eq!(finding.rule.as_str(), "xml.well-formedness");
        assert_eq!(finding.code, Some(SdiCode::EMPTY_OR_CORRUPT_FILE));
        assert_eq!(finding.level, Level::L1);
        assert_eq!(finding.severity, Severity::Error);
        assert!(finding.message.contains("line "));
        assert!(finding.message.contains("column "));
    }

    #[test]
    fn scan_depth_limit_is_a_warning_without_an_sdi_code() {
        let input = b"<a><b><c/></b></a>";
        let result = scan(input, Limits::new(2, 1_024));
        assert!(!result.completed);
        assert_eq!(result.findings.len(), 1);
        let finding = &result.findings[0];
        assert_eq!(finding.rule, RULE_DEPTH_LIMIT);
        assert_eq!(finding.code, None);
        assert_eq!(finding.severity, Severity::Warning);
        assert_eq!(finding.level, Level::L1);

        let exact = scan(input, Limits::new(3, 1_024));
        assert!(exact.completed);
        assert!(exact.findings.is_empty());
    }

    #[test]
    fn scan_text_limit_is_a_warning_without_an_sdi_code() {
        let result = scan(b"<r>abcd</r>", Limits::new(64, 3));
        assert!(!result.completed);
        assert_eq!(result.findings.len(), 1);
        let finding = &result.findings[0];
        assert_eq!(finding.rule, RULE_TEXT_LIMIT);
        assert_eq!(finding.code, None);
        assert_eq!(finding.severity, Severity::Warning);

        let exact = scan(b"<r>abcd</r>", Limits::new(64, 4));
        assert!(exact.completed);
        assert!(exact.findings.is_empty());
    }

    #[test]
    fn scan_reports_attribute_and_comment_failures() {
        let attribute = scan(br#"<r a="1" a="2"/>"#, Limits::default());
        assert!(!attribute.completed);
        assert_eq!(attribute.findings.len(), 1);
        assert!(attribute.findings[0].message.contains("attribute"));

        let comment = scan(b"<r><!-- broken </r>", Limits::default());
        assert!(!comment.completed);
        assert_eq!(comment.findings.len(), 1);
    }

    #[test]
    fn scan_handles_a_root_with_no_children_and_hand_built_oddities() {
        let result = scan(b"<r></r>", Limits::default());
        assert!(result.completed);
        assert_eq!(result.elements, 1);
        assert_eq!(result.max_depth, 1);
        assert!(result.findings.is_empty());

        let empty_root = scan(b"<r/>", Limits::default());
        assert!(empty_root.completed);
        assert_eq!(empty_root.elements, 1);
    }

    #[test]
    fn root_construction_ignores_malformed_hand_built_attributes() {
        // The reader rejects malformed attributes before yielding an element;
        // this hand-built element covers the total fallback in `RootElement::new`.
        let start = BytesStart::from_content("r a=bad", 1);
        let element = Element::new(start, Name::new(b"r"), 1, 0);
        let root = RootElement::new(element);
        assert_eq!(root.namespace(), None);
        assert_eq!(root.version(), None);
    }

    #[test]
    fn scan_helpers_cover_root_attribute_edge_cases() {
        // Namespace attributes bound to a different prefix, a prefixed
        // `versione`, and a non-UTF-8 namespace value are ignored or left
        // unresolved rather than panicking.
        let input = br#"<p:r xmlns:q="urn:q" versione="FPR12" xmlns:p="urn:p"/>"#;
        let result = scan(input, Limits::default());
        let root = result.root.expect("root");
        assert_eq!(root.namespace(), Some("urn:p"));
        assert_eq!(root.version(), Some("FPR12"));

        let no_default = scan(br#"<r xmlns:q="urn:q"/>"#, Limits::default());
        assert_eq!(no_default.root.expect("root").namespace(), None);

        let prefixed_version = scan(
            br#"<p:r xmlns:p="urn:p" p:versione="FPR12"/>"#,
            Limits::default(),
        );
        assert_eq!(
            prefixed_version.root.expect("root").version(),
            None,
            "the schema's root attributes are unqualified"
        );

        let mut invalid = b"<r xmlns=\"".to_vec();
        invalid.push(0xff);
        invalid.extend_from_slice(b"\"/>");
        let result = scan(&invalid, Limits::default());
        assert_eq!(result.root.expect("root").namespace(), None);
    }
}
