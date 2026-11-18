//! L1 — streaming XML ingestion.
//!
//! One pass over borrowed bytes, dispatching on namespace-agnostic local names.
//! The official schema leaves children unqualified (`elementFormDefault` is not
//! declared, so it defaults to `unqualified`), which makes prefix-based matching
//! wrong by construction (`P1 §5.1`). Encoding declaration is checked against
//! the actual bytes in E2.5; BOM, processing instructions and comments are
//! tolerated here (`P3 §8` plumbing items 1–6).
//!
//! No DOM is built on the validation path. Names and values borrow from the
//! input; only findings and the two root strings kept for reporting allocate.
//!
//! ```text
//! bytes → Reader → Event::{Start,Empty,End,Text,EntityRef,Eof} → model / scan
//! ```
//!
//! DTDs are rejected before any entity declaration is parsed, removing the
//! external-entity (XXE) attack surface. Depth and text-run limits are resource
//! guards: exceeding them yields a typed error, which [`scan`] reports as a
//! warning rather than aborting.
//!
//! # Stability
//!
//! Internal crate API. The event shape may change until E9.1 freezes the public
//! API; callers outside the crate must use `validate` instead.

mod error;
mod event;
mod local_name;
mod name;
mod reader;
mod scan;

pub use error::{Position, ReadError};
pub use event::{
    Attribute, AttributeError, Attributes, Element, EndTag, EntityError, EntityRef, Event, Text,
};
pub use local_name::{RootKind, is_signature, root_kind};
pub use name::Name;
pub use reader::{Limits, Reader};
pub use scan::{
    RULE_DEPTH_LIMIT, RULE_FORBIDDEN_DOCTYPE, RULE_TEXT_LIMIT, RULE_UNDEFINED_ENTITY,
    RULE_WELL_FORMED, RootElement, Scan, scan,
};
