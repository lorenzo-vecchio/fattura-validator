//! L0 — file layer: name, size, ZIP archive, `.p7m`, encoding.
//!
//! Callers pass bytes plus the original file name; no filesystem access happens
//! here. Archive handling is guarded against zip bombs and path traversal.
//!
//! # Stability
//!
//! Internal crate API. `zip` and `p7m` are feature-gated.

// Filled in by E7.1–E7.5 and E8.4.
