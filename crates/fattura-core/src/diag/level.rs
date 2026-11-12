//! Validation layers `L0` through `L5`.
//!
//! The six layers are defined in `PLAN §1.1`: `L0` file, `L1` well-formedness,
//! `L2` schema conformance, `L3` content rules, `L4` external registries (not
//! checked offline) and `L5` lifecycle (not checked offline). A finding always
//! names the layer that produced it.

use core::fmt;

use serde::{Deserialize, Serialize};

/// A validation layer.
///
/// Layers `L4` and `L5` are never the level of a [`Finding`](crate::diag::Finding)
/// in an offline report; they appear in
/// [`NotChecked`](crate::diag::NotChecked) instead (`PLAN §1.2`, `PLAN §5.2`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Level {
    /// `L0` — file name, size, archive integrity, encoding, `.p7m` envelope.
    L0,
    /// `L1` — XML well-formedness and declared encoding.
    L1,
    /// `L2` — XSD conformance.
    L2,
    /// `L3` — offline content rules (`004xx` and recommended checks).
    L3,
    /// `L4` — external registries; not checked offline.
    L4,
    /// `L5` — invoice lifecycle; not checked offline.
    L5,
}

impl Level {
    /// Every layer, in ascending order.
    pub const ALL: [Self; 6] = [Self::L0, Self::L1, Self::L2, Self::L3, Self::L4, Self::L5];

    /// Returns the two-character label, `L0` through `L5`.
    ///
    /// No allocation is performed.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::L0 => "L0",
            Self::L1 => "L1",
            Self::L2 => "L2",
            Self::L3 => "L3",
            Self::L4 => "L4",
            Self::L5 => "L5",
        }
    }
}

impl fmt::Display for Level {
    /// Writes `L0` through `L5` without allocating.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
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

    use serde_json::json;

    use super::Level;

    #[test]
    fn level_display_serde_and_ordering_cover_every_variant() {
        let expected = ["L0", "L1", "L2", "L3", "L4", "L5"];
        for (level, label) in Level::ALL.into_iter().zip(expected) {
            assert_eq!(level.as_str(), label);
            assert_eq!(level.to_string(), label);
            assert_eq!(serde_json::to_value(level).unwrap(), json!(label));
            assert_eq!(
                serde_json::from_value::<Level>(json!(label)).unwrap(),
                level
            );
        }

        assert_eq!(Level::ALL.len(), 6);
        assert!(Level::L0 < Level::L1);
        assert!(Level::L1 < Level::L2);
        assert!(Level::L2 < Level::L3);
        assert!(Level::L3 < Level::L4);
        assert!(Level::L4 < Level::L5);
    }
}
