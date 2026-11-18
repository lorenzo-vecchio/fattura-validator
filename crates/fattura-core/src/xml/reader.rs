//! Streaming XML reader over a borrowed byte slice.
//!
//! The reader is a thin, panic-free adapter over `quick-xml` (`ADR 0001`): it
//! yields borrowed names and values, tracks element depth and byte offsets, and
//! turns the parser's failure modes into [`ReadError`] values. It performs no
//! namespace resolution: the official schema leaves children unqualified, so
//! dispatch is on local names (`P1 §5.1`, `P3 §8` plumbing item 6).
//!
//! Comments and processing instructions are skipped; a `DOCTYPE` is rejected
//! immediately so entity declarations are never parsed (`P3 §8` plumbing items
//! 1–6). Depth and text-run limits are resource guards, not business rules, and
//! they surface as explicit errors instead of aborts.

use std::borrow::Cow;

use quick_xml::events::Event as QuickEvent;
use quick_xml::reader::Reader as QuickReader;

use super::error::{Position, ReadError};
use super::event::{Element, EndTag, EntityRef, Event, Text};
use super::name::Name;

/// Resource limits applied while reading.
///
/// The values are deliberately far above any document the official schema can
/// describe; they exist to stop hostile nesting and pathological text before a
/// consumer can build unbounded state. A limit breach is reported as an L1
/// finding, never as a panic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Maximum element nesting depth. The root element has depth `1`.
    pub max_depth: usize,
    /// Maximum length in bytes of one contiguous text run.
    pub max_text_run: usize,
}

impl Limits {
    /// Creates explicit limits.
    #[must_use]
    pub const fn new(max_depth: usize, max_text_run: usize) -> Self {
        Self {
            max_depth,
            max_text_run,
        }
    }
}

impl Default for Limits {
    /// Returns the production limits: 64 levels and 1 MiB of text.
    fn default() -> Self {
        Self::new(64, 1_048_576)
    }
}

/// A streaming reader over `&[u8]`.
///
/// ```
/// use fattura_core::xml::{Event, Limits, Reader};
///
/// let mut reader = Reader::new(b"<p:root xmlns:p=\"urn:test\"/>", Limits::default());
/// let Event::Empty(root) = reader.read_event().unwrap() else {
///     panic!("expected an empty root");
/// };
/// assert_eq!(root.name().local(), b"root");
/// assert_eq!(root.name().prefix(), Some(b"p".as_slice()));
/// assert_eq!(root.depth(), 1);
/// ```
#[derive(Debug)]
pub struct Reader<'a> {
    inner: QuickReader<&'a [u8]>,
    input: &'a [u8],
    limits: Limits,
    depth: usize,
    text_run: usize,
}

impl<'a> Reader<'a> {
    /// Creates a reader over the full input slice.
    #[must_use]
    pub fn new(input: &'a [u8], limits: Limits) -> Self {
        let mut inner = QuickReader::from_reader(input);
        // Comments are ignored, but a malformed comment is still malformed XML.
        inner.config_mut().check_comments = true;
        // Whitespace must be preserved: `Descrizione` and `Causale` are
        // length-checked and spaces are significant (`P3 §8` plumbing item 5).
        inner.config_mut().trim_text(false);
        Self {
            inner,
            input,
            limits,
            depth: 0,
            text_run: 0,
        }
    }

    /// Returns the configured limits.
    #[must_use]
    pub const fn limits(&self) -> Limits {
        self.limits
    }

    /// Returns the current element depth; `0` means outside the root.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }

    /// Returns the parser cursor as a zero-based byte offset.
    #[must_use]
    pub const fn position(&self) -> u64 {
        self.inner.buffer_position()
    }

    /// Reads the next event.
    ///
    /// Comments, processing instructions and the XML declaration are consumed
    /// internally. A `DOCTYPE` declaration is an error, as is any malformed
    /// markup. The returned event borrows from the input, so reading a healthy
    /// document creates no `String`.
    pub fn read_event(&mut self) -> Result<Event<'a>, ReadError> {
        loop {
            let offset = self.inner.buffer_position();
            match self.inner.read_event() {
                Err(source) => {
                    let position = Position::at(self.input, self.inner.buffer_position());
                    return Err(ReadError::Xml { position, source });
                }
                Ok(QuickEvent::Start(start)) => {
                    self.text_run = 0;
                    let depth = self.depth.saturating_add(1);
                    if depth > self.limits.max_depth {
                        return Err(ReadError::DepthLimit {
                            position: Position::at(self.input, offset),
                            limit: self.limits.max_depth,
                        });
                    }
                    validate_attributes(&start, self.input, offset)?;
                    self.depth = depth;
                    return Ok(Event::Start(Element::new(
                        start,
                        name_at(self.input, offset, false),
                        depth,
                        offset,
                    )));
                }
                Ok(QuickEvent::Empty(empty)) => {
                    self.text_run = 0;
                    let depth = self.depth.saturating_add(1);
                    if depth > self.limits.max_depth {
                        return Err(ReadError::DepthLimit {
                            position: Position::at(self.input, offset),
                            limit: self.limits.max_depth,
                        });
                    }
                    validate_attributes(&empty, self.input, offset)?;
                    return Ok(Event::Empty(Element::new(
                        empty,
                        name_at(self.input, offset, false),
                        depth,
                        offset,
                    )));
                }
                Ok(QuickEvent::End(_end)) => {
                    self.text_run = 0;
                    let depth = self.depth;
                    self.depth = self.depth.saturating_sub(1);
                    return Ok(Event::End(EndTag::new(
                        name_at(self.input, offset, true),
                        depth,
                        offset,
                    )));
                }
                Ok(QuickEvent::Text(text)) => {
                    let raw: Cow<'a, [u8]> = text.into_inner();
                    self.add_text_run(raw.len(), offset)?;
                    return Ok(Event::Text(Text::new(raw, false, self.depth, offset)));
                }
                Ok(QuickEvent::CData(cdata)) => {
                    let raw: Cow<'a, [u8]> = cdata.into_inner();
                    self.add_text_run(raw.len(), offset)?;
                    return Ok(Event::Text(Text::new(raw, true, self.depth, offset)));
                }
                Ok(QuickEvent::GeneralRef(reference)) => {
                    let length = reference.len().saturating_add(2);
                    self.add_text_run(length, offset)?;
                    return Ok(Event::EntityRef(EntityRef::new(
                        reference, self.depth, offset,
                    )));
                }
                Ok(QuickEvent::Comment(_) | QuickEvent::PI(_) | QuickEvent::Decl(_)) => {}
                Ok(QuickEvent::DocType(_)) => {
                    return Err(ReadError::ForbiddenDoctype {
                        position: Position::at(self.input, offset),
                    });
                }
                Ok(QuickEvent::Eof) => {
                    if self.depth == 0 {
                        return Ok(Event::Eof);
                    }
                    return Err(ReadError::UnexpectedEof {
                        position: Position::at(self.input, self.inner.buffer_position()),
                        depth: self.depth,
                    });
                }
            }
        }
    }

    fn add_text_run(&mut self, length: usize, offset: u64) -> Result<(), ReadError> {
        self.text_run = self.text_run.saturating_add(length);
        if self.text_run > self.limits.max_text_run {
            return Err(ReadError::TextLimit {
                position: Position::at(self.input, offset),
                limit: self.limits.max_text_run,
            });
        }
        Ok(())
    }
}

/// Extracts the raw qualified name from the input without borrowing the
/// parser's event buffer.
///
/// `quick-xml` exposes tag names through a borrow of the event, which cannot be
/// returned with the input lifetime. Re-slicing the input keeps every name
/// borrowed from the caller's bytes and therefore zero-copy.
fn name_at(input: &[u8], offset: u64, closing: bool) -> Name<'_> {
    let offset = usize::try_from(offset).unwrap_or(usize::MAX);
    let mut start = offset.saturating_add(1);
    if closing {
        start = start.saturating_add(1);
    }
    let tail = input.get(start..).unwrap_or_default();
    let end = tail
        .iter()
        .position(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n' | b'/' | b'>'))
        .unwrap_or(tail.len());
    Name::new(tail.get(..end).unwrap_or_default())
}

fn validate_attributes(
    start: &quick_xml::events::BytesStart<'_>,
    input: &[u8],
    offset: u64,
) -> Result<(), ReadError> {
    for attribute in start.attributes() {
        if let Err(error) = attribute {
            return Err(ReadError::Attribute {
                position: Position::at(input, offset),
                message: error.to_string(),
            });
        }
    }
    Ok(())
}
