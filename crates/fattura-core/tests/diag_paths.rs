//! Path-rendering contract snapshot (`PLAN §4`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]

use core::num::NonZeroUsize;

use fattura_core::diag::{ElementPath, Segment};

#[test]
fn diag_path_rendering_contract_snapshot() {
    let root = ElementPath::root();
    let example = ElementPath::root()
        .with_index("Body", 2)
        .with_name("DatiBeniServizi")
        .with_index("DettaglioLinee", 3)
        .with_name("PrezzoTotale");
    let multi_digit = ElementPath::root()
        .with_index("DettaglioLinee", 10)
        .with_name("AltriDatiGestionali");
    let zero_index = ElementPath::root()
        .with_index("Body", 0)
        .with_name("DatiGenerali");
    let prepared_segment = ElementPath::root()
        .with_segment(Segment::indexed("Allegati", NonZeroUsize::new(7).unwrap()));

    let rendered = serde_json::json!({
        "root": root.to_string(),
        "example": example.to_string(),
        "multi_digit": multi_digit.to_string(),
        "zero_index_is_absent": zero_index.to_string(),
        "prepared_segment": prepared_segment.to_string(),
    });

    insta::assert_json_snapshot!("path_rendering", rendered);
}
