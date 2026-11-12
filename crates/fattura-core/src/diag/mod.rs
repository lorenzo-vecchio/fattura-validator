//! Diagnostic vocabulary shared by every validation layer.
//!
//! This module owns the stable types of the public contract: [`Finding`],
//! [`SdiCode`], [`Level`], [`Severity`], [`RuleId`], [`ElementPath`] and the
//! layered [`Report`]. Every other module speaks these types; none may invent a
//! private error vocabulary.
//!
//! The model follows the report philosophy of `PLAN §1.3` and the public API
//! sketch of `PLAN §4`: a finding always carries an official `SdI` code when one
//! exists, a severity, a layer, an element path and a stable rule id. Items that
//! could not be checked are disclosed explicitly with [`NotChecked`] instead of
//! being silently ignored (`PLAN §5.2`, `P3 §7`).
//!
//! # Stability
//!
//! The JSON projection of [`Report`] is the cross-language contract. The Rust
//! types may gain fields in a minor release only when [`REPORT_SCHEMA_VERSION`]
//! is bumped and snapshots are updated.
//!
//! [`REPORT_SCHEMA_VERSION`]: crate::REPORT_SCHEMA_VERSION

mod code;
mod finding;
mod level;
mod path;
mod report;
mod rule;
mod severity;

pub use code::{ParseSdiCodeError, SdiCode};
pub use finding::Finding;
pub use level::Level;
pub use path::{ElementPath, Segment};
pub use report::{LevelSummary, NotChecked, Report, ReportBuilder};
pub use rule::RuleId;
pub use severity::{Severity, SeverityMode, SeverityProfile};
