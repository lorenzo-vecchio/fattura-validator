//! Namespace-agnostic local-name dispatch.
//!
//! The reader never resolves namespaces. Names are matched against byte-string
//! tables so the hot path performs only length comparisons and `memcmp` calls,
//! with no allocation and no hash map (`PLAN §2.4`).
//!
//! Only the dispatch needed by the ingestion layer lives here; the generated
//! model tables of E2.2 and the schema tables of E3.2 extend this strategy
//! rather than replacing it. The tables are deliberately **not** case folding:
//! XML names are case-sensitive (`P1 §5.1`).

/// The two official invoice roots.
///
/// `FatturaElettronica` covers both `FPA12` and `FPR12`; the transmission format
/// is selected by `@versione` and `<FormatoTrasmissione>` (`P1 §2.1`). The
/// simplified invoice uses a different root and namespace (`P1 §5.1`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RootKind {
    /// `FatturaElettronica` (`FPA12` or `FPR12`).
    Invoice,
    /// `FatturaElettronicaSemplificata` (`FSM10`).
    Simplified,
}

/// Classifies an element local name as one of the two official roots.
///
/// This is a byte-match table: no string is built and the comparison does not
/// depend on the element prefix or on namespace resolution (`P1 §5.1`,
/// `P3 §8` plumbing item 6).
#[must_use]
pub fn root_kind(local: &[u8]) -> Option<RootKind> {
    match local {
        b"FatturaElettronica" => Some(RootKind::Invoice),
        b"FatturaElettronicaSemplificata" => Some(RootKind::Simplified),
        _ => None,
    }
}

/// Returns `true` for the optional `XAdES` signature root of the `ds` namespace
/// (`P1 §5.1` item 2). The caller must still check placement; this predicate is
/// only a local-name dispatch.
#[must_use]
pub fn is_signature(local: &[u8]) -> bool {
    local == b"Signature"
}

#[cfg(test)]
mod tests {
    use super::{RootKind, is_signature, root_kind};

    #[test]
    fn root_dispatch_matches_both_official_roots_exactly() {
        assert_eq!(root_kind(b"FatturaElettronica"), Some(RootKind::Invoice));
        assert_eq!(
            root_kind(b"FatturaElettronicaSemplificata"),
            Some(RootKind::Simplified)
        );
        assert_eq!(root_kind(b"FatturaElettronicaHeader"), None);
        assert_eq!(root_kind(b"fatturaelettronica"), None);
        assert_eq!(root_kind(b""), None);
        assert_eq!(root_kind(b"FatturaElettronica "), None);
    }

    #[test]
    fn signature_dispatch_is_exact_and_case_sensitive() {
        assert!(is_signature(b"Signature"));
        assert!(!is_signature(b"signature"));
        assert!(!is_signature(b"SignatureValue"));
        assert!(!is_signature(b""));
    }
}
