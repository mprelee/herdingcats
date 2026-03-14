//! Behavior evaluation contract for the HerdingCats engine.
//!
//! Every rule in a game is expressed as a [`Behavior`] implementation. At dispatch
//! time the engine iterates behaviors in deterministic order — sorted by
//! `(order_key, name)` — calling [`Behavior::evaluate`] on each one with an
//! immutable borrow of the current state. The immutable borrow is structural
//! (enforced by the type system): a behavior *cannot* mutate state directly.
//! Instead it returns a [`BehaviorResult`] describing either a list of diffs to
//! apply or a reason to stop dispatch early.

use crate::spec::EngineSpec;

/// The result returned by a single [`Behavior::evaluate`] call.
///
/// # Example
///
/// ```
/// # use herdingcats::BehaviorResult;
/// // Continue with two diffs
/// let r: BehaviorResult<u8, String> = BehaviorResult::Continue(vec![1, 2]);
/// if let BehaviorResult::Continue(diffs) = &r {
///     assert_eq!(diffs, &vec![1u8, 2u8]);
/// }
///
/// // Stop with a reason
/// let s: BehaviorResult<u8, String> = BehaviorResult::Stop("out of bounds".to_string());
/// if let BehaviorResult::Stop(msg) = &s {
///     assert_eq!(msg, "out of bounds");
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum BehaviorResult<D, O> {
    /// Behavior produced zero or more diffs; dispatch continues to the next behavior.
    ///
    /// An empty `Vec` is valid — the behavior had nothing to contribute this turn.
    Continue(Vec<D>),

    /// Behavior halted dispatch immediately. The payload carries non-committed
    /// information (e.g. a reason, a suggestion) to return to the caller without
    /// touching the history stack.
    Stop(O),
}

/// A single rule in the game, evaluated once per dispatch call.
///
/// Implementors express one cohesive piece of game logic. The engine calls
/// [`evaluate`](Behavior::evaluate) for each behavior in deterministic order —
/// sorted by `(order_key, name)` pairs — so two behaviors with equal `order_key`
/// are broken by their `name` strings.
///
/// # Dyn-safety
///
/// `Behavior<E>` is object-safe: no method carries a generic type parameter beyond
/// those fixed by `E`. `Box<dyn Behavior<MySpec>>` is a valid type.
///
/// # Mutation prevention (structural)
///
/// The `evaluate` signature receives `state: &E::State` — a shared borrow. The
/// compiler prevents any behavior from mutating the live state, without requiring
/// any run-time checks or `unsafe` code.
///
/// The following snippet is intentionally rejected by the compiler:
///
/// ```compile_fail
/// # use herdingcats::{EngineSpec, Behavior, BehaviorResult};
/// # struct MySpec;
/// # impl EngineSpec for MySpec {
/// #     type State = Vec<u8>;
/// #     type Input = u8;
/// #     type Diff = u8;
/// #     type Trace = String;
/// #     type NonCommittedInfo = String;
/// #     type OrderKey = u32;
/// # }
/// struct BadBehavior;
///
/// impl Behavior<MySpec> for BadBehavior {
///     fn name(&self) -> &'static str { "bad" }
///     fn order_key(&self) -> u32 { 0 }
///     fn evaluate(
///         &self,
///         _input: &u8,
///         state: &Vec<u8>,   // shared borrow — mutation not allowed
///     ) -> BehaviorResult<u8, String> {
///         // ERROR: cannot assign to `*state` because it is behind a `&` reference
///         *state = Default::default();
///         BehaviorResult::Continue(vec![])
///     }
/// }
/// ```
pub trait Behavior<E: EngineSpec> {
    /// A unique, stable string identifier for this behavior.
    ///
    /// Used as the secondary sort key when two behaviors share the same
    /// [`order_key`](Behavior::order_key), guaranteeing deterministic ordering
    /// without relying on memory addresses or insertion order.
    fn name(&self) -> &'static str;

    /// Primary sort key for deterministic behavior ordering.
    ///
    /// Behaviors are evaluated in ascending order of `(order_key(), name())`.
    /// Lower values run first. Equal `order_key` values are broken by [`name`](Behavior::name).
    fn order_key(&self) -> E::OrderKey;

    /// Evaluate this behavior against the current input and state.
    ///
    /// Receives an immutable borrow of both `input` and `state`. The structural
    /// type prevents mutation; diffs are *returned*, not applied in-place.
    ///
    /// Returns [`BehaviorResult::Continue`] with any produced diffs, or
    /// [`BehaviorResult::Stop`] to halt dispatch immediately.
    fn evaluate(
        &self,
        input: &E::Input,
        state: &E::State,
    ) -> BehaviorResult<E::Diff, E::NonCommittedInfo>;
}

#[cfg(test)]
mod tests {
    use super::*;
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

    struct PassthroughBehavior;

    impl Behavior<TestSpec> for PassthroughBehavior {
        fn name(&self) -> &'static str {
            "passthrough"
        }

        fn order_key(&self) -> u32 {
            0
        }

        fn evaluate(&self, _input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
            BehaviorResult::Continue(vec![])
        }
    }

    struct StoppingBehavior;

    impl Behavior<TestSpec> for StoppingBehavior {
        fn name(&self) -> &'static str {
            "stopper"
        }

        fn order_key(&self) -> u32 {
            1
        }

        fn evaluate(&self, _input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
            BehaviorResult::Stop("halted".to_string())
        }
    }

    #[test]
    fn behavior_impl_compiles_without_warnings() {
        // Constructing and calling a Behavior<TestSpec> implementation — if it
        // compiles, all 3 required methods are provided with correct signatures.
        let b = PassthroughBehavior;
        assert_eq!(b.name(), "passthrough");
        assert_eq!(b.order_key(), 0u32);
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
        let r: BehaviorResult<u8, String> = BehaviorResult::Stop("reason".to_string());
        match r {
            BehaviorResult::Continue(_) => panic!("expected Stop"),
            BehaviorResult::Stop(msg) => assert_eq!(msg, "reason"),
        }
    }

    #[test]
    fn evaluate_receives_immutable_borrow_of_state() {
        // The compiler enforces this structurally. This test confirms the
        // signature compiles correctly with immutable state borrow.
        let b = PassthroughBehavior;
        let state = vec![1u8, 2u8];
        let result = b.evaluate(&42u8, &state);
        // state is still accessible (no move, no mutation)
        assert_eq!(state, vec![1u8, 2u8]);
        assert_eq!(result, BehaviorResult::Continue(vec![]));
    }

    #[test]
    fn behavior_is_dyn_safe() {
        // Box<dyn Behavior<TestSpec>> must be constructable — confirms object safety
        let b: Box<dyn Behavior<TestSpec>> = Box::new(PassthroughBehavior);
        assert_eq!(b.name(), "passthrough");

        let s: Box<dyn Behavior<TestSpec>> = Box::new(StoppingBehavior);
        assert_eq!(s.name(), "stopper");
    }
}
