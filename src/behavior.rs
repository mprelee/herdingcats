// ============================================================
// Behavior Trait
// ============================================================

use crate::action::Action;
use crate::mutation::Mutation;

/// An observer and modifier of in-flight actions during event dispatch.
///
/// A `Behavior` is the policy layer of this engine. Where a [`Mutation`] defines
/// *what* changes state, a `Behavior` defines *when* and *how* mutations are
/// injected, modified, or blocked. Behaviors are parameterized over the state type
/// `S`, mutation type `M`, input type `I`, and a priority type `P` (typically
/// `u8`). The engine calls all enabled behaviors in two phases for every dispatch:
/// `before()` runs in ascending priority order before mutations are applied,
/// giving behaviors a chance to push additional mutations or cancel the action;
/// `after()` runs in descending priority order after state has been mutated,
/// letting behaviors react to the new state (e.g., triggering follow-up effects).
/// A reader who understands both `Behavior` and [`Mutation`] understands the whole
/// engine model: events carry no logic — behaviors translate events into mutations.
pub trait Behavior<S, M, I, P>
where
    S: Clone,
    M: Mutation<S>,
    P: Copy + Ord,
{
    /// A stable, unique identifier for this behavior.
    ///
    /// The engine uses `id()` as the behavior's key in its `enabled` set and
    /// `lifetimes` map. It must be a `'static str` — typically a module-scoped
    /// constant or string literal — and must not change between calls. Two
    /// behaviors with the same `id` will overwrite each other's lifecycle state.
    fn id(&self) -> &'static str;

    /// The priority that determines this behavior's execution order.
    ///
    /// Lower values run first in `before()` (ascending order) and last in
    /// `after()` (descending order). Use priority to express behavior dependencies:
    /// a validation behavior with priority 0 runs before an effect behavior with
    /// priority 10, so the effect behavior can assume the action is valid.
    fn priority(&self) -> P;

    /// Called once per dispatch, before any mutations in the action are
    /// applied to state.
    ///
    /// Use `before` to inspect the incoming `event`, push additional mutations
    /// onto `tx.mutations`, or set `tx.cancelled = true` to abort the entire
    /// action. All enabled behaviors run their `before` hooks in ascending
    /// priority order before any mutations are applied. The default
    /// implementation is a no-op — override only what you need.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Mutation, Behavior, Action};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    ///
    /// impl Mutation<i32> for CounterOp {
    ///     fn apply(&self, state: &mut i32) { *state += 1; }
    ///     fn undo(&self, state: &mut i32)  { *state -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8>  { vec![0] }
    /// }
    ///
    /// struct DoubleRule;
    ///
    /// impl Behavior<i32, CounterOp, (), u8> for DoubleRule {
    ///     fn id(&self) -> &'static str { "double" }
    ///     fn priority(&self) -> u8 { 0 }
    ///     fn before(&self, _state: &i32, _event: &mut (), tx: &mut Action<CounterOp>) {
    ///         // For every Inc mutation already in the action, push a second Inc
    ///         let extra: Vec<CounterOp> = tx.mutations.iter().map(|_| CounterOp::Inc).collect();
    ///         tx.mutations.extend(extra);
    ///     }
    /// }
    /// ```
    fn before(&self, _state: &S, _event: &mut I, _tx: &mut Action<M>) {}

    /// Called once per dispatch, after all mutations in the action have
    /// been applied to state.
    ///
    /// Use `after` to inspect the *updated* state and push additional mutations
    /// for side effects — scoring events, unlock triggers, cascading reactions.
    /// All enabled behaviors run their `after` hooks in descending priority order
    /// after mutations are applied. The default implementation is a no-op.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Mutation, Behavior, Action};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    ///
    /// impl Mutation<i32> for CounterOp {
    ///     fn apply(&self, state: &mut i32) { *state += 1; }
    ///     fn undo(&self, state: &mut i32)  { *state -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8>  { vec![0] }
    /// }
    ///
    /// struct LogRule;
    ///
    /// impl Behavior<i32, CounterOp, (), u8> for LogRule {
    ///     fn id(&self) -> &'static str { "log" }
    ///     fn priority(&self) -> u8 { 255 }
    ///     fn after(&self, state: &i32, _event: &(), _tx: &mut Action<CounterOp>) {
    ///         // After state is updated, react to the new value
    ///         let _ = *state; // e.g., could push a score mutation if state > threshold
    ///     }
    /// }
    /// ```
    fn after(&self, _state: &S, _event: &I, _tx: &mut Action<M>) {}
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutation::Mutation;

    #[derive(Clone)]
    struct NoOp;

    impl Mutation<()> for NoOp {
        fn apply(&self, _state: &mut ()) {}
        fn undo(&self, _state: &mut ()) {}
        fn hash_bytes(&self) -> Vec<u8> {
            vec![]
        }
    }

    struct TestRule;

    impl Behavior<(), NoOp, (), u8> for TestRule {
        fn id(&self) -> &'static str {
            "test"
        }
        fn priority(&self) -> u8 {
            0
        }
    }

    #[test]
    fn behavior_id_and_priority() {
        let rule = TestRule;
        assert_eq!(rule.id(), "test");
        assert_eq!(rule.priority(), 0u8);
    }
}
