//! # HerdingCats
//!
//! A deterministic, auditable rule-orchestration framework for turn-based simulations.
//!
//! ## Philosophy
//!
//! Game logic is full of "cats to herd" — rules that fire at unpredictable times,
//! effects that interact in surprising ways, and bugs that only reproduce under
//! exact conditions. HerdingCats tames this by making every state change traceable
//! to the rule that caused it.
//!
//! The framework is built on three foundational concepts:
//!
//! - **[`Effect`](effect::Effect) = intent** — a symbolic, pure description of what should
//!   happen. Effects are values: they can be logged, replayed, and inspected
//!   independently of any live state.
//!
//! - **[`Delta`](delta::Delta) = proof** — the record of an effect being applied to a
//!   specific state. A Delta pairs the symbolic intent with cryptographic evidence
//!   of what the state looked like before and after.
//!
//! - **[`Rule`](rule::Rule) = cause** — the decision layer. Rules observe state and emit
//!   Effects when their conditions are met. They remain ignorant of the audit
//!   machinery; the [`RuleEngine`](rule::RuleEngine) handles hashing, stamping, and
//!   chain building.
//!
//! Together, these three types answer the fundamental question of any simulation:
//! **not just what changed, but why, and what the world looked like before and after.**
//!
//! ## Design Principles
//!
//! ### Separation of intent and outcome
//!
//! `Effect` values describe intent at the point of generation. `Damage(100)` applied
//! to an entity with 10 HP is still stored and logged as `Damage(100)`. Clamping,
//! saturation, and immune cases are handled inside `Effect::apply`. This keeps the
//! audit trail honest: the log shows the rule's *decision*, and the before/after
//! hashes in the Delta show the *actual result*.
//!
//! ### Purity and determinism
//!
//! All `Effect::apply` calls are pure functions: given the same state, they always
//! produce the same new state. No hidden randomness, no global mutation. This makes
//! the entire simulation replayable from any checkpoint.
//!
//! ### Full traceability
//!
//! Every [`Delta`](delta::Delta) carries the ID of the [`Rule`](rule::Rule) that
//! produced it. [`Changeset::trace`](delta::Changeset::trace) lets you ask "show me
//! everything the Poison rule did this turn" and get back exactly those deltas.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use herdingcats::prelude::*;
//!
//! // Define your state
//! #[derive(Clone, Hash)]
//! struct HealthState { current: u32, max: u32 }
//!
//! // Define effects as enums
//! enum HealthEffect { Heal(u32), Damage(u32) }
//!
//! // Implement Effect<S>
//! impl Effect<HealthState> for HealthEffect { /* ... */ }
//!
//! // Define rule IDs
//! #[derive(Clone, Copy, PartialEq, Eq, Hash)]
//! enum RuleId { Regen, Poison }
//!
//! // Implement Rule<E, S, R> for each rule struct
//! struct PoisonRule;
//! impl Rule<HealthEffect, HealthState, RuleId> for PoisonRule { /* ... */ }
//!
//! // Build and evaluate
//! let engine = RuleEngine::new(vec![Box::new(PoisonRule)]);
//! let changeset = engine.evaluate(&state);
//! assert!(changeset.is_consistent());
//! ```

pub mod delta;
pub mod effect;
pub mod rule;

pub use delta::*;
pub use effect::*;
pub use rule::*;
