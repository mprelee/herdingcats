#![warn(missing_docs)]

//! `herdingcats` is a deterministic, undoable game-event engine.
//!
//! The runtime crate is intentionally small: you provide handwritten state,
//! operations, events, and rules, and the engine handles ordered dispatch,
//! undo/redo, and replay hashing.
//!
//! v1.1 also adds a build-time DSL path through the separate
//! `herdingcats_codegen` companion crate. That path is build-time only:
//! there is no runtime parser, no runtime scripting surface, and no broad
//! generated `after()` mutation support in this crate.
//!
//! Handwritten-only usage remains fully supported when the DSL path is unused.

mod engine;
mod hash;
mod operation;
mod rule;
mod transaction;

pub use engine::Engine;
pub use operation::Operation;
pub use rule::Rule;
pub use transaction::{RuleLifetime, Transaction};
