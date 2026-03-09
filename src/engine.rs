// ============================================================
// Commit Frame (private)
// ============================================================

use std::collections::{HashMap, HashSet};

use crate::hash::{fnv1a_hash, FNV_OFFSET, FNV_PRIME};
use crate::operation::Operation;
use crate::rule::Rule;
use crate::transaction::{RuleLifetime, Transaction};

#[derive(Clone)]
struct CommitFrame<S, O> {
    tx: Transaction<O>,
    state_hash_before: u64,
    state_hash_after: u64,
    lifetime_snapshot: HashMap<&'static str, RuleLifetime>,
    enabled_snapshot: HashSet<&'static str>,
    _marker: std::marker::PhantomData<S>,
}

// ============================================================
// Engine
// ============================================================

pub struct Engine<S, O, E, P>
where
    S: Clone,
    O: Operation<S>,
    P: Copy + Ord,
{
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

    pub fn replay_hash(&self) -> u64 {
        self.replay_hash
    }

    pub fn add_rule<R>(
        &mut self,
        rule: R,
        lifetime: RuleLifetime,
    ) where
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

    pub fn dispatch(&mut self, mut event: E, mut tx: Transaction<O>) {
        let hash_before = self.replay_hash;
        let lifetime_snapshot = self.lifetimes.clone();
        let enabled_snapshot = self.enabled.clone();

        for rule in &self.rules {
            if self.enabled.contains(rule.id()) {
                rule.before(&self.state, &mut event, &mut tx);

                if let Some(RuleLifetime::Triggers(n)) =
                    self.lifetimes.get_mut(rule.id())
                {
                    if *n > 0 {
                        *n -= 1;
                        if *n == 0 {
                            self.enabled.remove(rule.id());
                        }
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
            if let RuleLifetime::Turns(n) = lifetime {
                if *n > 0 {
                    *n -= 1;
                    if *n == 0 {
                        self.enabled.remove(id);
                    }
                }
            }
        }

        if tx.irreversible && !tx.cancelled {
            for op in &tx.ops {
                let h = fnv1a_hash(&op.hash_bytes());
                self.replay_hash ^= h;
                self.replay_hash =
                    self.replay_hash.wrapping_mul(FNV_PRIME);
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

    pub fn read(&self) -> S {
        self.state.clone()
    }

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
