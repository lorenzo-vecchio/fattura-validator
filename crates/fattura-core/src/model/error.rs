//! Typed failures of the model parser.
//!
//! The parser stops on a structural failure it cannot interpret (malformed XML,
//! an element inside a text leaf, an undefined entity, invalid UTF-8). It does
//! **not** stop for lexical problems in a value: a malformed amount becomes
//! `None` and is reported by the schema layer, so one bad number never hides the
//! rest of the document.

use core::fmt;

use crate::xml::ReadError;

/// A failure while building the typed model.
#[derive(Debug)]
pub enum ModelError {
    /// The XML reader rejected the input (well-formedness, limits, DTD).
    Xml(ReadError),
    /// The input ended before the current element was closed.
    UnexpectedEof,
    /// No root element was found.
    MissingRoot,
    /// The root element is not `FatturaElettronica` (for example FSM10).
    ///
    /// E2.3 adds the simplified root; until then a foreign root is a typed
    /// failure rather than a silently empty model.
    UnsupportedRoot,
    /// An element appeared inside a text leaf.
    UnexpectedElement,
    /// An entity reference names no predefined entity and no character.
    UndefinedEntity,
    /// A text node or attribute value is not valid UTF-8.
    InvalidUtf8,
}

impl fmt::Display for ModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml(error) => write!(formatter, "malformed XML: {error}"),
            Self::UnexpectedEof => {
                formatter.write_str("unexpected end of input in the invoice tree")
            }
            Self::MissingRoot => formatter.write_str("no FatturaElettronica root element"),
            Self::UnsupportedRoot => {
                formatter.write_str("root element is not FatturaElettronica (FPA12/FPR12)")
            }
            Self::UnexpectedElement => formatter.write_str("element inside a text-only field"),
            Self::UndefinedEntity => formatter.write_str("undefined XML entity reference"),
            Self::InvalidUtf8 => formatter.write_str("text is not valid UTF-8"),
        }
    }
}

impl std::error::Error for ModelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml(error) => Some(error),
            _ => None,
        }
    }
}
