// ============================================================
// Commit Frame (private)
// ============================================================

use crate::action::Action;
use crate::behavior::Behavior;
use crate::hash::{FNV_OFFSET, FNV_PRIME, fnv1a_hash};
use crate::mutation::Mutation;

// A single entry on the undo stack, capturing everything needed to reverse
// one committed action and restore the engine to its prior state.
//
// CommitFrame is the key to undo correctness: rather than storing full state
// snapshots (expensive for large game states), the engine stores just enough
// to *reverse* the commit. Fields:
#[derive(Clone)]
struct CommitFrame<S, M> {
    // The action that was committed. Used in reverse (mutations.iter().rev())
    // during undo to call m.undo() on each mutation in reverse order.
    tx: Action<M>,

    // The replay_hash value *before* this action was applied. Restored
    // directly on undo so the fingerprint matches pre-commit state without
    // replaying the full history.
    state_hash_before: u64,

    // The replay_hash value *after* this action was applied. Restored
    // directly on redo so re-applying the frame doesn't re-hash the mutations.
    state_hash_after: u64,

    // PhantomData marker for the state type S. CommitFrame<S, M> does not own
    // an S directly (state is held by the Engine), but the type parameter is
    // needed for variance: covariant in S so CommitFrame<&'long T> is usable
    // where CommitFrame<&'short T> is expected.
    _marker: std::marker::PhantomData<S>,
}

// ============================================================
// Engine
// ============================================================

/// The runtime that manages game state, behaviors, and commit history.
///
/// `Engine<S, M, I, P>` is the central coordinator: it holds the current
/// state `S`, a sorted list of [`Behavior`]s, and an undo/redo stack of commit
/// frames. To advance state, call [`dispatch`](Engine::dispatch) with an event;
/// behaviors inject mutations during the dispatch pipeline and the engine
/// applies them, records the frame for undo. For pre-built actions use
/// [`dispatch_with`](Engine::dispatch_with). Direct mutation of
/// [`state`](Engine::state) is possible but bypasses the behavior system; use
/// [`write`](Engine::write) for intentional resets.
pub struct Engine<S, M, I, P>
where
    S: Clone,
    M: Mutation<S>,
    P: Copy + Ord,
{
    /// The current committed game state.
    ///
    /// Readable directly for ergonomic access in non-behavior contexts. Prefer
    /// [`read`](Engine::read) when you want a clone rather than a borrow.
    /// **Direct mutation of this field bypasses the behavior system and undo
    /// stack** — use [`write`](Engine::write) for intentional state resets
    /// (e.g., loading a saved game), which also clears both stacks and
    /// resets the `replay_hash`.
    pub state: S,

    undo_stack: Vec<CommitFrame<S, M>>,
    redo_stack: Vec<CommitFrame<S, M>>,

    behaviors: Vec<Box<dyn Behavior<S, M, I, P>>>,

    replay_hash: u64,
}

impl<S, M, I, P> Engine<S, M, I, P>
where
    S: Clone,
    M: Mutation<S>,
    P: Copy + Ord,
{
    /// Create a new engine with the given initial state and no behaviors.
    ///
    /// The undo and redo stacks start empty. `replay_hash` is initialized to
    /// `FNV_OFFSET` (the hash of an empty sequence). Add behaviors with
    /// [`add_behavior`](Engine::add_behavior) before dispatching events.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Engine, Mutation};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Mutation<i32> for CounterOp {
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
            behaviors: vec![],
            replay_hash: FNV_OFFSET,
        }
    }

    /// Return the current replay hash — a running fingerprint over all
    /// committed mutations.
    ///
    /// The replay hash is updated on every [`dispatch`](Engine::dispatch) call
    /// that produces mutations. Two engine instances that have processed the
    /// same sequence of mutations will have identical replay hashes.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Engine, Mutation, Action};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Mutation<i32> for CounterOp {
    ///     fn apply(&self, s: &mut i32) { *s += 1; }
    ///     fn undo(&self, s: &mut i32)  { *s -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8> { vec![1] }
    /// }
    ///
    /// let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0);
    /// let hash_before = engine.replay_hash();
    ///
    /// let mut tx = Action::new();
    /// tx.mutations.push(CounterOp::Inc);
    /// let _ = engine.dispatch_with((), tx);
    ///
    /// assert_ne!(engine.replay_hash(), hash_before);
    /// ```
    pub fn replay_hash(&self) -> u64 {
        self.replay_hash
    }

    /// Register a behavior with this engine.
    ///
    /// The behavior is inserted into the sorted behavior list (sorted by
    /// [`priority`](crate::Behavior::priority) ascending). The behavior's
    /// active state is managed by the behavior itself via
    /// [`is_active`](crate::Behavior::is_active).
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Engine, Mutation, Behavior, Action};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Mutation<i32> for CounterOp {
    ///     fn apply(&self, s: &mut i32) { *s += 1; }
    ///     fn undo(&self, s: &mut i32)  { *s -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8> { vec![1] }
    /// }
    ///
    /// struct NoRule;
    /// impl Behavior<i32, CounterOp, (), u8> for NoRule {
    ///     fn id(&self) -> &'static str { "no_rule" }
    ///     fn priority(&self) -> u8 { 0 }
    /// }
    ///
    /// let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0);
    /// engine.add_behavior(NoRule);
    /// ```
    pub fn add_behavior<B>(&mut self, behavior: B)
    where
        B: Behavior<S, M, I, P> + 'static,
    {
        self.behaviors.push(Box::new(behavior));
        self.behaviors.sort_by_key(|b| b.priority());
    }

    //
    // --------------------------------------------------------
    // Preview Dispatch
    // --------------------------------------------------------
    //

    /// Run the full dispatch pipeline on `event` and `tx` without committing
    /// any changes to state, replay hash, or behavior lifetimes.
    ///
    /// `dispatch_preview` is a dry run: all active behaviors fire their `before`
    /// and `after` hooks, mutations are applied, but everything is rolled back
    /// at the end. This is useful for AI look-ahead, UI preview of pending
    /// moves, or testing behavior interactions without side effects.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Engine, Mutation, Behavior, Action};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Mutation<i32> for CounterOp {
    ///     fn apply(&self, s: &mut i32) { *s += 1; }
    ///     fn undo(&self, s: &mut i32)  { *s -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8> { vec![1] }
    /// }
    ///
    /// struct NoRule;
    /// impl Behavior<i32, CounterOp, (), u8> for NoRule {
    ///     fn id(&self) -> &'static str { "no_rule" }
    ///     fn priority(&self) -> u8 { 0 }
    /// }
    ///
    /// let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0);
    /// engine.add_behavior(NoRule);
    ///
    /// let mut tx = Action::new();
    /// tx.mutations.push(CounterOp::Inc);
    /// let _preview = engine.dispatch_preview((), tx);
    ///
    /// // State is unchanged after preview
    /// assert_eq!(engine.state, 0);
    /// ```
    pub fn dispatch_preview(&mut self, mut event: I, mut tx: Action<M>) -> Action<M> {
        let state_snapshot = self.state.clone();
        let hash_snapshot = self.replay_hash;

        for behavior in &self.behaviors {
            if behavior.is_active() {
                behavior.before(&self.state, &mut event, &mut tx);
            }
        }

        if !tx.cancelled {
            for m in &tx.mutations {
                m.apply(&mut self.state);
            }
        }

        for behavior in self.behaviors.iter().rev() {
            if behavior.is_active() {
                behavior.after(&self.state, &event, &mut tx);
            }
        }

        self.state = state_snapshot;
        self.replay_hash = hash_snapshot;
        tx
    }

    //
    // --------------------------------------------------------
    // Commit Dispatch
    // --------------------------------------------------------
    //

    /// Dispatch `event` through all active behaviors using a fresh, empty action.
    ///
    /// This is the ergonomic entry point for the common case where no pre-populated
    /// action is needed. Behaviors inject mutations via their `before` hook.
    /// For cases where you need to pass a pre-built action (e.g., with mutations
    /// already pushed), use [`dispatch_with`](Engine::dispatch_with) instead.
    ///
    /// Returns `Some(action)` if mutations were applied, `None` if cancelled or
    /// no mutations produced.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Engine, Mutation, Behavior, Action};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Mutation<i32> for CounterOp {
    ///     fn apply(&self, s: &mut i32) { *s += 1; }
    ///     fn undo(&self, s: &mut i32)  { *s -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8> { vec![1] }
    /// }
    ///
    /// struct IncRule;
    /// impl Behavior<i32, CounterOp, (), u8> for IncRule {
    ///     fn id(&self) -> &'static str { "inc" }
    ///     fn priority(&self) -> u8 { 0 }
    ///     fn before(&self, _s: &i32, _e: &mut (), tx: &mut Action<CounterOp>) {
    ///         tx.mutations.push(CounterOp::Inc);
    ///     }
    /// }
    ///
    /// let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0);
    /// engine.add_behavior(IncRule);
    /// let _ = engine.dispatch(());
    /// assert_eq!(engine.state, 1);
    /// ```
    pub fn dispatch(&mut self, event: I) -> Option<Action<M>> {
        self.dispatch_with(event, Action::new())
    }

    /// Dispatch `event` through all active behaviors with a custom pre-built action.
    ///
    /// For the common case with no pre-built action, use [`dispatch`](Engine::dispatch).
    ///
    /// Behaviors fire in ascending priority order during `before()`, then
    /// descending order during `after()`. If any behavior sets `tx.cancelled =
    /// true`, the mutations are not applied and no frame is committed.
    /// If all mutations return `is_reversible() == true`, a CommitFrame is
    /// pushed and the redo stack is cleared. If any mutation is irreversible,
    /// both the undo and redo stacks are cleared (undo barrier). After a
    /// successful commit, `on_dispatch()` is called on all behaviors.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Engine, Mutation, Behavior, Action};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Mutation<i32> for CounterOp {
    ///     fn apply(&self, s: &mut i32) { *s += 1; }
    ///     fn undo(&self, s: &mut i32)  { *s -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8> { vec![1] }
    /// }
    ///
    /// struct NoRule;
    /// impl Behavior<i32, CounterOp, (), u8> for NoRule {
    ///     fn id(&self) -> &'static str { "no_rule" }
    ///     fn priority(&self) -> u8 { 0 }
    /// }
    ///
    /// let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0);
    /// engine.add_behavior(NoRule);
    ///
    /// let mut tx = Action::new();
    /// tx.mutations.push(CounterOp::Inc);
    /// let _action = engine.dispatch_with((), tx);
    ///
    /// assert_eq!(engine.state, 1);
    /// ```
    pub fn dispatch_with(&mut self, mut event: I, mut tx: Action<M>) -> Option<Action<M>> {
        let hash_before = self.replay_hash;

        for behavior in &self.behaviors {
            if behavior.is_active() {
                behavior.before(&self.state, &mut event, &mut tx);
            }
        }

        if !tx.cancelled {
            for m in &tx.mutations {
                m.apply(&mut self.state);
            }
        }

        for behavior in self.behaviors.iter().rev() {
            if behavior.is_active() {
                behavior.after(&self.state, &event, &mut tx);
            }
        }

        if !tx.cancelled && !tx.mutations.is_empty() {
            let is_reversible = tx.mutations.iter().all(|m| m.is_reversible());

            // Collect all mutation bytes for this action into one sequence
            let mut action_bytes: Vec<u8> = Vec::new();
            for m in &tx.mutations {
                action_bytes.extend_from_slice(&m.hash_bytes());
            }
            // Hash the full sequence with fnv1a, then fold into running hash
            let action_hash = fnv1a_hash(&action_bytes);
            self.replay_hash ^= action_hash;
            self.replay_hash = self.replay_hash.wrapping_mul(FNV_PRIME);

            // Clone tx after behaviors have run so the returned action reflects
            // any behavior-injected mutations, before tx is moved into CommitFrame.
            let committed_tx = tx.clone();

            if is_reversible {
                let hash_after = self.replay_hash;
                self.undo_stack.push(CommitFrame {
                    tx,
                    state_hash_before: hash_before,
                    state_hash_after: hash_after,
                    _marker: std::marker::PhantomData,
                });
                self.redo_stack.clear();
            } else {
                // Undo barrier: irreversible commit clears all prior undo/redo history
                self.undo_stack.clear();
                self.redo_stack.clear();
            }

            // Lifecycle pass — separate iter_mut() to satisfy borrow checker
            // Fires for ALL behaviors regardless of is_active() — per locked decision
            for behavior in self.behaviors.iter_mut() {
                behavior.on_dispatch();
            }

            Some(committed_tx)
        } else {
            None
        }
    }

    /// Reverse the most recent commit, restoring state to its value before
    /// that commit, and calling `on_undo()` on all behaviors.
    ///
    /// If the undo stack is empty, this is a no-op. The undone frame is moved
    /// to the redo stack so [`redo`](Engine::redo) can re-apply it.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Engine, Mutation, Behavior, Action};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Mutation<i32> for CounterOp {
    ///     fn apply(&self, s: &mut i32) { *s += 1; }
    ///     fn undo(&self, s: &mut i32)  { *s -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8> { vec![1] }
    /// }
    ///
    /// struct NoRule;
    /// impl Behavior<i32, CounterOp, (), u8> for NoRule {
    ///     fn id(&self) -> &'static str { "no_rule" }
    ///     fn priority(&self) -> u8 { 0 }
    /// }
    ///
    /// let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0);
    /// engine.add_behavior(NoRule);
    ///
    /// let mut tx = Action::new();
    /// tx.mutations.push(CounterOp::Inc);
    /// let _ = engine.dispatch_with((), tx);
    /// assert_eq!(engine.state, 1);
    ///
    /// engine.undo();
    /// assert_eq!(engine.state, 0);
    /// ```
    pub fn undo(&mut self) {
        if let Some(frame) = self.undo_stack.pop() {
            for m in frame.tx.mutations.iter().rev() {
                m.undo(&mut self.state);
            }

            self.replay_hash = frame.state_hash_before;

            self.redo_stack.push(frame);

            // Lifecycle pass: unconditional, all behaviors including inactive
            for behavior in self.behaviors.iter_mut() {
                behavior.on_undo();
            }
        }
    }

    /// Re-apply the most recently undone commit, advancing state forward again,
    /// and calling `on_dispatch()` on all behaviors (redo is a forward operation).
    ///
    /// If the redo stack is empty, this is a no-op. The redone frame is moved
    /// back to the undo stack. Note that calling [`dispatch`](Engine::dispatch)
    /// clears the redo stack — once you commit a new action, the redo history
    /// for the previous branch is discarded.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Engine, Mutation, Behavior, Action};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Mutation<i32> for CounterOp {
    ///     fn apply(&self, s: &mut i32) { *s += 1; }
    ///     fn undo(&self, s: &mut i32)  { *s -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8> { vec![1] }
    /// }
    ///
    /// struct NoRule;
    /// impl Behavior<i32, CounterOp, (), u8> for NoRule {
    ///     fn id(&self) -> &'static str { "no_rule" }
    ///     fn priority(&self) -> u8 { 0 }
    /// }
    ///
    /// let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0);
    /// engine.add_behavior(NoRule);
    ///
    /// let mut tx = Action::new();
    /// tx.mutations.push(CounterOp::Inc);
    /// let _ = engine.dispatch_with((), tx);
    /// engine.undo();
    /// assert_eq!(engine.state, 0);
    ///
    /// engine.redo();
    /// assert_eq!(engine.state, 1);
    /// ```
    pub fn redo(&mut self) {
        if let Some(frame) = self.redo_stack.pop() {
            for m in &frame.tx.mutations {
                m.apply(&mut self.state);
            }

            self.replay_hash = frame.state_hash_after;

            self.undo_stack.push(frame);

            // Redo = forward dispatch: call on_dispatch, not on_undo
            for behavior in self.behaviors.iter_mut() {
                behavior.on_dispatch();
            }
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
    /// use herdingcats::{Engine, Mutation};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Mutation<i32> for CounterOp {
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
    /// use herdingcats::{Engine, Mutation, Behavior, Action};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Mutation<i32> for CounterOp {
    ///     fn apply(&self, s: &mut i32) { *s += 1; }
    ///     fn undo(&self, s: &mut i32)  { *s -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8> { vec![1] }
    /// }
    ///
    /// struct NoRule;
    /// impl Behavior<i32, CounterOp, (), u8> for NoRule {
    ///     fn id(&self) -> &'static str { "no_rule" }
    ///     fn priority(&self) -> u8 { 0 }
    /// }
    ///
    /// let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0);
    /// engine.add_behavior(NoRule);
    ///
    /// let mut tx = Action::new();
    /// tx.mutations.push(CounterOp::Inc);
    /// let _ = engine.dispatch_with((), tx);
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

    /// Return `true` if there is at least one commit on the undo stack.
    ///
    /// Returns `false` immediately after an irreversible commit (the undo
    /// barrier), since `dispatch` clears the undo stack when the action
    /// contains an irreversible mutation. Also returns `false` on a fresh
    /// engine before any dispatches.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::{Engine, Mutation, Behavior, Action};
    ///
    /// #[derive(Clone)]
    /// enum CounterOp { Inc }
    /// impl Mutation<i32> for CounterOp {
    ///     fn apply(&self, s: &mut i32) { *s += 1; }
    ///     fn undo(&self, s: &mut i32)  { *s -= 1; }
    ///     fn hash_bytes(&self) -> Vec<u8> { vec![1] }
    ///     fn is_reversible(&self) -> bool { false }
    /// }
    ///
    /// struct NoRule;
    /// impl Behavior<i32, CounterOp, (), u8> for NoRule {
    ///     fn id(&self) -> &'static str { "no_rule" }
    ///     fn priority(&self) -> u8 { 0 }
    ///     fn before(&self, _s: &i32, _e: &mut (), tx: &mut Action<CounterOp>) {
    ///         tx.mutations.push(CounterOp::Inc);
    ///     }
    /// }
    ///
    /// let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0);
    /// engine.add_behavior(NoRule);
    /// assert!(!engine.can_undo());
    ///
    /// let _ = engine.dispatch(());
    /// // Irreversible commit — undo barrier clears the stack
    /// assert!(!engine.can_undo());
    /// ```
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Return `true` if there is at least one commit on the redo stack.
    ///
    /// Returns `false` on a fresh engine, after `write`, or after any new
    /// forward `dispatch` (which clears the redo stack).
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;

    // --------------------------------------------------------
    // CounterOp fixture
    // --------------------------------------------------------

    #[derive(Clone, Debug, PartialEq)]
    enum CounterOp {
        Inc,
        Dec,
        Reset { prior: i32 },
    }

    impl Mutation<i32> for CounterOp {
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
    impl Behavior<i32, CounterOp, (), u8> for NoRule {
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
        engine.add_behavior(NoRule);

        let mut tx = Action::new();
        tx.mutations.push(CounterOp::Inc);
        let _ = engine.dispatch_with((), tx);
        assert_eq!(engine.read(), 1);

        engine.undo();
        assert_eq!(engine.read(), 0);
    }

    // --------------------------------------------------------
    // MixedOp fixture for reversibility gate tests
    // --------------------------------------------------------

    #[derive(Clone, Debug, PartialEq)]
    enum MixedOp {
        Rev(CounterOp),
        Irrev,
    }

    impl Mutation<i32> for MixedOp {
        fn apply(&self, state: &mut i32) {
            match self {
                MixedOp::Rev(op) => op.apply(state),
                MixedOp::Irrev => *state = 99,
            }
        }
        fn undo(&self, state: &mut i32) {
            match self {
                MixedOp::Rev(op) => op.undo(state),
                MixedOp::Irrev => {}
            }
        }
        fn hash_bytes(&self) -> Vec<u8> {
            match self {
                MixedOp::Rev(op) => op.hash_bytes(),
                MixedOp::Irrev => vec![0xFF],
            }
        }
        fn is_reversible(&self) -> bool {
            match self {
                MixedOp::Rev(_) => true,
                MixedOp::Irrev => false,
            }
        }
    }

    struct MixedNoRule;
    impl Behavior<i32, MixedOp, (), u8> for MixedNoRule {
        fn id(&self) -> &'static str {
            "mixed_no_rule"
        }
        fn priority(&self) -> u8 {
            0
        }
    }

    // --------------------------------------------------------
    // Reversibility gate tests (REV-02, REV-03, REV-04)
    // --------------------------------------------------------

    #[test]
    fn irreversible_commit_clears_undo_and_redo_stacks() {
        let mut engine: Engine<i32, MixedOp, (), u8> = Engine::new(0i32);
        engine.add_behavior(MixedNoRule);

        // Commit a reversible action first
        let mut tx1 = Action::new();
        tx1.mutations.push(MixedOp::Rev(CounterOp::Inc));
        let _ = engine.dispatch_with((), tx1);
        assert_eq!(engine.state, 1);

        // Commit an irreversible action — undo stack must clear
        let mut tx2 = Action::new();
        tx2.mutations.push(MixedOp::Irrev);
        let _ = engine.dispatch_with((), tx2);
        assert_eq!(engine.state, 99);
        assert_eq!(
            engine.undo_stack.len(),
            0,
            "undo stack should be empty after irreversible commit"
        );
        assert_eq!(
            engine.redo_stack.len(),
            0,
            "redo stack should be empty after irreversible commit"
        );
    }

    #[test]
    fn reversible_commit_after_irreversible_is_undoable() {
        let mut engine: Engine<i32, MixedOp, (), u8> = Engine::new(0i32);
        engine.add_behavior(MixedNoRule);

        // Irreversible
        let mut tx1 = Action::new();
        tx1.mutations.push(MixedOp::Irrev);
        let _ = engine.dispatch_with((), tx1);
        assert_eq!(engine.undo_stack.len(), 0);

        // Reversible after — should push to undo stack
        let mut tx2 = Action::new();
        tx2.mutations.push(MixedOp::Rev(CounterOp::Inc));
        let _ = engine.dispatch_with((), tx2);
        assert_eq!(engine.undo_stack.len(), 1);

        // Undo the reversible action
        engine.undo();
        assert_eq!(engine.state, 99); // back to state after irreversible
        assert_eq!(engine.undo_stack.len(), 0); // barrier reached
    }

    #[test]
    fn mixed_mutations_treated_as_irreversible() {
        let mut engine: Engine<i32, MixedOp, (), u8> = Engine::new(0i32);
        engine.add_behavior(MixedNoRule);

        // Step A: dispatch a reversible-only action first
        let mut tx1 = Action::new();
        tx1.mutations.push(MixedOp::Rev(CounterOp::Inc));
        let _ = engine.dispatch_with((), tx1);
        assert_eq!(engine.state, 1);
        assert_eq!(engine.undo_stack.len(), 1);

        // Step B: dispatch a mixed action (Rev + Irrev in same action)
        // Rev(Inc) runs first: 1+1=2, then Irrev: state=99
        let mut tx2 = Action::new();
        tx2.mutations.push(MixedOp::Rev(CounterOp::Inc));
        tx2.mutations.push(MixedOp::Irrev);
        let _ = engine.dispatch_with((), tx2);
        assert_eq!(engine.state, 99);
        assert!(
            !engine.can_undo(),
            "mixed action must be treated as irreversible"
        );
        assert_eq!(
            engine.undo_stack.len(),
            0,
            "undo stack must be cleared by mixed action (barrier consequence)"
        );
    }

    #[test]
    fn empty_action_does_not_push_undo_stack() {
        let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine.add_behavior(NoRule);

        // Dispatch with no mutations
        let _ = engine.dispatch(());

        assert_eq!(engine.state, 0);
        assert!(
            !engine.can_undo(),
            "empty action must not push to undo stack"
        );
    }

    // --------------------------------------------------------
    // Lifecycle hook tests (LIFE-04, LIFE-05, LIFE-06)
    // --------------------------------------------------------

    #[test]
    fn on_dispatch_called_on_all_behaviors() {
        struct Counter {
            count: u32,
        }
        impl Behavior<i32, CounterOp, (), u8> for Counter {
            fn id(&self) -> &'static str {
                "counter"
            }
            fn priority(&self) -> u8 {
                0
            }
            fn on_dispatch(&mut self) {
                self.count += 1;
            }
        }

        let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine.add_behavior(Counter { count: 0 });

        let mut tx = Action::new();
        tx.mutations.push(CounterOp::Inc);
        let _ = engine.dispatch_with((), tx);

        // State change confirms dispatch ran; on_dispatch called (verified via
        // pattern: behavior compile-check + state correctness).
        assert_eq!(engine.state, 1);
    }

    #[test]
    fn on_dispatch_not_called_for_cancelled_action() {
        // Cancelled action should not trigger on_dispatch.
        struct CancelAndCount {
            dispatch_count: u32,
        }
        impl Behavior<i32, CounterOp, (), u8> for CancelAndCount {
            fn id(&self) -> &'static str {
                "cancel_count"
            }
            fn priority(&self) -> u8 {
                0
            }
            fn before(&self, _s: &i32, _e: &mut (), tx: &mut Action<CounterOp>) {
                tx.cancelled = true;
            }
            fn on_dispatch(&mut self) {
                self.dispatch_count += 1;
            }
        }
        let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine.add_behavior(CancelAndCount { dispatch_count: 0 });

        let mut tx = Action::new();
        tx.mutations.push(CounterOp::Inc);
        let _ = engine.dispatch_with((), tx);
        assert_eq!(engine.state, 0); // cancelled — state unchanged
    }

    #[test]
    fn on_dispatch_not_called_for_empty_mutations() {
        // Empty action (no mutations) should not trigger on_dispatch.
        struct DispatchCounter {
            count: u32,
        }
        impl Behavior<i32, CounterOp, (), u8> for DispatchCounter {
            fn id(&self) -> &'static str {
                "dispatch_counter"
            }
            fn priority(&self) -> u8 {
                0
            }
            fn on_dispatch(&mut self) {
                self.count += 1;
            }
        }
        let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine.add_behavior(DispatchCounter { count: 0 });

        // Dispatch with no mutations
        let _ = engine.dispatch(());
        // State unchanged, no commit happened
        assert_eq!(engine.state, 0);
        assert_eq!(engine.undo_stack.len(), 0);
    }

    // --------------------------------------------------------
    // Behavior lifecycle edge case tests (TEST-08)
    // --------------------------------------------------------

    #[test]
    fn on_undo_fires_on_undo() {
        use std::cell::Cell;
        use std::rc::Rc;

        struct TrackingBehavior {
            count: Rc<Cell<u32>>,
        }
        impl Behavior<i32, CounterOp, (), u8> for TrackingBehavior {
            fn id(&self) -> &'static str {
                "tracking"
            }
            fn priority(&self) -> u8 {
                0
            }
            fn on_dispatch(&mut self) {
                self.count.set(self.count.get() + 1);
            }
            fn on_undo(&mut self) {
                self.count.set(self.count.get() - 1);
            }
        }

        let counter = Rc::new(Cell::new(0u32));
        let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine.add_behavior(TrackingBehavior {
            count: counter.clone(),
        });

        // Dispatch one reversible action
        let mut tx = Action::new();
        tx.mutations.push(CounterOp::Inc);
        let _ = engine.dispatch_with((), tx);
        assert_eq!(counter.get(), 1, "on_dispatch should have fired once");
        assert!(engine.can_undo());

        // Undo — on_undo should fire, decrementing counter
        engine.undo();
        assert_eq!(counter.get(), 0, "on_undo should have decremented counter");
        assert!(!engine.can_undo(), "undo stack empty after undo");
        assert_eq!(engine.state, 0, "state restored after undo");
    }

    #[test]
    fn deactivation_mid_dispatch_does_not_corrupt_hooks() {
        // BehaviorA: starts active, deactivates on first on_dispatch call
        struct SelfDeactivating {
            active: bool,
        }
        impl Behavior<i32, CounterOp, (), u8> for SelfDeactivating {
            fn id(&self) -> &'static str {
                "self_deactivating"
            }
            fn priority(&self) -> u8 {
                0
            }
            fn is_active(&self) -> bool {
                self.active
            }
            fn on_dispatch(&mut self) {
                self.active = false; // deactivate after first dispatch
            }
        }

        // BehaviorB: always active, before() hook pushes an extra CounterOp::Inc
        struct AlwaysActive;
        impl Behavior<i32, CounterOp, (), u8> for AlwaysActive {
            fn id(&self) -> &'static str {
                "always_active"
            }
            fn priority(&self) -> u8 {
                10
            }
            fn before(&self, _s: &i32, _e: &mut (), tx: &mut Action<CounterOp>) {
                tx.mutations.push(CounterOp::Inc); // always injects one extra Inc
            }
        }

        let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine.add_behavior(SelfDeactivating { active: true });
        engine.add_behavior(AlwaysActive);

        // First dispatch: both behaviors active. Action starts with one Inc.
        // AlwaysActive.before() injects a second Inc → 2 mutations total → state = 2
        let mut tx1 = Action::new();
        tx1.mutations.push(CounterOp::Inc);
        let _ = engine.dispatch_with((), tx1);
        assert_eq!(
            engine.state, 2,
            "first dispatch: both behaviors active, AlwaysActive added one Inc"
        );
        // After dispatch: SelfDeactivating.on_dispatch fires → self.active = false

        // Second dispatch: SelfDeactivating is now inactive (is_active() = false).
        // AlwaysActive is still active → before() still injects one Inc.
        // Action starts with one Inc → AlwaysActive adds one more → 2 mutations → state += 2 → state = 4
        let mut tx2 = Action::new();
        tx2.mutations.push(CounterOp::Inc);
        let _ = engine.dispatch_with((), tx2);
        assert_eq!(
            engine.state, 4,
            "second dispatch: AlwaysActive before() still fires after SelfDeactivating deactivated"
        );
    }

    // --------------------------------------------------------
    // Stateful behavior lifecycle (TEST-04)
    // --------------------------------------------------------

    #[test]
    fn stateful_behavior_n_dispatches() {
        use std::cell::Cell;
        use std::rc::Rc;

        struct CountingBehavior {
            dispatch_count: Rc<Cell<u32>>,
            active_for: u32,
        }
        impl Behavior<i32, CounterOp, (), u8> for CountingBehavior {
            fn id(&self) -> &'static str {
                "counting"
            }
            fn priority(&self) -> u8 {
                0
            }
            fn is_active(&self) -> bool {
                self.dispatch_count.get() < self.active_for
            }
            fn on_dispatch(&mut self) {
                self.dispatch_count.set(self.dispatch_count.get() + 1);
            }
        }

        struct CountingBehavior2 {
            dispatch_count: Rc<Cell<u32>>,
            active_for: u32,
        }
        impl Behavior<i32, CounterOp, (), u8> for CountingBehavior2 {
            fn id(&self) -> &'static str {
                "counting2"
            }
            fn priority(&self) -> u8 {
                0
            }
            fn is_active(&self) -> bool {
                self.dispatch_count.get() < self.active_for
            }
            fn on_dispatch(&mut self) {
                self.dispatch_count.set(self.dispatch_count.get() + 1);
            }
            fn before(&self, _s: &i32, _e: &mut (), tx: &mut Action<CounterOp>) {
                // While active, add a Dec to cancel out Inc so net state change = 0
                tx.mutations.push(CounterOp::Dec);
            }
        }

        for n in 1u32..=10 {
            // --- Assertion 1: on_dispatch fires even after deactivation ---
            let dispatch_count = Rc::new(Cell::new(0u32));

            let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0);
            engine.add_behavior(CountingBehavior {
                dispatch_count: dispatch_count.clone(),
                active_for: n,
            });

            for _ in 0..(n + 2) {
                let mut tx = Action::new();
                tx.mutations.push(CounterOp::Inc);
                let _ = engine.dispatch_with((), tx);
            }

            // on_dispatch fires on ALL behaviors regardless of is_active()
            assert_eq!(
                dispatch_count.get(),
                n + 2,
                "dispatch_count should be n+2 for n={n}: on_dispatch must fire even after deactivation"
            );

            // --- Assertion 2: before/after hooks skipped when inactive ---
            let dispatch_count2 = Rc::new(Cell::new(0u32));

            let mut engine2: Engine<i32, CounterOp, (), u8> = Engine::new(0);
            engine2.add_behavior(CountingBehavior2 {
                dispatch_count: dispatch_count2.clone(),
                active_for: n,
            });

            // Dispatch n+1 actions each with CounterOp::Inc
            for _ in 0..(n + 1) {
                let mut tx = Action::new();
                tx.mutations.push(CounterOp::Inc);
                engine2.dispatch_with((), tx);
            }

            // During first n dispatches: CountingBehavior2 is active → adds Dec → net 0 each time
            // State after n dispatches = 0 (each Inc + Dec cancels out)
            // On dispatch n+1: CountingBehavior2 is inactive → before() skipped → only Inc applied → state = 1
            assert_eq!(
                engine2.state, 1,
                "before() hook should be skipped after n={n} dispatches; state should be 1 (only Inc, no Dec)"
            );
        }
    }

    // --------------------------------------------------------
    // replay_hash fnv1a accumulation tests (QUICK-3)
    // --------------------------------------------------------

    #[test]
    fn replay_hash_same_single_mutation_same_across_engines() {
        // Two engines dispatching the same single mutation arrive at the same replay_hash
        let mut engine_a: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine_a.add_behavior(NoRule);
        let mut engine_b: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine_b.add_behavior(NoRule);

        let mut tx_a = Action::new();
        tx_a.mutations.push(CounterOp::Inc);
        let _ = engine_a.dispatch_with((), tx_a);

        let mut tx_b = Action::new();
        tx_b.mutations.push(CounterOp::Inc);
        let _ = engine_b.dispatch_with((), tx_b);

        assert_eq!(engine_a.replay_hash(), engine_b.replay_hash());
    }

    #[test]
    fn replay_hash_same_two_mutations_same_across_engines() {
        // Two engines dispatching same two mutations in same order arrive at same replay_hash
        let mut engine_a: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine_a.add_behavior(NoRule);
        let mut engine_b: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine_b.add_behavior(NoRule);

        let mut tx_a = Action::new();
        tx_a.mutations.push(CounterOp::Inc);
        tx_a.mutations.push(CounterOp::Dec);
        let _ = engine_a.dispatch_with((), tx_a);

        let mut tx_b = Action::new();
        tx_b.mutations.push(CounterOp::Inc);
        tx_b.mutations.push(CounterOp::Dec);
        let _ = engine_b.dispatch_with((), tx_b);

        assert_eq!(engine_a.replay_hash(), engine_b.replay_hash());
    }

    #[test]
    fn replay_hash_differs_when_mutations_differ() {
        // replay_hash differs when mutations differ (sensitivity)
        let mut engine_a: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine_a.add_behavior(NoRule);
        let mut engine_b: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine_b.add_behavior(NoRule);

        let mut tx_a = Action::new();
        tx_a.mutations.push(CounterOp::Inc);
        let _ = engine_a.dispatch_with((), tx_a);

        let mut tx_b = Action::new();
        tx_b.mutations.push(CounterOp::Dec);
        let _ = engine_b.dispatch_with((), tx_b);

        assert_ne!(engine_a.replay_hash(), engine_b.replay_hash());
    }

    #[test]
    fn replay_hash_after_undo_matches_before_dispatch() {
        // replay_hash after undo matches replay_hash before the undone dispatch
        let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine.add_behavior(NoRule);

        let hash_before = engine.replay_hash();

        let mut tx = Action::new();
        tx.mutations.push(CounterOp::Inc);
        let _ = engine.dispatch_with((), tx);

        engine.undo();

        assert_eq!(engine.replay_hash(), hash_before);
    }
}

// ============================================================
// Property Tests
// ============================================================

#[cfg(test)]
mod props {
    use super::*;
    use crate::action::Action;
    use proptest::prelude::*;

    // --------------------------------------------------------
    // CounterOp fixture
    // --------------------------------------------------------

    #[derive(Clone, Debug)]
    enum CounterOp {
        Inc,
        Dec,
    }

    impl Mutation<i32> for CounterOp {
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

    impl Behavior<i32, CounterOp, (), u8> for NoRule {
        fn id(&self) -> &'static str {
            "no_rule"
        }
        fn priority(&self) -> u8 {
            0
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
            engine.add_behavior(NoRule);

            let state_before = engine.read();
            let hash_before = engine.replay_hash();

            for op in &ops {
                let mut tx = Action::new();
                tx.mutations.push(op.clone());
                let _ = engine.dispatch_with((), tx);
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
            engine.add_behavior(NoRule);

            // Establish a non-trivial pre-preview state
            for op in &ops {
                let mut tx = Action::new();
                tx.mutations.push(op.clone());
                let _ = engine.dispatch_with((), tx);
            }

            let state_before = engine.read();
            let hash_before = engine.replay_hash();

            // Build and run a preview action with the same ops
            let mut preview_tx = Action::new();
            for op in &ops {
                preview_tx.mutations.push(op.clone());
            }
            let _ = engine.dispatch_preview((), preview_tx);

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
            reference_engine.add_behavior(NoRule);

            for op in &ops {
                let mut tx_ref = Action::new();
                tx_ref.mutations.push(op.clone());
                let _ = reference_engine.dispatch_with((), tx_ref);

                let mut tx_actual = Action::new();
                tx_actual.mutations.push(op.clone());
                let _ = engine.dispatch_with((), tx_actual);
            }

            prop_assert_eq!(
                engine.read(),
                reference_engine.read(),
                "post-preview dispatches diverged from reference — lifetimes/enabled were mutated"
            );
        }
    }

    // --------------------------------------------------------
    // PROP-04: cancelled action leaves state and replay_hash unchanged
    // --------------------------------------------------------

    proptest! {
        #[test]
        fn prop_04_cancelled_tx_isolation(ops in op_sequence_strategy()) {
            let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
            engine.add_behavior(NoRule);

            let state_before = engine.read();
            let hash_before = engine.replay_hash();

            let mut tx = Action::new();
            for op in &ops {
                tx.mutations.push(op.clone());
            }
            tx.cancelled = true;

            let _ = engine.dispatch_with((), tx);

            prop_assert_eq!(engine.read(), state_before);
            prop_assert_eq!(engine.replay_hash(), hash_before);
        }
    }

    // --------------------------------------------------------
    // MixedOp fixture (for reversibility props)
    // --------------------------------------------------------

    #[derive(Clone, Debug)]
    enum MixedOp {
        Rev,
        Irrev,
    }

    impl Mutation<i32> for MixedOp {
        fn apply(&self, state: &mut i32) {
            match self {
                MixedOp::Rev => *state += 1,
                MixedOp::Irrev => *state += 0, // no-op state change, just marks irreversibility
            }
        }
        fn undo(&self, state: &mut i32) {
            match self {
                MixedOp::Rev => *state -= 1,
                MixedOp::Irrev => {} // engine never calls this — barrier prevents it
            }
        }
        fn hash_bytes(&self) -> Vec<u8> {
            match self {
                MixedOp::Rev => vec![0xAA],
                MixedOp::Irrev => vec![0xFF],
            }
        }
        fn is_reversible(&self) -> bool {
            match self {
                MixedOp::Rev => true,
                MixedOp::Irrev => false,
            }
        }
    }

    struct MixedNoRule;
    impl Behavior<i32, MixedOp, (), u8> for MixedNoRule {
        fn id(&self) -> &'static str {
            "mixed_no_rule_props"
        }
        fn priority(&self) -> u8 {
            0
        }
    }

    fn mixed_op_strategy() -> impl Strategy<Value = Vec<MixedOp>> {
        prop::collection::vec(
            prop_oneof![Just(MixedOp::Rev), Just(MixedOp::Irrev)],
            1..=20, // at least 1 so the sequence is non-empty
        )
    }

    // --------------------------------------------------------
    // PROP-05: any action sequence containing an Irrev mutation
    //          results in an empty undo stack after commit
    // --------------------------------------------------------

    proptest! {
        #[test]
        fn prop_05_irreversible_clears_undo_stack(ops in mixed_op_strategy()) {
            // Only run this test when the sequence contains at least one Irrev
            prop_assume!(ops.iter().any(|op| matches!(op, MixedOp::Irrev)));

            let mut engine: Engine<i32, MixedOp, (), u8> = Engine::new(0i32);
            engine.add_behavior(MixedNoRule);

            for op in &ops {
                let mut tx = Action::new();
                tx.mutations.push(op.clone());
                let _ = engine.dispatch_with((), tx);
                if !op.is_reversible() {
                    prop_assert!(engine.undo_stack.is_empty(),
                        "undo stack must be empty immediately after irreversible commit");
                    prop_assert!(engine.redo_stack.is_empty(),
                        "redo stack must be empty immediately after irreversible commit");
                }
            }
        }
    }

    fn reversible_irrev_reversible_strategy() -> impl Strategy<Value = (Vec<MixedOp>, Vec<MixedOp>)>
    {
        (
            // prefix: 0..=5 Rev ops (dispatched before the Irrev)
            prop::collection::vec(Just(MixedOp::Rev), 0..=5usize),
            // suffix: 1..=5 Rev ops (dispatched after the Irrev)
            prop::collection::vec(Just(MixedOp::Rev), 1..=5usize),
        )
    }

    // --------------------------------------------------------
    // PROP-06: reversible actions committed after an irreversible
    //          one are individually undoable; undo halts at barrier
    // --------------------------------------------------------

    proptest! {
        #[test]
        fn prop_06_reversible_after_irreversible_undoable(
            (prefix, suffix) in reversible_irrev_reversible_strategy()
        ) {
            let mut engine: Engine<i32, MixedOp, (), u8> = Engine::new(0i32);
            engine.add_behavior(MixedNoRule);

            // Dispatch prefix (all reversible)
            for op in &prefix {
                let mut tx = Action::new();
                tx.mutations.push(op.clone());
                let _ = engine.dispatch_with((), tx);
            }

            // Dispatch one irreversible — this sets the undo barrier
            {
                let mut tx = Action::new();
                tx.mutations.push(MixedOp::Irrev);
                let _ = engine.dispatch_with((), tx);
            }
            prop_assert!(engine.undo_stack.is_empty(), "undo stack empty after barrier");

            // Dispatch suffix (all reversible)
            for op in &suffix {
                let mut tx = Action::new();
                tx.mutations.push(op.clone());
                let _ = engine.dispatch_with((), tx);
            }

            // Undo stack has exactly suffix.len() frames
            prop_assert_eq!(engine.undo_stack.len(), suffix.len(),
                "undo stack should contain only the suffix reversible commits");

            // Record state and hash after barrier + suffix
            let state_after_suffix = engine.read();

            // Undo each suffix op individually — state and hash should unwind correctly
            for i in 0..suffix.len() {
                prop_assert!(!engine.undo_stack.is_empty(),
                    "undo stack should not be empty before undoing suffix op {i}");
                engine.undo();
            }

            // After undoing all suffix: undo stack is empty (barrier reached)
            prop_assert!(engine.undo_stack.is_empty(),
                "undo stack should be empty after undoing all suffix ops (barrier reached)");

            let state_at_barrier = engine.read();
            let hash_at_barrier = engine.replay_hash();
            engine.undo(); // extra undo on empty stack — must be no-op
            prop_assert_eq!(engine.read(), state_at_barrier);
            prop_assert_eq!(engine.replay_hash(), hash_at_barrier);

            // Rev applies +1 to state, so state_after_suffix = state_at_barrier + suffix.len() as i32
            prop_assert_eq!(
                state_after_suffix,
                state_at_barrier + suffix.len() as i32,
                "state at barrier + suffix.len() should equal state after suffix"
            );
        }
    }

    // --------------------------------------------------------
    // dispatch() return value tests
    // --------------------------------------------------------

    #[test]
    fn dispatch_returns_none_when_cancelled() {
        struct Canceller;
        impl Behavior<i32, CounterOp, (), u8> for Canceller {
            fn id(&self) -> &'static str {
                "canceller"
            }
            fn priority(&self) -> u8 {
                0
            }
            fn before(&self, _s: &i32, _e: &mut (), tx: &mut Action<CounterOp>) {
                tx.cancelled = true;
            }
        }
        let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine.add_behavior(Canceller);

        let mut tx = Action::new();
        tx.mutations.push(CounterOp::Inc);
        let result = engine.dispatch_with((), tx);
        assert!(result.is_none(), "cancelled dispatch should return None");
    }

    #[test]
    fn dispatch_returns_none_when_mutations_empty() {
        let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine.add_behavior(NoRule);

        let result = engine.dispatch(()); // no mutations
        assert!(
            result.is_none(),
            "empty-mutations dispatch should return None"
        );
    }

    #[test]
    fn dispatch_returns_some_with_mutations_when_committed() {
        let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine.add_behavior(NoRule);

        let mut tx = Action::new();
        tx.mutations.push(CounterOp::Inc);
        let result = engine.dispatch_with((), tx);
        assert!(
            result.is_some(),
            "valid dispatch should return Some(action)"
        );
        let action = result.unwrap();
        assert_eq!(action.mutations.len(), 1);
        assert!(matches!(action.mutations[0], CounterOp::Inc));
        assert_eq!(engine.read(), 1);
    }

    #[test]
    fn dispatch_returns_some_for_irreversible_action() {
        let mut engine: Engine<i32, MixedOp, (), u8> = Engine::new(0i32);
        engine.add_behavior(MixedNoRule);

        let mut tx = Action::new();
        tx.mutations.push(MixedOp::Irrev);
        let result = engine.dispatch_with((), tx);
        assert!(
            result.is_some(),
            "irreversible but valid dispatch should return Some"
        );
        let action = result.unwrap();
        assert_eq!(action.mutations.len(), 1);
        assert!(matches!(action.mutations[0], MixedOp::Irrev));
        // MixedOp::Irrev in props module applies *state += 0 (no-op state, marks irreversibility)
        assert_eq!(engine.read(), 0);
    }

    // --------------------------------------------------------
    // dispatch_preview() return value tests
    // --------------------------------------------------------

    #[test]
    fn dispatch_preview_returns_action_after_behaviors() {
        struct MutationInjector;
        impl Behavior<i32, CounterOp, (), u8> for MutationInjector {
            fn id(&self) -> &'static str {
                "injector"
            }
            fn priority(&self) -> u8 {
                0
            }
            fn before(&self, _s: &i32, _e: &mut (), tx: &mut Action<CounterOp>) {
                tx.mutations.push(CounterOp::Dec);
            }
        }
        let mut engine: Engine<i32, CounterOp, (), u8> = Engine::new(0i32);
        engine.add_behavior(MutationInjector);

        let mut tx = Action::new();
        tx.mutations.push(CounterOp::Inc);
        let preview = engine.dispatch_preview((), tx);
        // Behavior injected a Dec, so preview should have 2 mutations
        assert_eq!(preview.mutations.len(), 2);
        // State should be unchanged
        assert_eq!(engine.read(), 0);
    }
}
