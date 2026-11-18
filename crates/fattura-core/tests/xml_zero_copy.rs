//! Zero-copy proof for element names, attribute names/values and text.
//!
//! Any `String` or `Vec<u8>` copy of a name would move its pointer outside the
//! caller's input buffer. Checking that every borrowed slice points **inside**
//! the input is therefore a direct, allocation-agnostic proof of zero-copy
//! behaviour (`PLAN §2.4`, issue #16 acceptance criterion).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::assert_is_empty
)]

use fattura_core::xml::{Event, Limits, Name, Reader, scan};

const FPR12: &str = include_str!("data/xml/fpr12_mixed_prefix.xml");

fn assert_borrowed(input: &str, bytes: &[u8]) {
    let range = input.as_bytes().as_ptr_range();
    assert!(
        range.contains(&bytes.as_ptr()),
        "slice starts outside the input buffer"
    );
}

fn assert_name_borrowed(input: &str, name: Name<'_>) {
    assert_borrowed(input, name.raw());
    assert_borrowed(input, name.local());
    if let Some(prefix) = name.prefix() {
        assert_borrowed(input, prefix);
    }
}

#[test]
fn element_names_attributes_and_text_borrow_from_the_input() {
    let mut reader = Reader::new(FPR12.as_bytes(), Limits::default());
    let mut elements = 0usize;
    let mut attributes = 0usize;
    let mut texts = 0usize;

    loop {
        match reader.read_event().unwrap() {
            Event::Start(element) | Event::Empty(element) => {
                elements = elements.saturating_add(1);
                assert_name_borrowed(FPR12, element.name());
                for attribute in element.attributes() {
                    let attribute = attribute.unwrap();
                    attributes = attributes.saturating_add(1);
                    assert_name_borrowed(FPR12, attribute.name());
                    assert_borrowed(FPR12, attribute.value());
                }
            }
            Event::End(element) => {
                assert_name_borrowed(FPR12, element.name());
            }
            Event::Text(text) => {
                texts = texts.saturating_add(1);
                assert_borrowed(FPR12, text.raw());
            }
            Event::EntityRef(reference) => {
                assert_borrowed(FPR12, reference.raw());
            }
            Event::Eof => break,
        }
    }

    assert!(elements > 40, "fixture should exercise many names");
    assert!(attributes > 5, "fixture should exercise attributes");
    assert!(texts > 40, "fixture should exercise text");
}

#[test]
fn names_are_stable_across_reader_and_scan_paths() {
    let result = scan(FPR12.as_bytes(), Limits::default());
    let root = result.root.expect("root");
    assert_name_borrowed(FPR12, root.name());
    assert!(result.findings.is_empty());
}
