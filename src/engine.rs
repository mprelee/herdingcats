// ============================================================
// Commit Frame (private)
// ============================================================

use std::collections::{HashMap, HashSet};

use crate::hash::{FNV_OFFSET, FNV_PRIME, fnv1a_hash};
use crate::operation::Operation;
use crate::rule::Rule;
use crate::transaction::{RuleLifetime, Transaction};

// A single entry on the undo stack, capturing everything needed to reverse
// one committed transaction and restore the engine to its prior state.
//
// CommitFrame is the key to undo correctness: rather than storing full state
// snapshots (expensive for large game states), the engine stores just enough
// to *reverse* the commit. Fields:
#[derive(Clone)]
struct CommitFrame<S, O> {
    // The transaction that was committed. Used in reverse (ops.iter().rev())
    // during undo to call op.undo() on each operation in reverse order.
    tx: Transaction<O>,

    // The replay_hash value *before* this transaction was applied. Restored
    // directly on undo so the fingerprint matches pre-commit state without
    // replaying the full history.
    state_hash_before: u64,

    // The replay_hash value *after* this transaction was applied. Restored
    // directly on redo so re-applying the frame doesn't re-hash the ops.
    state_hash_after: u64,

    // Snapshot of rule lifetimes at the moment this transaction was committed.
    // Restored on undo so turn-limited and trigger-counted rules rewind to
    // their pre-commit counts — the rule lifecycle mirrors the game state.
    lifetime_snapshot: HashMap<&'static str, RuleLifetime>,

    // Snapshot of the enabled rule set at the moment this transaction was
    // committed. Restored on undo so rules that expired during this commit
    // are re-enabled, and rules that were disabled before are not re-enabled.
    enabled_snapshot: HashSet<&'static str>,

    // PhantomData marker for the state type S. CommitFrame<S, O> does not own
    // an S directly (state is held by the Engine), but the type parameter is
    // needed for variance: covariant in S so CommitFrame<&'long T> is usable
    // where CommitFrame<&'short T> is expected.
    _marker: std::marker::PhantomData<S>,
}

// ============================================================
// Engine
// ============================================================

/// The runtime that manages game state, rules, and commit history.
///
/// `Engine<S, O, E, P>` is the central coordinator: it holds the current
/// state `S`, a sorted list of [`Rule`]s, and an undo/redo stack of commit
/// frames. To advance state, callers construct a [`Transaction`] and pass
/// it — along with an event value — to [`dispatch`](Engine::dispatch). The
/// engine runs all enabled rules in priority order, applies the resulting
/// operations, and records the frame for undo. Direct mutation of
/// [`state`](Engine::state) is possible but bypasses the rule system; use
/// [`write`](Engine::write) for intentional resets.
pub struct Engine<S, O, E, P>
where
    S: Clone,
    O: Operation<S>,
    P: Copy + Ord,
{
    /// The current committed game state.
    ///
    /// Readable directly for ergonomic access in non-rule contexts. Prefer
    /// [`read`](Engine::read) when you want a clone rather than a borrow.
    /// **Direct mutation of this field bypasses the rule system and undo
    /// stack** — use [`write`](Engine::write) for intentional state resets
    /// (e.g., loading a saved game), which also clears both stacks and
    /// resets the `replay_hash`.
    pub state: S,

    undo_stack: Vec<CommitFrame<S, O>>,
    redo_stack: Vec<CommitFrame<S, O>>,

    rules: Vec<Box<dyn Rule<S, O, E, P>>>,
    enabled: HashSet<&'static str>,
    lifetimes: HashMap<&'static str, RuleLifetime>,

    replay_hash: u64,
}

impl<S, O, E, P> Engine<S, O, E, P>
where
    S: Clone,
    O: Operation<S>,
    P: Copy + Ord,
{
    /// Create a new engine with the given initial state and no rules.
    ///
    /// The undo and redo stacks start empty. `replay_hash` is initialized to
    /// `FNV_OFFSET` (the hash of an empty sequence). Add rules with
    /// [`add_rule`](Engine::add_rule) before dispatching events.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Engine, Operation};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Operation<i32> for CounterOp {
    ///     fn apply(&self, s: &mut i32) { *s += 1; }
    ///     fn undo(&self, s: &mut i32)  { *s -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8> { vec![1] }
    /// }
    ///
    /// let engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
    /// assert_eq!(engine.state, 0);
    /// ```
    pub fn new(state: S) -> Self {
        Self {
            state,
            undo_stack: vec![],
            redo_stack: vec![],
            rules: vec![],
            enabled: HashSet::new(),
            lifetimes: HashMap::new(),
            replay_hash: FNV_OFFSET,
        }
    }

    /// Return the current replay hash — a running fingerprint over all
    /// committed, deterministic operations.
    ///
    /// The replay hash is updated on every [`dispatch`](Engine::dispatch) call
    /// where the transaction is both `irreversible` and `deterministic`. Two
    /// engine instances that have processed the same sequence of deterministic
    /// operations will have identical replay hashes, regardless of any
    /// non-deterministic or irreversible commits in between.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Engine, Operation, Transaction, RuleLifetime};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Operation<i32> for CounterOp {
    ///     fn apply(&self, s: &mut i32) { *s += 1; }
    ///     fn undo(&self, s: &mut i32)  { *s -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8> { vec![1] }
    /// }
    ///
    /// let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0);
    /// let hash_before = engine.replay_hash();
    ///
    /// let mut tx = Transaction::new();
    /// tx.ops.push(CounterOp::Inc);
    /// engine.dispatch((), tx);
    ///
    /// assert_ne!(engine.replay_hash(), hash_before);
    /// ```
    pub fn replay_hash(&self) -> u64 {
        self.replay_hash
    }

    /// Register a rule with this engine and assign it a lifetime.
    ///
    /// The rule is inserted into the sorted rule list (sorted by
    /// [`priority`](crate::Rule::priority) ascending). The rule starts
    /// enabled; the [`RuleLifetime`] controls when it is automatically
    /// disabled. If a rule with the same `id` is added twice, both rule
    /// objects remain in the list, but they share a lifetime entry — the
    /// second `add_rule` overwrites the first's lifetime.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Engine, Operation, Rule, Transaction, RuleLifetime};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Operation<i32> for CounterOp {
    ///     fn apply(&self, s: &mut i32) { *s += 1; }
    ///     fn undo(&self, s: &mut i32)  { *s -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8> { vec![1] }
    /// }
    ///
    /// struct NoRule;
    /// impl Rule<i32, CounterOp, (), u8> for NoRule {
    ///     fn id(&self) -> &'static str { "no_rule" }
    ///     fn priority(&self) -> u8 { 0 }
    /// }
    ///
    /// let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0);
    /// engine.add_rule(NoRule, RuleLifetime::Permanent);
    /// ```
    pub fn add_rule<R>(&mut self, rule: R, lifetime: RuleLifetime)
    where
        R: Rule<S, O, E, P> + 'static,
    {
        let id = rule.id();
        self.rules.push(Box::new(rule));

        self.rules.sort_by_key(|r| r.priority());

        self.enabled.insert(id);
        self.lifetimes.insert(id, lifetime);
    }

    //
    // --------------------------------------------------------
    // Preview Dispatch
    // --------------------------------------------------------
    //

    /// Run the full dispatch pipeline on `event` and `tx` without committing
    /// any changes to state, replay hash, or rule lifetimes.
    ///
    /// `dispatch_preview` is a dry run: all enabled rules fire their `before`
    /// and `after` hooks, operations are applied, but everything is rolled back
    /// at the end. This is useful for AI look-ahead, UI preview of pending
    /// moves, or testing rule interactions without side effects.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Engine, Operation, Rule, Transaction, RuleLifetime};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Operation<i32> for CounterOp {
    ///     fn apply(&self, s: &mut i32) { *s += 1; }
    ///     fn undo(&self, s: &mut i32)  { *s -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8> { vec![1] }
    /// }
    ///
    /// struct NoRule;
    /// impl Rule<i32, CounterOp, (), u8> for NoRule {
    ///     fn id(&self) -> &'static str { "no_rule" }
    ///     fn priority(&self) -> u8 { 0 }
    /// }
    ///
    /// let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0);
    /// engine.add_rule(NoRule, RuleLifetime::Permanent);
    ///
    /// let mut tx = Transaction::new();
    /// tx.ops.push(CounterOp::Inc);
    /// engine.dispatch_preview((), tx);
    ///
    /// // State is unchanged after preview
    /// assert_eq!(engine.state, 0);
    /// ```
    pub fn dispatch_preview(&mut self, mut event: E, mut tx: Transaction<O>) {
        let state_snapshot = self.state.clone();
        let lifetime_snapshot = self.lifetimes.clone();
        let enabled_snapshot = self.enabled.clone();
        let hash_snapshot = self.replay_hash;

        for rule in &self.rules {
            if self.enabled.contains(rule.id()) {
                rule.before(&self.state, &mut event, &mut tx);
            }
        }

        if !tx.cancelled {
            for op in &tx.ops {
                op.apply(&mut self.state);
            }
        }

        for rule in self.rules.iter().rev() {
            if self.enabled.contains(rule.id()) {
                rule.after(&self.state, &event, &mut tx);
            }
        }

        self.state = state_snapshot;
        self.lifetimes = lifetime_snapshot;
        self.enabled = enabled_snapshot;
        self.replay_hash = hash_snapshot;
    }

    //
    // --------------------------------------------------------
    // Commit Dispatch
    // --------------------------------------------------------
    //

    /// Dispatch `event` through all enabled rules, apply the resulting
    /// operations, and — if the transaction is reversible — push a
    /// `CommitFrame` onto the undo stack.
    ///
    /// Rules fire in ascending priority order during `before()`, then
    /// descending order during `after()`. If any rule sets `tx.cancelled =
    /// true`, the operations are not applied and no frame is committed.
    /// Rule lifetimes ([`Turns`](RuleLifetime::Turns),
    /// [`Triggers`](RuleLifetime::Triggers)) are decremented here.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Engine, Operation, Rule, Transaction, RuleLifetime};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Operation<i32> for CounterOp {
    ///     fn apply(&self, s: &mut i32) { *s += 1; }
    ///     fn undo(&self, s: &mut i32)  { *s -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8> { vec![1] }
    /// }
    ///
    /// struct NoRule;
    /// impl Rule<i32, CounterOp, (), u8> for NoRule {
    ///     fn id(&self) -> &'static str { "no_rule" }
    ///     fn priority(&self) -> u8 { 0 }
    /// }
    ///
    /// let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0);
    /// engine.add_rule(NoRule, RuleLifetime::Permanent);
    ///
    /// let mut tx = Transaction::new();
    /// tx.ops.push(CounterOp::Inc);
    /// engine.dispatch((), tx);
    ///
    /// assert_eq!(engine.state, 1);
    /// ```
    pub fn dispatch(&mut self, mut event: E, mut tx: Transaction<O>) {
        let hash_before = self.replay_hash;
        let lifetime_snapshot = self.lifetimes.clone();
        let enabled_snapshot = self.enabled.clone();

        for rule in &self.rules {
            if self.enabled.contains(rule.id()) {
                rule.before(&self.state, &mut event, &mut tx);

                if let Some(RuleLifetime::Triggers(n)) = self.lifetimes.get_mut(rule.id())
                    && *n > 0
                {
                    *n -= 1;
                    if *n == 0 {
                        self.enabled.remove(rule.id());
                    }
                }
            }
        }

        if !tx.cancelled {
            for op in &tx.ops {
                op.apply(&mut self.state);
            }
        }

        for rule in self.rules.iter().rev() {
            if self.enabled.contains(rule.id()) {
                rule.after(&self.state, &event, &mut tx);
            }
        }

        for (id, lifetime) in self.lifetimes.iter_mut() {
            if let RuleLifetime::Turns(n) = lifetime
                && *n > 0
            {
                *n -= 1;
                if *n == 0 {
                    self.enabled.remove(id);
                }
            }
        }

        if tx.irreversible && !tx.cancelled {
            for op in &tx.ops {
                let h = fnv1a_hash(&op.hash_bytes());
                self.replay_hash ^= h;
                self.replay_hash = self.replay_hash.wrapping_mul(FNV_PRIME);
            }

            let hash_after = self.replay_hash;

            self.undo_stack.push(CommitFrame {
                tx,
                state_hash_before: hash_before,
                state_hash_after: hash_after,
                lifetime_snapshot,
                enabled_snapshot,
                _marker: std::marker::PhantomData,
            });

            self.redo_stack.clear();
        }
    }

    /// Reverse the most recent reversible commit, restoring state and rule
    /// lifetimes to their values before that commit.
    ///
    /// If the undo stack is empty, this is a no-op. The undone frame is moved
    /// to the redo stack so [`redo`](Engine::redo) can re-apply it.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Engine, Operation, Rule, Transaction, RuleLifetime};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Operation<i32> for CounterOp {
    ///     fn apply(&self, s: &mut i32) { *s += 1; }
    ///     fn undo(&self, s: &mut i32)  { *s -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8> { vec![1] }
    /// }
    ///
    /// struct NoRule;
    /// impl Rule<i32, CounterOp, (), u8> for NoRule {
    ///     fn id(&self) -> &'static str { "no_rule" }
    ///     fn priority(&self) -> u8 { 0 }
    /// }
    ///
    /// let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0);
    /// engine.add_rule(NoRule, RuleLifetime::Permanent);
    ///
    /// let mut tx = Transaction::new();
    /// tx.ops.push(CounterOp::Inc);
    /// engine.dispatch((), tx);
    /// assert_eq!(engine.state, 1);
    ///
    /// engine.undo();
    /// assert_eq!(engine.state, 0);
    /// ```
    pub fn undo(&mut self) {
        if let Some(frame) = self.undo_stack.pop() {
            for op in frame.tx.ops.iter().rev() {
                op.undo(&mut self.state);
            }

            self.replay_hash = frame.state_hash_before;
            self.lifetimes = frame.lifetime_snapshot.clone();
            self.enabled = frame.enabled_snapshot.clone();

            self.redo_stack.push(frame);
        }
    }

    /// Re-apply the most recently undone commit, advancing state forward again.
    ///
    /// If the redo stack is empty, this is a no-op. The redone frame is moved
    /// back to the undo stack. Note that calling [`dispatch`](Engine::dispatch)
    /// clears the redo stack — once you commit a new action, the redo history
    /// for the previous branch is discarded.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Engine, Operation, Rule, Transaction, RuleLifetime};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Operation<i32> for CounterOp {
    ///     fn apply(&self, s: &mut i32) { *s += 1; }
    ///     fn undo(&self, s: &mut i32)  { *s -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8> { vec![1] }
    /// }
    ///
    /// struct NoRule;
    /// impl Rule<i32, CounterOp, (), u8> for NoRule {
    ///     fn id(&self) -> &'static str { "no_rule" }
    ///     fn priority(&self) -> u8 { 0 }
    /// }
    ///
    /// let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0);
    /// engine.add_rule(NoRule, RuleLifetime::Permanent);
    ///
    /// let mut tx = Transaction::new();
    /// tx.ops.push(CounterOp::Inc);
    /// engine.dispatch((), tx);
    /// engine.undo();
    /// assert_eq!(engine.state, 0);
    ///
    /// engine.redo();
    /// assert_eq!(engine.state, 1);
    /// ```
    pub fn redo(&mut self) {
        if let Some(frame) = self.redo_stack.pop() {
            for op in &frame.tx.ops {
                op.apply(&mut self.state);
            }

            self.replay_hash = frame.state_hash_after;
            self.lifetimes = frame.lifetime_snapshot.clone();
            self.enabled = frame.enabled_snapshot.clone();

            self.undo_stack.push(frame);
        }
    }

    /// Return a clone of the current state.
    ///
    /// Use `read` when you need an owned snapshot rather than borrowing
    /// `engine.state` directly. This is the idiomatic way to hand state to
    /// code that needs ownership (e.g., serialization, AI evaluation).
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Engine, Operation};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Operation<i32> for CounterOp {
    ///     fn apply(&self, s: &mut i32) { *s += 1; }
    ///     fn undo(&self, s: &mut i32)  { *s -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8> { vec![1] }
    /// }
    ///
    /// let engine: Engine<i32, CounterOp, (), u8> = Engine::new(42i32);
    /// let snapshot = engine.read();
    /// assert_eq!(snapshot, 42);
    /// ```
    pub fn read(&self) -> S {
        self.state.clone()
    }

    /// Replace the engine's state with `snapshot` and reset all history.
    ///
    /// `write` clears both the undo and redo stacks and resets `replay_hash`
    /// to its initial value. Use it for intentional state resets — loading a
    /// saved game, starting a new round — where you want to discard all prior
    /// history. Unlike direct mutation of `engine.state`, `write` guarantees
    /// the stacks and hash are coherent with the new state.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Engine, Operation, Rule, Transaction, RuleLifetime};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Operation<i32> for CounterOp {
    ///     fn apply(&self, s: &mut i32) { *s += 1; }
    ///     fn undo(&self, s: &mut i32)  { *s -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8> { vec![1] }
    /// }
    ///
    /// struct NoRule;
    /// impl Rule<i32, CounterOp, (), u8> for NoRule {
    ///     fn id(&self) -> &'static str { "no_rule" }
    ///     fn priority(&self) -> u8 { 0 }
    /// }
    ///
    /// let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0);
    /// engine.add_rule(NoRule, RuleLifetime::Permanent);
    ///
    /// let mut tx = Transaction::new();
    /// tx.ops.push(CounterOp::Inc);
    /// engine.dispatch((), tx);
    /// assert_eq!(engine.state, 1);
    ///
    /// // Reset to a fresh state — undo history is cleared
    /// engine.write(100);
    /// assert_eq!(engine.state, 100);
    /// let hash_after_write = engine.replay_hash();
    ///
    /// // replay_hash is back to its initial value
    /// let fresh_engine: Engine<i32, CounterOp, (), u8> = Engine::new(0);
    /// assert_eq!(hash_after_write, fresh_engine.replay_hash());
    /// ```
    pub fn write(&mut self, snapshot: S) {
        self.state = snapshot;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.replay_hash = FNV_OFFSET;
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{RuleLifetime, Transaction};

    // --------------------------------------------------------
    // CounterOp fixture
    // --------------------------------------------------------

    #[derive(Clone, Debug, PartialEq)]
    enum CounterOp {
        Inc,
        Dec,
        Reset { prior: i32 },
    }

    impl Operation<i32> for CounterOp {
        fn apply(&self, state: &mut i32) {
            match self {
                CounterOp::Inc => *state += 1,
                CounterOp::Dec => *state -= 1,
                CounterOp::Reset { .. } => *state = 0,
            }
        }
        fn undo(&self, state: &mut i32) {
            match self {
                CounterOp::Inc => *state -= 1,
                CounterOp::Dec => *state += 1,
                CounterOp::Reset { prior } => *state = *prior,
            }
        }
        fn hash_bytes(&self) -> Vec<u8> {
            match self {
                CounterOp::Inc => vec![0],
                CounterOp::Dec => vec![1],
                CounterOp::Reset { prior } => {
                    let mut v = vec![2];
                    v.extend_from_slice(&prior.to_le_bytes());
                    v
                }
            }
        }
    }

    struct NoRule;
    impl Rule<i32, CounterOp, (), u8> for NoRule {
        fn id(&self) -> &'static str {
            "no_rule"
        }
        fn priority(&self) -> u8 {
            0
        }
    }

    // --------------------------------------------------------
    // apply+undo roundtrip tests (TEST-03)
    // --------------------------------------------------------

    #[test]
    fn apply_undo_inc() {
        let mut state = 0i32;
        CounterOp::Inc.apply(&mut state);
        assert_eq!(state, 1);
        CounterOp::Inc.undo(&mut state);
        assert_eq!(state, 0);
    }

    #[test]
    fn apply_undo_dec() {
        let mut state = 0i32;
        CounterOp::Dec.apply(&mut state);
        assert_eq!(state, -1);
        CounterOp::Dec.undo(&mut state);
        assert_eq!(state, 0);
    }

    #[test]
    fn apply_undo_reset() {
        let mut state = 5i32;
        let op = CounterOp::Reset { prior: 5 };
        op.apply(&mut state);
        assert_eq!(state, 0);
        op.undo(&mut state);
        assert_eq!(state, 5);
    }

    // --------------------------------------------------------
    // hash_bytes determinism and non-empty (TEST-04)
    // --------------------------------------------------------

    #[test]
    fn hash_bytes_nonempty() {
        assert!(!CounterOp::Inc.hash_bytes().is_empty());
        assert!(!CounterOp::Dec.hash_bytes().is_empty());
        assert!(!CounterOp::Reset { prior: 0 }.hash_bytes().is_empty());
    }

    #[test]
    fn hash_bytes_determinism() {
        assert_eq!(CounterOp::Inc.hash_bytes(), CounterOp::Inc.hash_bytes());
        assert_eq!(CounterOp::Dec.hash_bytes(), CounterOp::Dec.hash_bytes());
        assert_eq!(
            CounterOp::Reset { prior: 7 }.hash_bytes(),
            CounterOp::Reset { prior: 7 }.hash_bytes()
        );
    }

    #[test]
    fn hash_bytes_variant_uniqueness() {
        assert_ne!(CounterOp::Inc.hash_bytes(), CounterOp::Dec.hash_bytes());
        assert_ne!(
            CounterOp::Inc.hash_bytes(),
            CounterOp::Reset { prior: 0 }.hash_bytes()
        );
    }

    // --------------------------------------------------------
    // Engine integration smoke test
    // --------------------------------------------------------

    #[test]
    fn engine_dispatch_undo() {
        let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine.add_rule(NoRule, RuleLifetime::Permanent);

        let mut tx = Transaction::new();
        tx.ops.push(CounterOp::Inc);
        engine.dispatch((), tx);
        assert_eq!(engine.read(), 1);

        engine.undo();
        assert_eq!(engine.read(), 0);
    }
}

// ============================================================
// Property Tests
// ============================================================

#[cfg(test)]
mod props {
    use super::*;
    use crate::transaction::{RuleLifetime, Transaction};
    use proptest::prelude::*;
    use std::cell::Cell;
    use std::rc::Rc;

    // --------------------------------------------------------
    // CounterOp fixture
    // --------------------------------------------------------

    #[derive(Clone, Debug)]
    enum CounterOp {
        Inc,
        Dec,
    }

    impl Operation<i32> for CounterOp {
        fn apply(&self, state: &mut i32) {
            match self {
                CounterOp::Inc => *state += 1,
                CounterOp::Dec => *state -= 1,
            }
        }
        fn undo(&self, state: &mut i32) {
            match self {
                CounterOp::Inc => *state -= 1,
                CounterOp::Dec => *state += 1,
            }
        }
        fn hash_bytes(&self) -> Vec<u8> {
            match self {
                CounterOp::Inc => vec![0],
                CounterOp::Dec => vec![1],
            }
        }
    }

    // --------------------------------------------------------
    // NoRule fixture
    // --------------------------------------------------------

    struct NoRule;

    impl Rule<i32, CounterOp, (), u8> for NoRule {
        fn id(&self) -> &'static str {
            "no_rule"
        }
        fn priority(&self) -> u8 {
            0
        }
    }

    // --------------------------------------------------------
    // CountingRule fixture (for PROP-03)
    // --------------------------------------------------------

    struct CountingRule {
        trigger_count: Rc<Cell<u32>>,
    }

    impl Rule<i32, CounterOp, (), u8> for CountingRule {
        fn id(&self) -> &'static str {
            "counting_rule"
        }
        fn priority(&self) -> u8 {
            0
        }
        fn before(&self, _state: &i32, _event: &mut (), _tx: &mut Transaction<CounterOp>) {
            self.trigger_count.set(self.trigger_count.get() + 1);
        }
    }

    // --------------------------------------------------------
    // Shared strategy
    // --------------------------------------------------------

    fn op_sequence_strategy() -> impl Strategy<Value = Vec<CounterOp>> {
        prop::collection::vec(
            prop_oneof![Just(CounterOp::Inc), Just(CounterOp::Dec)],
            0..=20,
        )
    }

    // --------------------------------------------------------
    // PROP-01: undo roundtrip restores both state and replay_hash
    // --------------------------------------------------------

    proptest! {
        #[test]
        fn prop_01_undo_roundtrip(ops in op_sequence_strategy()) {
            let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
            engine.add_rule(NoRule, RuleLifetime::Permanent);

            let state_before = engine.read();
            let hash_before = engine.replay_hash();

            for op in &ops {
                let mut tx = Transaction::new();
                tx.ops.push(op.clone());
                engine.dispatch((), tx);
            }

            for _ in &ops {
                engine.undo();
            }

            prop_assert_eq!(engine.read(), state_before);
            prop_assert_eq!(engine.replay_hash(), hash_before);
        }
    }

    // --------------------------------------------------------
    // PROP-02: dispatch_preview leaves state and replay_hash unchanged
    // --------------------------------------------------------

    proptest! {
        #[test]
        fn prop_02_preview_isolation(ops in op_sequence_strategy()) {
            let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
            engine.add_rule(NoRule, RuleLifetime::Permanent);

            // Establish a non-trivial pre-preview state
            for op in &ops {
                let mut tx = Transaction::new();
                tx.ops.push(op.clone());
                engine.dispatch((), tx);
            }

            let state_before = engine.read();
            let hash_before = engine.replay_hash();

            // Build and run a preview transaction with the same ops
            let mut preview_tx = Transaction::new();
            for op in &ops {
                preview_tx.ops.push(op.clone());
            }
            engine.dispatch_preview((), preview_tx);

            // Direct observable comparison: state and hash must be unchanged
            prop_assert_eq!(
                engine.read(),
                state_before,
                "state changed after dispatch_preview"
            );
            prop_assert_eq!(
                engine.replay_hash(),
                hash_before,
                "replay_hash changed after dispatch_preview"
            );

            // Indirect lifetime/enabled isolation check: if dispatch_preview
            // had mutated lifetimes or enabled, subsequent dispatches would
            // diverge from a reference engine that never saw the preview.
            let mut reference_engine: Engine<i32, CounterOp, (), u8> =
                Engine::new(state_before);
            reference_engine.add_rule(NoRule, RuleLifetime::Permanent);

            for op in &ops {
                let mut tx_ref = Transaction::new();
                tx_ref.ops.push(op.clone());
                reference_engine.dispatch((), tx_ref);

                let mut tx_actual = Transaction::new();
                tx_actual.ops.push(op.clone());
                engine.dispatch((), tx_actual);
            }

            prop_assert_eq!(
                engine.read(),
                reference_engine.read(),
                "post-preview dispatches diverged from reference — lifetimes/enabled were mutated"
            );
        }
    }

    // --------------------------------------------------------
    // PROP-03: rule lifetimes disable at exactly n dispatches
    // --------------------------------------------------------

    proptest! {
        #[test]
        fn prop_03_turns_lifetime(n in 1u32..=10u32) {
            let trigger_count = Rc::new(Cell::new(0u32));
            let trigger_count_clone = Rc::clone(&trigger_count);

            let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
            engine.add_rule(
                CountingRule { trigger_count },
                RuleLifetime::Turns(n),
            );

            // Dispatch n - 1 times: rule must still be enabled (Turns not yet 0)
            for _ in 0..n.saturating_sub(1) {
                let tx = Transaction::new();
                engine.dispatch((), tx);
            }

            // n-th dispatch: Turns reaches 0 and rule is removed from enabled
            // CountingRule.before() is NOT called by Turns (Turns uses post-dispatch loop)
            // so trigger_count tracks before() calls, which reflect prior dispatches
            let tx = Transaction::new();
            engine.dispatch((), tx);

            // After n dispatches, the Turns post-dispatch loop should have fired n times
            // CountingRule.before() is called on each dispatch while enabled
            prop_assert_eq!(
                trigger_count_clone.get(),
                n,
                "CountingRule.before() should have been called exactly n={} times, got {}",
                n,
                trigger_count_clone.get()
            );

            // Confirm disabled: one more dispatch must not increment trigger_count
            let tx_extra = Transaction::new();
            engine.dispatch((), tx_extra);

            prop_assert_eq!(
                trigger_count_clone.get(),
                n,
                "Rule still fired after being disabled at n={} Turns",
                n
            );
        }
    }

    proptest! {
        #[test]
        fn prop_03_triggers_lifetime(n in 1u32..=10u32) {
            let trigger_count = Rc::new(Cell::new(0u32));
            let trigger_count_clone = Rc::clone(&trigger_count);

            let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
            engine.add_rule(
                CountingRule { trigger_count },
                RuleLifetime::Triggers(n),
            );

            // Dispatch n times: rule fires n times, on the nth before() call
            // the Triggers counter reaches 0 and rule is removed from enabled
            for _ in 0..n {
                let tx = Transaction::new();
                engine.dispatch((), tx);
            }

            prop_assert_eq!(
                trigger_count_clone.get(),
                n,
                "CountingRule.before() should have been called exactly n={} times, got {}",
                n,
                trigger_count_clone.get()
            );

            // Confirm disabled: one more dispatch must not increment trigger_count
            let tx_extra = Transaction::new();
            engine.dispatch((), tx_extra);

            prop_assert_eq!(
                trigger_count_clone.get(),
                n,
                "Rule still fired after being disabled at n={} Triggers",
                n
            );
        }
    }

    // --------------------------------------------------------
    // PROP-04: cancelled transaction leaves state and replay_hash unchanged
    // --------------------------------------------------------

    proptest! {
        #[test]
        fn prop_04_cancelled_tx_isolation(ops in op_sequence_strategy()) {
            let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
            engine.add_rule(NoRule, RuleLifetime::Permanent);

            let state_before = engine.read();
            let hash_before = engine.replay_hash();

            let mut tx = Transaction::new();
            for op in &ops {
                tx.ops.push(op.clone());
            }
            tx.cancelled = true;

            engine.dispatch((), tx);

            prop_assert_eq!(engine.read(), state_before);
            prop_assert_eq!(engine.replay_hash(), hash_before);
        }
    }
}
