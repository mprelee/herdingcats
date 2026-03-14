//! # HerdingCats
//!
//! A deterministic, turn-based game engine where an ordered set of statically
//! known behaviors resolves every input unambiguously.
//!
//! ## Quick Start
//!
//! 1. Define your game types and bundle them into an [`EngineSpec`] impl.
//! 2. Define [`BehaviorDef`] entries — plain structs with fn pointer fields.
//! 3. Construct an [`Engine`] and call `dispatch(input, reversibility)`.
//!
//! All public types are re-exported at the crate root.

mod spec;
mod behavior;
mod outcome;
mod apply;
mod reversibility;
mod engine;

pub use crate::spec::EngineSpec;
pub use crate::behavior::{BehaviorDef, BehaviorResult};
pub use crate::outcome::{EngineError, Frame, HistoryDisallowed, NonCommittedOutcome, Outcome};
pub use crate::apply::Apply;
pub use crate::reversibility::Reversibility;
pub use crate::engine::Engine;
