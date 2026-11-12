//! Typed document model for FPA12/FPR12 and FSM10.
//!
//! The model keeps borrowed slices from the input where possible and uses
//! `SmallVec` for per-body collections. It represents the multi-body *lotto*
//! natively: one `FatturaElettronicaBody` is not a special case.
//!
//! # Stability
//!
//! Internal crate API until E9.1.

// Filled in by E2.2–E2.4.
