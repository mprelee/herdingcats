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

    /// Whether this behavior participates in the current dispatch.
    ///
    /// Returns `true` by default. When `false`, the engine skips `before()`
    /// and `after()` for this behavior — the behavior is "sleeping" but not
    /// removed. Sleeping behaviors still receive `on_dispatch()` and `on_undo()`
    /// calls so they can track dispatch history and self-reactivate (e.g.,
    /// a behavior that wakes after N turns).
    ///
    /// # Examples
    ///
    /// ```
    /// # use herdingcats::{Mutation, Behavior, Action, Engine};
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    ///
    /// impl Mutation<i32> for CounterOp {
    ///     fn apply(&self, state: &mut i32) { *state += 1; }
    ///     fn undo(&self, state: &mut i32)  { *state -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8>  { vec![0] }
    /// }
    ///
    /// struct SleepingRule;
    ///
    /// impl Behavior<i32, CounterOp, (), u8> for SleepingRule {
    ///     fn id(&self) -> &'static str { "sleeping" }
    ///     fn priority(&self) -> u8    { 0 }
    ///     fn is_active(&self) -> bool { false }
    ///     fn before(&self, _state: &i32, _event: &mut (), tx: &mut Action<CounterOp>) {
    ///         // This would inject an Inc, but is_active() is false so it is skipped
    ///         tx.mutations.push(CounterOp::Inc);
    ///     }
    /// }
    ///
    /// let mut engine = Engine::new(0i32);
    /// engine.add_behavior(SleepingRule);
    ///
    /// // Dispatch with an empty action — SleepingRule.before() is never called
    /// engine.dispatch((), Action::<CounterOp>::new());
    ///
    /// // State is still 0 because the sleeping behavior's before() was skipped
    /// assert_eq!(engine.read(), 0);
    /// ```
    fn is_active(&self) -> bool {
        true
    }

    /// Called after each committed action (including redo) on ALL behaviors,
    /// regardless of `is_active()`.
    ///
    /// Use this to update behavior-internal state in response to a commit
    /// (e.g., decrement a charge counter, toggle an activation flag).
    /// This hook fires in a separate pass after state mutations are applied,
    /// avoiding borrow conflicts with the `&self` hooks `before` and `after`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use herdingcats::{Mutation, Behavior, Action, Engine};
    /// # use std::rc::Rc;
    /// # use std::cell::Cell;
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    ///
    /// impl Mutation<i32> for CounterOp {
    ///     fn apply(&self, state: &mut i32) { *state += 1; }
    ///     fn undo(&self, state: &mut i32)  { *state -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8>  { vec![0] }
    /// }
    ///
    /// struct DispatchCounter { count: Rc<Cell<u32>> }
    ///
    /// impl Behavior<i32, CounterOp, (), u8> for DispatchCounter {
    ///     fn id(&self) -> &'static str { "dispatch_counter" }
    ///     fn priority(&self) -> u8    { 0 }
    ///     fn on_dispatch(&mut self) {
    ///         self.count.set(self.count.get() + 1);
    ///     }
    /// }
    ///
    /// let counter = Rc::new(Cell::new(0u32));
    /// let mut engine = Engine::new(0i32);
    /// engine.add_behavior(DispatchCounter { count: Rc::clone(&counter) });
    ///
    /// let mut tx1 = Action::new();
    /// tx1.mutations.push(CounterOp::Inc);
    /// engine.dispatch((), tx1);
    ///
    /// let mut tx2 = Action::new();
    /// tx2.mutations.push(CounterOp::Inc);
    /// engine.dispatch((), tx2);
    ///
    /// // on_dispatch was called once per committed action
    /// assert_eq!(counter.get(), 2);
    /// ```
    fn on_dispatch(&mut self) {}

    /// Called after each undo on ALL behaviors, regardless of `is_active()`.
    ///
    /// Use this to reverse behavior-internal state changes made in `on_dispatch`.
    /// Symmetry: `on_undo` reverses what `on_dispatch` advanced.
    ///
    /// # Examples
    ///
    /// ```
    /// # use herdingcats::{Mutation, Behavior, Action, Engine};
    /// # use std::rc::Rc;
    /// # use std::cell::Cell;
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    ///
    /// impl Mutation<i32> for CounterOp {
    ///     fn apply(&self, state: &mut i32) { *state += 1; }
    ///     fn undo(&self, state: &mut i32)  { *state -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8>  { vec![0] }
    /// }
    ///
    /// struct DispatchCounter { count: Rc<Cell<u32>> }
    ///
    /// impl Behavior<i32, CounterOp, (), u8> for DispatchCounter {
    ///     fn id(&self) -> &'static str { "dispatch_counter" }
    ///     fn priority(&self) -> u8    { 0 }
    ///     fn on_dispatch(&mut self) { self.count.set(self.count.get() + 1); }
    ///     fn on_undo(&mut self)     { self.count.set(self.count.get() - 1); }
    /// }
    ///
    /// let counter = Rc::new(Cell::new(0u32));
    /// let mut engine = Engine::new(0i32);
    /// engine.add_behavior(DispatchCounter { count: Rc::clone(&counter) });
    ///
    /// let mut tx = Action::new();
    /// tx.mutations.push(CounterOp::Inc);
    /// engine.dispatch((), tx);
    /// assert_eq!(counter.get(), 1); // on_dispatch incremented
    ///
    /// engine.undo();
    /// assert_eq!(counter.get(), 0); // on_undo decremented back to zero
    /// ```
    fn on_undo(&mut self) {}
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

    #[test]
    fn is_active_default_true() {
        let rule = TestRule;
        assert!(rule.is_active());
    }

    #[test]
    fn on_dispatch_default_noop() {
        let mut rule = TestRule;
        rule.on_dispatch(); // must compile and not panic
    }

    #[test]
    fn on_undo_default_noop() {
        let mut rule = TestRule;
        rule.on_undo(); // must compile and not panic
    }

    struct CountingBehavior { dispatches: u32 }

    impl Behavior<(), NoOp, (), u8> for CountingBehavior {
        fn id(&self) -> &'static str { "counter" }
        fn priority(&self) -> u8 { 0 }
        fn is_active(&self) -> bool { self.dispatches < 3 }
        fn on_dispatch(&mut self) { self.dispatches += 1; }
        fn on_undo(&mut self) { if self.dispatches > 0 { self.dispatches -= 1; } }
    }

    #[test]
    fn stateful_behavior_lifecycle() {
        let mut b = CountingBehavior { dispatches: 0 };
        assert!(b.is_active());
        b.on_dispatch();
        b.on_dispatch();
        b.on_dispatch();
        assert!(!b.is_active()); // deactivated after 3 dispatches
        b.on_undo();
        assert!(b.is_active()); // reactivated after undo
    }
}
