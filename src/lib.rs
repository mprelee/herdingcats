#![warn(missing_docs)]

//! herdingcats — a deterministic, undoable game-event engine.

mod hash;
mod operation;
mod transaction;
mod rule;
mod engine;

pub use operation::Operation;
pub use transaction::{RuleLifetime, Transaction};
pub use rule::Rule;
pub use engine::Engine;
