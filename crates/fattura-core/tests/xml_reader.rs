//! Unit-level tests for the streaming XML reader.
//!
//! Moved out of `src/xml/reader.rs` so coverage instrumentation reports only the
//! production paths. The same boundaries are also exercised end-to-end by
//! `tests/xml_scan.rs`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::assert_is_empty
)]

use fattura_core::xml::{Event, Limits, ReadError, Reader};

#[test]
fn limits_defaults_and_accessors() {
    let limits = Limits::default();
    assert_eq!(limits.max_depth, 64);
    assert_eq!(limits.max_text_run, 1_048_576);
    assert_eq!(
        Limits::new(1, 2),
        Limits {
            max_depth: 1,
            max_text_run: 2
        }
    );

    let mut reader = Reader::new(b"<r>", limits);
    assert_eq!(reader.limits(), limits);
    assert_eq!(reader.depth(), 0);
    assert_eq!(reader.position(), 0);
    assert!(matches!(reader.read_event().unwrap(), Event::Start(_)));
    assert_eq!(
        reader.position(),
        3,
        "the cursor advances past the start tag"
    );
}

#[test]
fn reads_events_with_depth_offsets_and_borrowed_names() {
    let input = br#"<p:a xmlns:p="urn:x"><b>text</b><c/></p:a>"#;
    let mut reader = Reader::new(input, Limits::default());

    let Event::Start(root) = reader.read_event().unwrap() else {
        panic!("expected start");
    };
    assert_eq!(root.name().prefix(), Some(b"p".as_slice()));
    assert_eq!(root.name().local(), b"a");
    assert_eq!(root.depth(), 1);
    assert_eq!(root.offset(), 0);
    assert_eq!(reader.depth(), 1);

    let Event::Start(child) = reader.read_event().unwrap() else {
        panic!("expected child start");
    };
    assert_eq!(child.name().local(), b"b");
    assert_eq!(child.depth(), 2);
    assert_eq!(child.offset(), 21);
    assert_eq!(reader.depth(), 2);

    let Event::Text(text) = reader.read_event().unwrap() else {
        panic!("expected text");
    };
    assert_eq!(text.raw(), b"text");
    assert!(!text.is_cdata());
    assert_eq!(text.depth(), 2);
    assert_eq!(text.offset(), 24);

    let Event::End(end) = reader.read_event().unwrap() else {
        panic!("expected end");
    };
    assert_eq!(end.name().local(), b"b");
    assert_eq!(end.depth(), 2);
    assert_eq!(reader.depth(), 1);
    assert_eq!(end.offset(), 28);

    let Event::Empty(empty) = reader.read_event().unwrap() else {
        panic!("expected empty");
    };
    assert_eq!(empty.name().local(), b"c");
    assert_eq!(empty.depth(), 2);
    assert_eq!(reader.depth(), 1, "empty elements do not change depth");

    let Event::End(root_end) = reader.read_event().unwrap() else {
        panic!("expected root end");
    };
    assert_eq!(root_end.depth(), 1);
    assert_eq!(reader.depth(), 0);
    assert!(matches!(reader.read_event().unwrap(), Event::Eof));
    assert!(matches!(reader.read_event().unwrap(), Event::Eof));
}

#[test]
fn skips_declaration_comments_and_processing_instructions() {
    let input =
        br#"<?xml version="1.0"?><?xml-stylesheet href="x"?><!-- a --><r><!-- b --><?pi?><x/></r>"#;
    let mut reader = Reader::new(input, Limits::default());

    let Event::Start(root) = reader.read_event().unwrap() else {
        panic!("expected root");
    };
    assert_eq!(root.name().local(), b"r");
    let Event::Empty(child) = reader.read_event().unwrap() else {
        panic!("expected child");
    };
    assert_eq!(child.name().local(), b"x");
    assert!(matches!(reader.read_event().unwrap(), Event::End(_)));
    assert!(matches!(reader.read_event().unwrap(), Event::Eof));
}

#[test]
fn preserves_whitespace_in_text_and_cdata() {
    let input = b"<r>  a \n b  <![CDATA[c  d]]></r>";
    let mut reader = Reader::new(input, Limits::default());
    assert!(matches!(reader.read_event().unwrap(), Event::Start(_)));

    let Event::Text(first) = reader.read_event().unwrap() else {
        panic!("expected text");
    };
    assert_eq!(first.raw(), b"  a \n b  ");
    assert!(!first.is_cdata());

    let Event::Text(second) = reader.read_event().unwrap() else {
        panic!("expected cdata");
    };
    assert_eq!(second.raw(), b"c  d");
    assert!(second.is_cdata());
    assert!(matches!(reader.read_event().unwrap(), Event::End(_)));
}

#[test]
fn rejects_doctype_before_entities_can_be_declared() {
    let input = br#"<!DOCTYPE r [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><r>&xxe;</r>"#;
    let mut reader = Reader::new(input, Limits::default());
    let error = reader.read_event().unwrap_err();
    assert!(matches!(error, ReadError::ForbiddenDoctype { .. }));
    assert_eq!(error.position().byte, 0);
}

#[test]
fn rejects_malformed_markup_with_a_position() {
    let cases: &[(&[u8], u64)] = &[
        (b"<r><a></r>", 9),
        (b"<r><a>", 6),
        (b"<r><!-- unclosed", 12),
        (b"<r><!-- a -- b --></r>", 3),
        (b"<r><?pi unclosed</r>", 3),
        (b"<?xml version=\"1.0\"", 0),
        (b"<r attr=unquoted/>", 0),
        (b"<r attr='unclosed></r>", 0),
        (b"<r><![CDATA[unclosed</r>", 3),
        (b"<r><a></b></r>", 6),
        (b"<r></r></r>", 6),
    ];
    for (input, minimum) in cases {
        let mut reader = Reader::new(input, Limits::default());
        let mut error = None;
        for _ in 0..10 {
            match reader.read_event() {
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(value) => {
                    error = Some(value);
                    break;
                }
            }
        }
        let error = error.unwrap_or_else(|| panic!("expected error for {input:?}"));
        assert!(error.position().byte >= *minimum, "{input:?}: {error:?}");
        assert!(error.position().line >= 1);
        assert!(error.position().column >= 1);
    }
}

#[test]
fn depth_limit_boundaries() {
    let input = b"<a><b><c/></b></a>";
    for (limit, succeeds) in [(2usize, false), (3, true), (4, true)] {
        let mut reader = Reader::new(input, Limits::new(limit, 1_024));
        let mut error = None;
        loop {
            match reader.read_event() {
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(value) => {
                    error = Some(value);
                    break;
                }
            }
        }
        assert_eq!(error.is_none(), succeeds, "limit {limit}: {error:?}");
        if let Some(error) = error {
            assert!(matches!(error, ReadError::DepthLimit { limit: 2, .. }));
        }
    }

    // A Start element exactly at the limit must be accepted; only depth `limit + 1`
    // is a breach. This distinguishes `>` from `>=` at the Start branch.
    for (input, limit, succeeds) in [
        (b"<a><b></b></a>".as_slice(), 2usize, true),
        (b"<a><b></b></a>".as_slice(), 1, false),
    ] {
        let mut reader = Reader::new(input, Limits::new(limit, 1_024));
        let mut error = None;
        loop {
            match reader.read_event() {
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(value) => {
                    error = Some(value);
                    break;
                }
            }
        }
        assert_eq!(error.is_none(), succeeds, "start limit {limit}: {error:?}");
    }
}

#[test]
fn text_limit_boundaries_count_cdata_and_entities() {
    for (limit, succeeds) in [(3usize, false), (4, true), (5, true)] {
        let input = b"<r>abcd</r>";
        let mut reader = Reader::new(input, Limits::new(64, limit));
        assert!(matches!(reader.read_event().unwrap(), Event::Start(_)));
        match reader.read_event() {
            Ok(Event::Text(text)) => assert_eq!(text.raw(), b"abcd"),
            Err(error) => {
                assert!(!succeeds);
                assert!(matches!(error, ReadError::TextLimit { limit: 3, .. }));
            }
            Ok(other) => panic!("unexpected event {other:?}"),
        }
    }

    let mut reader = Reader::new(b"<r>&amp;</r>", Limits::new(64, 4));
    assert!(matches!(reader.read_event().unwrap(), Event::Start(_)));
    let error = reader.read_event().unwrap_err();
    assert!(matches!(error, ReadError::TextLimit { limit: 4, .. }));

    let mut reader = Reader::new(b"<r><![CDATA[abcd]]></r>", Limits::new(64, 3));
    assert!(matches!(reader.read_event().unwrap(), Event::Start(_)));
    let error = reader.read_event().unwrap_err();
    assert!(matches!(error, ReadError::TextLimit { limit: 3, .. }));
}

#[test]
fn entity_references_are_exposed_and_resolved() {
    let mut nested_reader = Reader::new(b"<a><b>&amp;</b></a>", Limits::default());
    assert!(matches!(
        nested_reader.read_event().unwrap(),
        Event::Start(_)
    ));
    assert!(matches!(
        nested_reader.read_event().unwrap(),
        Event::Start(_)
    ));
    let Event::EntityRef(nested) = nested_reader.read_event().unwrap() else {
        panic!("expected nested amp");
    };
    assert_eq!(nested.depth(), 2);

    let mut reader = Reader::new(b"<r>&amp;&#65;&#x42;&bad;</r>", Limits::default());
    assert!(matches!(reader.read_event().unwrap(), Event::Start(_)));

    let Event::EntityRef(amp) = reader.read_event().unwrap() else {
        panic!("expected amp");
    };
    assert_eq!(amp.raw(), b"amp");
    assert!(amp.is_predefined());
    assert!(!amp.is_char_ref());
    assert_eq!(amp.resolve_char_ref().unwrap(), None);
    assert_eq!(amp.depth(), 1);
    assert_eq!(amp.offset(), 3);

    let Event::EntityRef(dec) = reader.read_event().unwrap() else {
        panic!("expected decimal ref");
    };
    assert!(dec.is_char_ref());
    assert_eq!(dec.resolve_char_ref().unwrap(), Some('A'));

    let Event::EntityRef(hex) = reader.read_event().unwrap() else {
        panic!("expected hex ref");
    };
    assert_eq!(hex.resolve_char_ref().unwrap(), Some('B'));

    let Event::EntityRef(named) = reader.read_event().unwrap() else {
        panic!("expected named ref");
    };
    assert_eq!(named.raw(), b"bad");
    assert!(!named.is_predefined());
    assert!(!named.is_char_ref());
}

#[test]
fn invalid_character_reference_is_an_error() {
    let mut reader = Reader::new(b"<r>&#0;</r>", Limits::default());
    assert!(matches!(reader.read_event().unwrap(), Event::Start(_)));
    let Event::EntityRef(reference) = reader.read_event().unwrap() else {
        panic!("expected reference");
    };
    let error = reference.resolve_char_ref().unwrap_err();
    assert!(!error.to_string().is_empty());
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn attribute_iterator_is_borrowed_and_reports_values() {
    let mut reader = Reader::new(
        br#"<p:r xmlns:p="urn:x" versione="FPR12" xsi:schemaLocation="a b"/>"#,
        Limits::default(),
    );
    let Event::Empty(root) = reader.read_event().unwrap() else {
        panic!("expected empty root");
    };
    let mut names = Vec::new();
    for attribute in root.attributes() {
        let attribute = attribute.unwrap();
        names.push(attribute.name().raw().to_vec());
        assert!(!attribute.value().is_empty());
    }
    assert_eq!(
        names,
        vec![
            b"xmlns:p".to_vec(),
            b"versione".to_vec(),
            b"xsi:schemaLocation".to_vec()
        ]
    );
}

#[test]
fn duplicate_attributes_are_rejected_by_the_reader() {
    let mut reader = Reader::new(br#"<r a="1" a="2"/>"#, Limits::default());
    let error = reader.read_event().unwrap_err();
    assert!(matches!(error, ReadError::Attribute { .. }));
}
