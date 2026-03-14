//! Behavior evaluation contract for the HerdingCats engine.
//!
//! Every rule in a game is expressed as a [`BehaviorDef<E>`] entry. At dispatch
//! time the engine iterates behaviors in deterministic order — sorted by
//! `(order_key, name)` — calling the `evaluate` fn pointer on each one with an
//! immutable borrow of the current state. The immutable borrow is structural
//! (enforced by the type system): a behavior *cannot* mutate state directly.
//! Instead it returns a [`BehaviorResult`] describing either a list of diffs to
//! apply or a reason to stop dispatch early.
//!
//! ## Design
//!
//! The behavior set is static and known at compile time. `BehaviorDef<E>` is a
//! plain struct with fn pointer fields — no trait objects, no `dyn` dispatch,
//! no runtime registration. Behaviors are constructed once, sorted by
//! `(order_key, name)`, and stored in the engine's `Vec<BehaviorDef<E>>`. The
//! engine calls each `evaluate` fn pointer in order, collecting diffs that are
//! applied centrally.

use crate::outcome::NonCommittedOutcome;
use crate::spec::EngineSpec;

/// The result returned by a single [`BehaviorDef`] evaluate call.
///
/// # Example
///
/// ```
/// # use herdingcats::{BehaviorResult, NonCommittedOutcome};
/// // Continue with two diffs
/// let r: BehaviorResult<u8, String> = BehaviorResult::Continue(vec![1, 2]);
/// if let BehaviorResult::Continue(diffs) = &r {
///     assert_eq!(diffs, &vec![1u8, 2u8]);
/// }
///
/// // Stop with an explicit non-committed outcome
/// let s: BehaviorResult<u8, String> =
///     BehaviorResult::Stop(NonCommittedOutcome::Aborted("out of bounds".to_string()));
/// if let BehaviorResult::Stop(NonCommittedOutcome::Aborted(msg)) = &s {
///     assert_eq!(msg, "out of bounds");
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum BehaviorResult<D, N> {
    /// Behavior produced zero or more diffs; dispatch continues to the next behavior.
    ///
    /// An empty `Vec` is valid — the behavior had nothing to contribute this turn.
    Continue(Vec<D>),

    /// Behavior halted dispatch immediately. The payload explicitly declares which
    /// [`NonCommittedOutcome`] variant applies: `InvalidInput`, `Disallowed`, or `Aborted`.
    /// The engine maps this directly to the corresponding [`Outcome`](crate::outcome::Outcome) variant.
    Stop(NonCommittedOutcome<N>),
}

/// A single rule in the game, expressed as a plain struct with fn pointer fields.
///
/// The behavior set is static and determined at engine construction. `BehaviorDef<E>`
/// requires no trait implementation — simply populate its fields with a name, an
/// `order_key`, and an `evaluate` fn pointer.
///
/// Behaviors are sorted once by `(order_key, name)` when the engine is constructed.
/// Lower `order_key` values run first; equal keys are broken by `name` (lexicographic),
/// guaranteeing deterministic evaluation order without relying on memory addresses or
/// insertion order.
///
/// # Mutation prevention (structural)
///
/// The `evaluate` fn pointer signature receives `state: &E::State` — a shared borrow.
/// The compiler prevents any evaluate function from mutating the live state, without
/// requiring any run-time checks or `unsafe` code.
///
/// The following snippet is intentionally rejected by the compiler:
///
/// ```compile_fail
/// # use herdingcats::{EngineSpec, BehaviorDef, BehaviorResult};
/// # struct MySpec;
/// # impl EngineSpec for MySpec {
/// #     type State = Vec<u8>;
/// #     type Input = u8;
/// #     type Diff = u8;
/// #     type Trace = String;
/// #     type NonCommittedInfo = String;
/// #     type OrderKey = u32;
/// # }
/// fn bad_eval(
///     _input: &u8,
///     state: &Vec<u8>,   // shared borrow — mutation not allowed
/// ) -> BehaviorResult<u8, String> {
///     // ERROR: cannot assign to `*state` because it is behind a `&` reference
///     *state = Default::default();
///     BehaviorResult::Continue(vec![])
/// }
///
/// let _b = BehaviorDef::<MySpec> {
///     name: "bad",
///     order_key: 0,
///     evaluate: bad_eval,
/// };
/// ```
///
/// # Example
///
/// ```
/// # use herdingcats::{EngineSpec, BehaviorDef, BehaviorResult};
/// # use herdingcats::Apply;
/// # struct MySpec;
/// # impl EngineSpec for MySpec {
/// #     type State = Vec<u8>;
/// #     type Input = u8;
/// #     type Diff = u8;
/// #     type Trace = String;
/// #     type NonCommittedInfo = String;
/// #     type OrderKey = u32;
/// # }
/// # impl Apply<MySpec> for u8 {
/// #     fn apply(&self, state: &mut Vec<u8>) -> Vec<String> {
/// #         state.push(*self);
/// #         vec![format!("pushed {}", self)]
/// #     }
/// # }
/// fn echo_eval(input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
///     BehaviorResult::Continue(vec![*input])
/// }
///
/// let behavior = BehaviorDef::<MySpec> {
///     name: "echo",
///     order_key: 0,
///     evaluate: echo_eval,
/// };
///
/// assert_eq!(behavior.name, "echo");
/// assert_eq!(behavior.order_key, 0u32);
/// let result = (behavior.evaluate)(&42u8, &vec![]);
/// assert_eq!(result, BehaviorResult::Continue(vec![42u8]));
/// ```
pub struct BehaviorDef<E: EngineSpec> {
    /// A unique, stable string identifier for this behavior.
    ///
    /// Used as the secondary sort key when two behaviors share the same
    /// `order_key`, guaranteeing deterministic ordering without relying on
    /// memory addresses or insertion order.
    pub name: &'static str,

    /// Primary sort key for deterministic behavior ordering.
    ///
    /// Behaviors are evaluated in ascending order of `(order_key, name)`.
    /// Lower values run first. Equal `order_key` values are broken by `name`.
    pub order_key: E::OrderKey,

    /// Evaluate this behavior against the current input and state.
    ///
    /// Receives immutable borrows of both `input` and `state`. The shared
    /// borrow prevents mutation; diffs are *returned*, not applied in-place.
    ///
    /// Returns [`BehaviorResult::Continue`] with any produced diffs, or
    /// [`BehaviorResult::Stop`] to halt dispatch immediately.
    ///
    /// Note: call as `(behavior.evaluate)(input, state)` — the parentheses
    /// are required because Rust would otherwise interpret `behavior.evaluate(...)`
    /// as a method call rather than a fn pointer call.
    pub evaluate: fn(&E::Input, &E::State) -> BehaviorResult<E::Diff, E::NonCommittedInfo>,
}

impl<E: EngineSpec> std::fmt::Debug for BehaviorDef<E>
where
    E::OrderKey: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BehaviorDef")
            .field("name", &self.name)
            .field("order_key", &self.order_key)
            .field("evaluate", &"fn(...)")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apply::Apply;
    use crate::outcome::NonCommittedOutcome;
    use crate::spec::EngineSpec;

    struct TestSpec;

    impl EngineSpec for TestSpec {
        type State = Vec<u8>;
        type Input = u8;
        type Diff = u8;
        type Trace = String;
        type NonCommittedInfo = String;
        type OrderKey = u32;
    }

    // u8 satisfies the Apply<TestSpec> bound required by EngineSpec::Diff.
    impl Apply<TestSpec> for u8 {
        fn apply(&self, state: &mut Vec<u8>) -> Vec<String> {
            state.push(*self);
            vec![format!("applied {}", self)]
        }
    }

    fn passthrough_eval(_input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
        BehaviorResult::Continue(vec![])
    }

    fn stopping_eval(_input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
        BehaviorResult::Stop(NonCommittedOutcome::Aborted("halted".to_string()))
    }

    #[test]
    fn behavior_def_fields_are_accessible() {
        let b = BehaviorDef::<TestSpec> {
            name: "passthrough",
            order_key: 0u32,
            evaluate: passthrough_eval,
        };
        assert_eq!(b.name, "passthrough");
        assert_eq!(b.order_key, 0u32);
    }

    #[test]
    fn behavior_def_evaluate_fn_pointer_is_callable() {
        let b = BehaviorDef::<TestSpec> {
            name: "passthrough",
            order_key: 0u32,
            evaluate: passthrough_eval,
        };
        let state = vec![1u8, 2u8];
        let result = (b.evaluate)(&42u8, &state);
        // state is still accessible (no move, no mutation)
        assert_eq!(state, vec![1u8, 2u8]);
        assert_eq!(result, BehaviorResult::Continue(vec![]));
    }

    #[test]
    fn behavior_result_continue_is_constructable_and_matchable() {
        let r: BehaviorResult<u8, String> = BehaviorResult::Continue(vec![1u8, 2u8]);
        match r {
            BehaviorResult::Continue(diffs) => assert_eq!(diffs, vec![1u8, 2u8]),
            BehaviorResult::Stop(_) => panic!("expected Continue"),
        }
    }

    #[test]
    fn behavior_result_stop_is_constructable_and_matchable() {
        let r: BehaviorResult<u8, String> =
            BehaviorResult::Stop(NonCommittedOutcome::Aborted("reason".to_string()));
        match r {
            BehaviorResult::Continue(_) => panic!("expected Stop"),
            BehaviorResult::Stop(NonCommittedOutcome::Aborted(msg)) => assert_eq!(msg, "reason"),
            BehaviorResult::Stop(_) => panic!("expected Aborted variant"),
        }
    }

    #[test]
    fn behavior_def_stopping_evaluate_returns_stop() {
        let b = BehaviorDef::<TestSpec> {
            name: "stopper",
            order_key: 1u32,
            evaluate: stopping_eval,
        };
        let result = (b.evaluate)(&0u8, &vec![]);
        assert!(matches!(result, BehaviorResult::Stop(_)));
    }

    #[test]
    fn behavior_def_debug_shows_name_and_order_key() {
        let b = BehaviorDef::<TestSpec> {
            name: "passthrough",
            order_key: 42u32,
            evaluate: passthrough_eval,
        };
        let debug_str = format!("{:?}", b);
        assert!(debug_str.contains("passthrough"), "debug must include name");
        assert!(debug_str.contains("42"), "debug must include order_key");
        assert!(
            debug_str.contains("fn(...)"),
            "debug must show evaluate as fn placeholder"
        );
    }
}
