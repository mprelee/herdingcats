//! # HerdingCats
//!
//! A deterministic, turn-based game engine where an ordered set of statically
//! known behaviors resolves every input unambiguously.
//!
//! ## Quick Start
//!
//! 1. Define your game types and bundle them into an [`EngineSpec`] impl.
//! 2. Implement [`Behavior`] for each rule in your game.
//! 3. Construct an `Engine` (Phase 2) and call `dispatch(input, reversibility)`.
//!
//! All public types are re-exported at the crate root.

mod spec;
mod behavior;
mod outcome;

pub use crate::spec::EngineSpec;
pub use crate::behavior::{Behavior, BehaviorResult};
pub use crate::outcome::{EngineError, Frame, Outcome};
