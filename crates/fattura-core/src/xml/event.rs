//! Borrowed reader events.
//!
//! Every event carries the byte offset of its opening `<` and the element depth
//! at which it occurred. Names and values borrow from the input slice; no
//! `String` is created while reading a healthy document (`PLAN §2.4`).

use core::fmt;
use std::borrow::Cow;

use quick_xml::events::attributes::AttrError;
use quick_xml::events::{BytesRef, BytesStart};

use super::name::Name;

/// An element start tag or self-closing tag.
#[derive(Clone, Debug)]
pub struct Element<'a> {
    start: BytesStart<'a>,
    name: Name<'a>,
    depth: usize,
    offset: u64,
}

impl<'a> Element<'a> {
    pub(super) fn new(start: BytesStart<'a>, name: Name<'a>, depth: usize, offset: u64) -> Self {
        Self {
            start,
            name,
            depth,
            offset,
        }
    }

    /// Returns the qualified name as written in the input.
    #[must_use]
    pub const fn name(&self) -> Name<'a> {
        self.name
    }

    /// Returns the 1-based element depth. The document root has depth `1`.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }

    /// Returns the byte offset of the opening `<`.
    #[must_use]
    pub const fn offset(&self) -> u64 {
        self.offset
    }

    /// Iterates over the attributes of the tag.
    ///
    /// Values are raw: XML entity references are **not** resolved, so no
    /// allocation happens. Callers that need a decoded value use
    /// [`Attribute::value`] and decode it at their boundary.
    #[must_use]
    pub fn attributes(&self) -> Attributes<'_> {
        Attributes {
            inner: self.start.attributes(),
        }
    }
}

/// An element end tag.
#[derive(Clone, Debug)]
pub struct EndTag<'a> {
    name: Name<'a>,
    depth: usize,
    offset: u64,
}

impl<'a> EndTag<'a> {
    pub(super) const fn new(name: Name<'a>, depth: usize, offset: u64) -> Self {
        Self {
            name,
            depth,
            offset,
        }
    }

    /// Returns the qualified name as written in the input.
    #[must_use]
    pub const fn name(&self) -> Name<'a> {
        self.name
    }

    /// Returns the 1-based depth of the element being closed.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }

    /// Returns the byte offset of the opening `<` of the end tag.
    #[must_use]
    pub const fn offset(&self) -> u64 {
        self.offset
    }
}

/// Character data, either ordinary text or a CDATA section.
#[derive(Clone, Debug)]
pub struct Text<'a> {
    inner: Cow<'a, [u8]>,
    cdata: bool,
    depth: usize,
    offset: u64,
}

impl<'a> Text<'a> {
    pub(super) fn new(inner: Cow<'a, [u8]>, cdata: bool, depth: usize, offset: u64) -> Self {
        Self {
            inner,
            cdata,
            depth,
            offset,
        }
    }

    /// Returns the raw bytes exactly as they appear in the input.
    ///
    /// XML escapes and character references are not resolved; they arrive as
    /// separate [`Event::EntityRef`] events. Whitespace is never trimmed, because
    /// `Descrizione` and `Causale` are length-checked and spaces are significant
    /// (`P3 §8` plumbing item 5).
    #[must_use]
    pub fn raw(&self) -> &[u8] {
        &self.inner
    }

    /// Returns `true` when the data came from a CDATA section.
    #[must_use]
    pub const fn is_cdata(&self) -> bool {
        self.cdata
    }

    /// Returns the element depth containing the text. Text outside the root has
    /// depth `0`.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }

    /// Returns the byte offset of the first text byte.
    #[must_use]
    pub const fn offset(&self) -> u64 {
        self.offset
    }
}

/// One entity or character reference, such as `&amp;` or `&#233;`.
#[derive(Clone, Debug)]
pub struct EntityRef<'a> {
    inner: BytesRef<'a>,
    depth: usize,
    offset: u64,
}

impl<'a> EntityRef<'a> {
    pub(super) fn new(inner: BytesRef<'a>, depth: usize, offset: u64) -> Self {
        Self {
            inner,
            depth,
            offset,
        }
    }

    /// Returns the reference body without `&` and `;`.
    #[must_use]
    pub fn raw(&self) -> &[u8] {
        &self.inner
    }

    /// Returns the element depth containing the reference.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }

    /// Returns the byte offset of the `&`.
    #[must_use]
    pub const fn offset(&self) -> u64 {
        self.offset
    }

    /// Returns `true` for a character reference (`&#...;`).
    #[must_use]
    pub fn is_char_ref(&self) -> bool {
        self.inner.is_char_ref()
    }

    /// Returns `true` for one of the five XML predefined entities.
    ///
    /// Any other entity is undefined unless a `DOCTYPE` declares it — and DTDs
    /// are rejected before this point (`P3 §8` plumbing items 1–6).
    #[must_use]
    pub fn is_predefined(&self) -> bool {
        matches!(self.raw(), b"amp" | b"lt" | b"gt" | b"apos" | b"quot")
    }

    /// Resolves a character reference to its character.
    ///
    /// Returns `Ok(None)` for a named entity. An invalid numeric reference is an
    /// error; the message is allocated only on the error path.
    pub fn resolve_char_ref(&self) -> Result<Option<char>, EntityError> {
        match self.inner.resolve_char_ref() {
            Ok(value) => Ok(value),
            Err(error) => Err(EntityError {
                message: error.to_string(),
            }),
        }
    }
}

/// A malformed entity reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityError {
    message: String,
}

impl fmt::Display for EntityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for EntityError {}

/// One attribute of a start tag.
#[derive(Clone, Debug)]
pub struct Attribute<'a> {
    name: Name<'a>,
    value: Cow<'a, [u8]>,
}

impl<'a> Attribute<'a> {
    /// Returns the qualified attribute name.
    #[must_use]
    pub const fn name(&self) -> Name<'a> {
        self.name
    }

    /// Returns the raw attribute value.
    ///
    /// The value is exactly the bytes between the quotes; entity references are
    /// not resolved here (`P3 §8` plumbing item 4).
    #[must_use]
    pub fn value(&self) -> &[u8] {
        &self.value
    }
}

/// A malformed attribute.
///
/// The underlying parser type is kept private so the XML dependency does not
/// leak into the ingestion API.
#[derive(Clone, Debug)]
pub struct AttributeError {
    source: AttrError,
}

impl AttributeError {
    pub(super) fn new(source: AttrError) -> Self {
        Self { source }
    }
}

impl fmt::Display for AttributeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.source, formatter)
    }
}

impl std::error::Error for AttributeError {}

/// Iterator over the attributes of an element.
#[derive(Clone)]
pub struct Attributes<'a> {
    inner: quick_xml::events::attributes::Attributes<'a>,
}

impl fmt::Debug for Attributes<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Attributes(..)")
    }
}

impl<'a> Iterator for Attributes<'a> {
    type Item = Result<Attribute<'a>, AttributeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|result| {
            result
                .map(|attribute| Attribute {
                    name: Name::new(attribute.key.into_inner()),
                    value: attribute.value,
                })
                .map_err(AttributeError::new)
        })
    }
}

/// An event produced by the reader.
///
/// Comments, processing instructions and the XML declaration are consumed by the
/// reader and never surface here (`P3 §8` plumbing item 3). Self-closing tags are
/// reported as [`Event::Empty`] so the reader stays allocation-free.
#[derive(Clone, Debug)]
pub enum Event<'a> {
    /// An opening tag.
    Start(Element<'a>),
    /// A self-closing tag.
    Empty(Element<'a>),
    /// A closing tag.
    End(EndTag<'a>),
    /// Character data or a CDATA section.
    Text(Text<'a>),
    /// An entity or character reference.
    EntityRef(EntityRef<'a>),
    /// End of input.
    Eof,
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

    use quick_xml::events::BytesRef;
    use quick_xml::events::BytesStart;

    use super::{Attributes, Element, EntityRef, Event, Text};
    use crate::xml::Name;

    #[test]
    fn malformed_attribute_exposes_a_message_and_debug() {
        let start = BytesStart::from_content("r a=bad", 1);
        let element = Element::new(start, Name::new(b"r"), 1, 0);
        let mut attributes = element.attributes();

        let error = attributes.next().unwrap().unwrap_err();
        assert!(!error.to_string().is_empty());
        assert!(std::error::Error::source(&error).is_none());
        assert!(format!("{:?}", element.attributes()).contains("Attributes(..)"));
        assert!(attributes.next().is_none());

        // Exercise the hand-built `Attributes` debug formatter directly; the
        // reader can only produce elements whose attributes are valid.
        let start = BytesStart::from_content("r", 1);
        let element = Element::new(start, Name::new(b"r"), 1, 0);
        let attributes: Attributes<'_> = element.attributes();
        assert!(format!("{attributes:?}").contains("Attributes(..)"));
    }

    #[test]
    fn all_predefined_entities_are_recognized() {
        for name in ["amp", "lt", "gt", "apos", "quot"] {
            let reference = EntityRef::new(BytesRef::new(name), 1, 0);
            assert!(reference.is_predefined(), "{name:?}");
            assert!(!reference.is_char_ref());
        }
        let numeric = EntityRef::new(BytesRef::new("#65"), 1, 0);
        assert!(!numeric.is_predefined());
        assert!(numeric.is_char_ref());
        assert_eq!(numeric.resolve_char_ref().unwrap(), Some('A'));
    }

    #[test]
    fn text_and_event_accessors_are_total() {
        let text = Text::new(std::borrow::Cow::Borrowed(b"abc"), true, 2, 7);
        assert_eq!(text.raw(), b"abc");
        assert!(text.is_cdata());
        assert_eq!(text.depth(), 2);
        assert_eq!(text.offset(), 7);

        let event = Event::Text(text);
        assert!(format!("{event:?}").contains("Text"));
    }
}
