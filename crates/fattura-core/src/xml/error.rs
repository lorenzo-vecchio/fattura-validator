//! Typed L1 reader errors with byte, line and column positions.
//!
//! XML syntax failures are not exceptions: they become [`ReadError`] values
//! which the scanner maps to an L1 finding (`P3 §2.1`, code `00106`;
//! `PLAN §5.1`). Error messages are only formatted on the reporting path, never
//! while reading a healthy document.

use core::fmt;

/// A position inside the input buffer.
///
/// `byte` is a zero-based byte offset and is clamped to the input length so a
/// parser position can never point outside the buffer. `line` and `column` are
/// one-based; `column` counts **bytes**, not Unicode scalar values, because XML
/// markup is ASCII-delimited and byte columns are deterministic for every
/// encoding (`P3 §8` plumbing items 1–2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    /// Zero-based byte offset, clamped to the input length.
    pub byte: u64,
    /// One-based line number, counting `\n` line terminators.
    pub line: usize,
    /// One-based byte column within the line.
    pub column: usize,
}

impl Position {
    /// Computes a position from a byte offset.
    ///
    /// The computation is linear in the offset and is only used on the error
    /// path, never for every event.
    #[must_use]
    pub fn at(input: &[u8], byte: u64) -> Self {
        let limit = usize::try_from(byte).unwrap_or(usize::MAX);
        let end = limit.min(input.len());
        let mut line = 1usize;
        let mut column = 1usize;
        for &value in input.get(..end).unwrap_or_default() {
            if value == b'\n' {
                line = line.saturating_add(1);
                column = 1;
            } else {
                column = column.saturating_add(1);
            }
        }
        Self {
            byte: u64::try_from(end).unwrap_or(u64::MAX),
            line,
            column,
        }
    }
}

impl fmt::Display for Position {
    /// Writes `line L, column C (byte B)`.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "line {}, column {} (byte {})",
            self.line, self.column, self.byte
        )
    }
}

/// A failure while reading XML bytes.
///
/// The reader stops at the first failure. The scanner converts it into a
/// finding; callers that use [`Reader`](super::Reader) directly get the typed
/// error and may decide how to report it.
#[derive(Clone, Debug)]
pub enum ReadError {
    /// `quick-xml` rejected the document as malformed.
    Xml {
        /// Position of the parser cursor when the error was raised.
        position: Position,
        /// Underlying parser error, with its own diagnostic text.
        source: quick_xml::Error,
    },
    /// The attribute iterator rejected an attribute.
    Attribute {
        /// Position of the start of the element.
        position: Position,
        /// Parser message, allocated only on the error path.
        message: String,
    },
    /// Element nesting exceeded [`Limits::max_depth`](super::Limits::max_depth).
    DepthLimit {
        /// Position where the limit was exceeded.
        position: Position,
        /// Configured maximum.
        limit: usize,
    },
    /// A contiguous text run exceeded
    /// [`Limits::max_text_run`](super::Limits::max_text_run).
    TextLimit {
        /// Position where the limit was exceeded.
        position: Position,
        /// Configured maximum in bytes.
        limit: usize,
    },
    /// A `DOCTYPE` declaration was found.
    ///
    /// DTDs are rejected before any entity declaration can be processed, which
    /// removes the external-entity (XXE) and entity-expansion attack surface
    /// (`P3 §8` plumbing items 1–6).
    ForbiddenDoctype {
        /// Position of the declaration.
        position: Position,
    },
    /// End of input was reached with elements still open.
    UnexpectedEof {
        /// Position of the end of input.
        position: Position,
        /// Number of elements that were never closed.
        depth: usize,
    },
}

impl ReadError {
    /// Returns the position attached to the error.
    #[must_use]
    pub const fn position(&self) -> Position {
        match self {
            Self::Xml { position, .. }
            | Self::Attribute { position, .. }
            | Self::DepthLimit { position, .. }
            | Self::TextLimit { position, .. }
            | Self::ForbiddenDoctype { position }
            | Self::UnexpectedEof { position, .. } => *position,
        }
    }
}

impl fmt::Display for ReadError {
    /// Writes a complete diagnostic without allocating.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml { position, source } => {
                write!(formatter, "malformed XML: {source} at {position}")
            }
            Self::Attribute { position, message } => {
                write!(formatter, "malformed attribute: {message} at {position}")
            }
            Self::DepthLimit { position, limit } => {
                write!(
                    formatter,
                    "element nesting exceeds {limit} levels at {position}"
                )
            }
            Self::TextLimit { position, limit } => {
                write!(formatter, "text run exceeds {limit} bytes at {position}")
            }
            Self::ForbiddenDoctype { position } => {
                write!(
                    formatter,
                    "DOCTYPE declarations are not allowed at {position}"
                )
            }
            Self::UnexpectedEof { position, depth } => {
                write!(
                    formatter,
                    "end of input reached with {depth} unclosed element(s) at {position}"
                )
            }
        }
    }
}

impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml { source, .. } => Some(source),
            Self::Attribute { .. }
            | Self::DepthLimit { .. }
            | Self::TextLimit { .. }
            | Self::ForbiddenDoctype { .. }
            | Self::UnexpectedEof { .. } => None,
        }
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

    use super::{Position, ReadError};

    #[test]
    fn position_counts_lines_and_byte_columns() {
        let input = b"<a>\n  <b>\n<c/>";
        assert_eq!(
            Position::at(input, 0),
            Position {
                byte: 0,
                line: 1,
                column: 1
            }
        );
        assert_eq!(
            Position::at(input, 4),
            Position {
                byte: 4,
                line: 2,
                column: 1
            }
        );
        assert_eq!(
            Position::at(input, 6),
            Position {
                byte: 6,
                line: 2,
                column: 3
            }
        );

        // Offsets past the end and impossible `usize` values are clamped.
        let end = u64::try_from(input.len()).unwrap();
        assert_eq!(
            Position::at(input, end.saturating_add(100)),
            Position {
                byte: end,
                line: 3,
                column: 5
            }
        );
        assert_eq!(Position::at(input, u64::MAX).byte, end);
    }

    #[test]
    fn position_handles_crlf_and_utf8_bytes() {
        let input = "<a>\r\né\n".as_bytes();
        assert_eq!(Position::at(input, 4).line, 1, "CR does not start a line");
        assert_eq!(Position::at(input, 5).line, 2);
        assert_eq!(Position::at(input, 6).line, 2, "the UTF-8 lead byte");
        assert_eq!(Position::at(input, 7).line, 2, "the UTF-8 tail byte");
        assert_eq!(Position::at(input, 8).line, 3);
    }

    #[test]
    fn errors_display_and_expose_their_position() {
        let position = Position {
            byte: 12,
            line: 3,
            column: 4,
        };

        let xml = ReadError::Xml {
            position,
            source: quick_xml::Error::Syntax(quick_xml::errors::SyntaxError::UnclosedTag),
        };
        assert_eq!(
            xml.to_string(),
            "malformed XML: syntax error: tag not closed: `>` not found before end of input \
             at line 3, column 4 (byte 12)"
        );
        assert!(std::error::Error::source(&xml).is_some());

        let attribute = ReadError::Attribute {
            position,
            message: "unquoted value".to_owned(),
        };
        assert_eq!(
            attribute.to_string(),
            "malformed attribute: unquoted value at line 3, column 4 (byte 12)"
        );
        assert!(std::error::Error::source(&attribute).is_none());

        let depth = ReadError::DepthLimit {
            position,
            limit: 64,
        };
        assert_eq!(
            depth.to_string(),
            "element nesting exceeds 64 levels at line 3, column 4 (byte 12)"
        );
        assert_eq!(depth.position(), position);

        let text = ReadError::TextLimit {
            position,
            limit: 1_024,
        };
        assert_eq!(
            text.to_string(),
            "text run exceeds 1024 bytes at line 3, column 4 (byte 12)"
        );

        let doctype = ReadError::ForbiddenDoctype { position };
        assert_eq!(
            doctype.to_string(),
            "DOCTYPE declarations are not allowed at line 3, column 4 (byte 12)"
        );
        assert_eq!(doctype.position(), position);

        let eof = ReadError::UnexpectedEof { position, depth: 2 };
        assert_eq!(
            eof.to_string(),
            "end of input reached with 2 unclosed element(s) at line 3, column 4 (byte 12)"
        );
        assert_eq!(eof.position(), position);
        assert!(std::error::Error::source(&eof).is_none());
        assert_eq!(xml.position(), position);
        assert_eq!(attribute.position(), position);
        assert_eq!(text.position(), position);
    }
}
