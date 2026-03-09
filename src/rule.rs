// ============================================================
// Rule Trait
// ============================================================

use crate::operation::Operation;
use crate::transaction::Transaction;

/// An observer and modifier of in-flight transactions during event dispatch.
///
/// A `Rule` is the policy layer of this engine. Where an [`Operation`] defines
/// *what* changes state, a `Rule` defines *when* and *how* operations are
/// injected, modified, or blocked. Rules are parameterized over the state type
/// `S`, operation type `O`, event type `E`, and a priority type `P` (typically
/// `u8`). The engine calls all enabled rules in two phases for every dispatch:
/// `before()` runs in ascending priority order before operations are applied,
/// giving rules a chance to push additional ops or cancel the transaction;
/// `after()` runs in descending priority order after state has been mutated,
/// letting rules react to the new state (e.g., triggering follow-up effects).
/// A reader who understands both `Rule` and [`Operation`] understands the whole
/// engine model: events carry no logic — rules translate events into operations.
pub trait Rule<S, O, E, P>
where
    S: Clone,
    O: Operation<S>,
    P: Copy + Ord,
{
    /// A stable, unique identifier for this rule.
    ///
    /// The engine uses `id()` as the rule's key in its `enabled` set and
    /// `lifetimes` map. It must be a `'static str` — typically a module-scoped
    /// constant or string literal — and must not change between calls. Two
    /// rules with the same `id` will overwrite each other's lifecycle state.
    fn id(&self) -> &'static str;

    /// The priority that determines this rule's execution order.
    ///
    /// Lower values run first in `before()` (ascending order) and last in
    /// `after()` (descending order). Use priority to express rule dependencies:
    /// a validation rule with priority 0 runs before an effect rule with
    /// priority 10, so the effect rule can assume the transaction is valid.
    fn priority(&self) -> P;

    /// Called once per dispatch, before any operations in the transaction are
    /// applied to state.
    ///
    /// Use `before` to inspect the incoming `event`, push additional operations
    /// onto `tx.ops`, or set `tx.cancelled = true` to abort the entire
    /// transaction. All enabled rules run their `before` hooks in ascending
    /// priority order before any operations are applied. The default
    /// implementation is a no-op — override only what you need.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Operation, Rule, Transaction, RuleLifetime};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    ///
    /// impl Operation<i32> for CounterOp {
    ///     fn apply(&self, state: &mut i32) { *state += 1; }
    ///     fn undo(&self, state: &mut i32)  { *state -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8>  { vec![0] }
    /// }
    ///
    /// struct DoubleRule;
    ///
    /// impl Rule<i32, CounterOp, (), u8> for DoubleRule {
    ///     fn id(&self) -> &'static str { "double" }
    ///     fn priority(&self) -> u8 { 0 }
    ///     fn before(&self, _state: &i32, _event: &mut (), tx: &mut Transaction<CounterOp>) {
    ///         // For every Inc op already in the transaction, push a second Inc
    ///         let extra: Vec<CounterOp> = tx.ops.iter().map(|_| CounterOp::Inc).collect();
    ///         tx.ops.extend(extra);
    ///     }
    /// }
    /// ```
    fn before(
        &self,
        _state: &S,
        _event: &mut E,
        _tx: &mut Transaction<O>,
    ) {}

    /// Called once per dispatch, after all operations in the transaction have
    /// been applied to state.
    ///
    /// Use `after` to inspect the *updated* state and push additional operations
    /// for side effects — scoring events, unlock triggers, cascading reactions.
    /// All enabled rules run their `after` hooks in descending priority order
    /// after operations are applied. The default implementation is a no-op.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Operation, Rule, Transaction, RuleLifetime};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    ///
    /// impl Operation<i32> for CounterOp {
    ///     fn apply(&self, state: &mut i32) { *state += 1; }
    ///     fn undo(&self, state: &mut i32)  { *state -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8>  { vec![0] }
    /// }
    ///
    /// struct LogRule;
    ///
    /// impl Rule<i32, CounterOp, (), u8> for LogRule {
    ///     fn id(&self) -> &'static str { "log" }
    ///     fn priority(&self) -> u8 { 255 }
    ///     fn after(&self, state: &i32, _event: &(), _tx: &mut Transaction<CounterOp>) {
    ///         // After state is updated, react to the new value
    ///         let _ = *state; // e.g., could push a score op if state > threshold
    ///     }
    /// }
    /// ```
    fn after(
        &self,
        _state: &S,
        _event: &E,
        _tx: &mut Transaction<O>,
    ) {}
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::Operation;

    #[derive(Clone)]
    struct NoOp;

    impl Operation<()> for NoOp {
        fn apply(&self, _state: &mut ()) {}
        fn undo(&self, _state: &mut ()) {}
        fn hash_bytes(&self) -> Vec<u8> {
            vec![]
        }
    }

    struct TestRule;

    impl Rule<(), NoOp, (), u8> for TestRule {
        fn id(&self) -> &'static str {
            "test"
        }
        fn priority(&self) -> u8 {
            0
        }
    }

    #[test]
    fn rule_id_and_priority() {
        let rule = TestRule;
        assert_eq!(rule.id(), "test");
        assert_eq!(rule.priority(), 0u8);
    }
}
