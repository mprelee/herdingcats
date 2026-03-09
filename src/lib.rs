#![warn(missing_docs)]

//! herdingcats — a deterministic, undoable game-event engine.

mod engine;
mod hash;
mod operation;
mod rule;
mod transaction;

pub use engine::Engine;
pub use operation::Operation;
pub use rule::Rule;
pub use transaction::{RuleLifetime, Transaction};
