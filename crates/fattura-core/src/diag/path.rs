//! Element paths for findings.
//!
//! A path is a sequence of named segments, each optionally annotated with a
//! 1-based occurrence index. It renders exactly as the public contract requires
//! (`PLAN §4`):
//!
//! ```text
//! Body[2]/DatiBeniServizi/DettaglioLinee[3]/PrezzoTotale
//! ```
//!
//! Paths are built only when a finding is emitted, so the validation hot path
//! pays nothing for them (`PLAN §2.4`). Rendering writes directly into the
//! caller's formatter; sorting a report may allocate the rendered form.

use core::fmt;
use core::num::NonZeroUsize;

use serde::Serialize;

/// One named segment of an [`ElementPath`].
///
/// The index, when present, is a 1-based occurrence counter. Zero is not a valid
/// index and is represented by `None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Segment {
    name: &'static str,
    index: Option<NonZeroUsize>,
}

impl Segment {
    /// Creates an unindexed segment.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self { name, index: None }
    }

    /// Creates a segment annotated with a 1-based index.
    #[must_use]
    pub const fn indexed(name: &'static str, index: NonZeroUsize) -> Self {
        Self {
            name,
            index: Some(index),
        }
    }

    /// Returns the XML local name of the element.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Returns the 1-based index, or `None` when the element is not repeated.
    #[must_use]
    pub const fn index(self) -> Option<NonZeroUsize> {
        self.index
    }
}

impl fmt::Display for Segment {
    /// Writes `Name` or `Name[n]` without allocating.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name)?;
        if let Some(index) = self.index {
            write!(formatter, "[{index}]")?;
        }
        Ok(())
    }
}

/// The path from the document root to the element that produced a finding.
///
/// Use [`ElementPath::root`] for the document root and append segments with the
/// builder methods or the `push` methods. Equality and ordering are structural
/// (segment name first, then numeric index); report ordering uses the rendered
/// text instead, so the public order matches what a reader sees.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementPath {
    segments: Vec<Segment>,
}

impl ElementPath {
    /// Creates the empty path for the document root.
    #[must_use]
    pub const fn root() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Appends a prepared segment, consuming the path.
    #[must_use]
    pub fn with_segment(mut self, segment: Segment) -> Self {
        self.push_segment(segment);
        self
    }

    /// Appends an unindexed named segment, consuming the path.
    #[must_use]
    pub fn with_name(mut self, name: &'static str) -> Self {
        self.push(name);
        self
    }

    /// Appends an indexed named segment, consuming the path.
    ///
    /// Indices are 1-based. Passing `0` appends an unindexed segment: an element
    /// occurrence must be at least one, and silently ignoring the invalid index
    /// is safer than rendering `[0]` in a report.
    #[must_use]
    pub fn with_index(mut self, name: &'static str, index: usize) -> Self {
        self.push_index(name, index);
        self
    }

    /// Appends a prepared segment.
    pub fn push_segment(&mut self, segment: Segment) {
        self.segments.push(segment);
    }

    /// Appends an unindexed named segment.
    pub fn push(&mut self, name: &'static str) {
        self.segments.push(Segment::new(name));
    }

    /// Appends an indexed named segment.
    ///
    /// Indices are 1-based; `0` is treated as absent, as in
    /// [`ElementPath::with_index`].
    pub fn push_index(&mut self, name: &'static str, index: usize) {
        match NonZeroUsize::new(index) {
            Some(index) => self.segments.push(Segment::indexed(name, index)),
            None => self.segments.push(Segment::new(name)),
        }
    }

    /// Returns the segments from the root to the element.
    #[must_use]
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// Returns the number of segments.
    #[must_use]
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Returns `true` when the path is the document root.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
}

impl fmt::Display for ElementPath {
    /// Writes `Name[n]/Name/...` without allocating.
    ///
    /// The root path renders as the empty string.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (position, segment) in self.segments.iter().enumerate() {
            if position > 0 {
                formatter.write_str("/")?;
            }
            fmt::Display::fmt(segment, formatter)?;
        }
        Ok(())
    }
}

impl Serialize for ElementPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
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

    use core::num::NonZeroUsize;

    use serde_json::json;

    use super::{ElementPath, Segment};

    /// Fails after a fixed number of successful writes, to exercise every
    /// formatter-error branch without a panic.
    struct BudgetWriter {
        remaining: usize,
    }

    impl core::fmt::Write for BudgetWriter {
        fn write_str(&mut self, _text: &str) -> core::fmt::Result {
            if self.remaining == 0 {
                return Err(core::fmt::Error);
            }
            self.remaining -= 1;
            Ok(())
        }
    }

    fn render_with_budget(path: &ElementPath, budget: usize) -> core::fmt::Result {
        let mut writer = BudgetWriter { remaining: budget };
        core::fmt::write(&mut writer, format_args!("{path}"))
    }

    fn example_path() -> ElementPath {
        ElementPath::root()
            .with_index("Body", 2)
            .with_name("DatiBeniServizi")
            .with_index("DettaglioLinee", 3)
            .with_name("PrezzoTotale")
    }

    #[test]
    fn element_path_renders_the_contract_example() {
        let path = example_path();
        assert_eq!(
            path.to_string(),
            "Body[2]/DatiBeniServizi/DettaglioLinee[3]/PrezzoTotale"
        );
        assert_eq!(
            serde_json::to_value(&path).unwrap(),
            json!("Body[2]/DatiBeniServizi/DettaglioLinee[3]/PrezzoTotale")
        );
        assert_eq!(path.len(), 4);
        assert!(!path.is_empty());
    }

    #[test]
    fn element_path_display_propagates_formatter_errors() {
        let indexed = ElementPath::root().with_index("Linea", 2);
        let unindexed = ElementPath::root()
            .with_name("Body")
            .with_name("DatiGenerali");

        assert!(render_with_budget(&indexed, 0).is_err());
        assert!(
            render_with_budget(&indexed, 1).is_err(),
            "the index write fails after the name"
        );
        assert!(
            render_with_budget(&unindexed, 1).is_err(),
            "the separator write fails after the first segment"
        );
        assert!(
            render_with_budget(&unindexed, 2).is_err(),
            "the second segment write fails"
        );
        assert!(render_with_budget(&unindexed, 3).is_ok());
    }

    #[test]
    fn element_path_root_is_empty_and_renders_empty() {
        let root = ElementPath::root();
        assert_eq!(root.to_string(), "");
        assert!(root.is_empty());
        assert_eq!(root.len(), 0);
        assert_eq!(root, ElementPath::default());
        assert_eq!(root.segments().len(), 0);
        assert_eq!(serde_json::to_value(&root).unwrap(), json!(""));
    }

    #[test]
    fn element_path_index_zero_is_treated_as_absent() {
        let path = ElementPath::root()
            .with_index("Body", 0)
            .with_name("DatiGenerali")
            .with_index("Data", 1);
        assert_eq!(path.to_string(), "Body/DatiGenerali/Data[1]");
        assert_eq!(path.segments()[0].index(), None);
        assert_eq!(
            path.segments()[2].index(),
            Some(NonZeroUsize::new(1).unwrap())
        );

        let mut pushed = ElementPath::root();
        pushed.push_index("Body", 0);
        pushed.push("DatiGenerali");
        assert_eq!(pushed.to_string(), "Body/DatiGenerali");
    }

    #[test]
    fn element_path_push_and_segment_constructors_cover_both_forms() {
        let mut path = ElementPath::root();
        path.push("Body");
        path.push_index("DettaglioLinee", 10);
        path.push_segment(Segment::new("PrezzoTotale"));
        path.push_segment(Segment::indexed(
            "AltriDatiGestionali",
            NonZeroUsize::new(2).unwrap(),
        ));

        assert_eq!(
            path.to_string(),
            "Body/DettaglioLinee[10]/PrezzoTotale/AltriDatiGestionali[2]"
        );
        assert_eq!(path.segments().len(), 4);
        assert_eq!(path.segments()[1].name(), "DettaglioLinee");
        assert_eq!(
            path.segments()[1].index(),
            Some(NonZeroUsize::new(10).unwrap())
        );
        assert_eq!(
            path.clone().with_name("Extra").to_string(),
            "Body/DettaglioLinee[10]/PrezzoTotale/AltriDatiGestionali[2]/Extra"
        );
        assert_eq!(
            path.clone().with_segment(Segment::new("Extra")).len(),
            path.len() + 1
        );
        assert_eq!(
            Segment::indexed("Linea", NonZeroUsize::new(3).unwrap()).to_string(),
            "Linea[3]"
        );
        assert_eq!(Segment::new("Linea").to_string(), "Linea");
    }

    #[test]
    fn element_path_equality_and_ordering_are_structural() {
        let first = ElementPath::root().with_name("Body").with_index("Linea", 2);
        let second = ElementPath::root()
            .with_name("Body")
            .with_index("Linea", 10);

        assert_eq!(first, first.clone());
        assert_ne!(first, second);
        assert!(first < second, "numeric indices order numerically");
        assert_eq!(
            ElementPath::root(),
            ElementPath::root(),
            "root paths compare equal"
        );
        assert!(ElementPath::root() < ElementPath::root().with_name("Body"));

        let mut paths = [second, ElementPath::root(), first];
        paths.sort();
        assert_eq!(
            paths.iter().map(ToString::to_string).collect::<Vec<_>>(),
            vec!["", "Body/Linea[2]", "Body/Linea[10]"]
        );
    }
}
