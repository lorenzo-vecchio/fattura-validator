//! Shared model primitives: borrowed text, index-preserving lists and the
//! record of elements the model does not recognise.
//!
//! Strings borrow from the input unless an XML entity reference forces a
//! decoded copy; a healthy invoice therefore builds no `String` for its text.
//! Repeated elements use [`List`], a `SmallVec` with one inline slot: a body
//! with a single line pays no heap allocation, while large lots spill to the
//! heap without changing the order of the elements.

use std::borrow::Cow;

use smallvec::SmallVec;

use crate::xml::Name;

/// A text leaf borrowed from the input, or owned when entity references were
/// decoded.
///
/// A `Cow::Owned` value appears only when the input contained an entity such as
/// `&amp;` or a character reference. No other conversion allocates.
pub type Text<'a> = Cow<'a, str>;

/// An ordered, index-preserving collection of repeated elements.
///
/// The 1-based occurrence index used by [`ElementPath`](crate::diag::ElementPath)
/// is the element's position in this list plus one. The inline capacity is one:
/// the common single child stays on the stack, and a lotto body with many lines
/// spills to the heap exactly once.
pub type List<T> = SmallVec<[T; 1]>;

/// An element that is not part of the FPA12/FPR12 tree for its position.
///
/// Unknown elements are kept, in document order, so the XSD layer can report
/// them with their byte offset instead of silently discarding evidence
/// (`PLAN §2.2`). The name borrows from the input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnknownElement<'a> {
    name: Name<'a>,
    offset: u64,
    depth: usize,
}

impl<'a> UnknownElement<'a> {
    /// Records an unrecognised element.
    pub const fn new(name: Name<'a>, offset: u64, depth: usize) -> Self {
        Self {
            name,
            offset,
            depth,
        }
    }

    /// Returns the qualified name as written in the input.
    #[must_use]
    pub const fn name(self) -> Name<'a> {
        self.name
    }

    /// Returns the byte offset of the opening `<`.
    #[must_use]
    pub const fn offset(self) -> u64 {
        self.offset
    }

    /// Returns the 1-based element depth at which the element appeared.
    #[must_use]
    pub const fn depth(self) -> usize {
        self.depth
    }
}
