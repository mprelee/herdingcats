#![warn(missing_docs)]

//! herdingcats — a deterministic, undoable game-event engine.
//!
//! ## State machine model
//!
//! The engine is a **Mealy machine**: `S` is the automaton's internal memory;
//! events (inputs of type `I`) drive transitions; mutations (outputs of type
//! `M`) transform state; behaviors implement the transition function. Each
//! dispatch produces a new state *and* a set of mutations that encode the
//! transition. Undo reverses the mutation sequence, restoring the prior state
//! without storing full snapshots.
//!
//! ## Four concepts
//!
//! | Type | Role |
//! |------|------|
//! | [`Engine<S,M,I,P>`](Engine) | The automaton: holds state, runs behaviors, commits actions |
//! | [`Behavior<S,M,I,P>`](Behavior) | The transition function: translates events into mutations via `before`/`after` hooks |
//! | [`Mutation<S>`](Mutation) | The output: atomic, invertible state change; `apply` advances, `undo` reverses |
//! | [`Action<M>`](Action) | The transition batch: collects mutations for a single dispatch; reversible iff all mutations are |
//!
//! ## Dispatch pipeline
//!
//! ```text
//! Dispatch pipeline
//! ─────────────────
//! Event (I)
//!   │
//!   ├─► [Behavior.before()]  ← inject/cancel mutations
//!   │     (ascending priority)
//!   │
//!   ├─► [Mutation.apply()]   ← advance state S → S'
//!   │
//!   └─► [Behavior.after()]   ← react to new state S'
//!         (descending priority)
//!
//! Undo path (reversible actions only)
//! ────────────────────────────────────
//! [Mutation.undo()]    ← restore S' → S
//! [Behavior.on_undo()] ← reverse behavior-internal state
//! ```
//!
//! ## Quick start
//!
//! ```
//! use herdingcats::{Engine, Mutation, Behavior, Action};
//!
//! #[derive(Clone)]
//! struct Counter(i32);
//!
//! #[derive(Clone)]
//! enum CounterOp { Inc }
//!
//! impl Mutation<Counter> for CounterOp {
//!     fn apply(&self, state: &mut Counter) { state.0 += 1; }
//!     fn undo(&self, state: &mut Counter)  { state.0 -= 1; }
//!     fn hash_bytes(&self) -> Vec<u8>      { vec![1] }
//! }
//!
//! struct LogBehavior;
//!
//! impl Behavior<Counter, CounterOp, (), u8> for LogBehavior {
//!     fn id(&self) -> &'static str { "log" }
//!     fn priority(&self) -> u8    { 0 }
//! }
//!
//! let mut engine = Engine::new(Counter(0));
//! engine.add_behavior(LogBehavior);
//!
//! let mut tx = Action::new();
//! tx.mutations.push(CounterOp::Inc);
//! let _ = engine.dispatch((), tx);
//!
//! assert_eq!(engine.read().0, 1);
//!
//! engine.undo();
//! assert_eq!(engine.read().0, 0);
//! ```

mod action;
mod behavior;
mod engine;
mod hash;
mod mutation;

pub use action::Action;
pub use behavior::Behavior;
pub use engine::Engine;
pub use mutation::Mutation;
