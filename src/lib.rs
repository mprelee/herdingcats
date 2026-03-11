#![warn(missing_docs)]

//! herdingcats — a deterministic, undoable game-event engine.

mod action;
mod behavior;
mod engine;
mod hash;
mod mutation;

pub use action::Action;
pub use behavior::Behavior;
pub use engine::Engine;
pub use mutation::Mutation;
