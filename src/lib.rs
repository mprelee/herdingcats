use std::collections::{HashMap, HashSet};

//
// ============================================================
// FNV-1a 64-bit
// ============================================================
//

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

fn fnv1a_hash(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

//
// ============================================================
// Operation Trait
// ============================================================
//

pub trait Operation<S>: Clone {
    fn apply(&self, state: &mut S);
    fn undo(&self, state: &mut S);
    fn hash_bytes(&self) -> Vec<u8>;
}

//
// ============================================================
// Transaction
// ============================================================
//

#[derive(Clone)]
pub struct Transaction<O> {
    pub ops: Vec<O>,
    pub irreversible: bool,
    pub deterministic: bool,
    pub cancelled: bool,
}

impl<O> Transaction<O> {
    pub fn new() -> Self {
        Self {
            ops: vec![],
            irreversible: true,
            deterministic: true,
            cancelled: false,
        }
    }
}

//
// ============================================================
// Rule Lifetime
// ============================================================
//

#[derive(Clone, Copy, Debug)]
pub enum RuleLifetime {
    Permanent,
    Turns(u32),
    Triggers(u32),
}

//
// ============================================================
// Rule Trait
// ============================================================
//

pub trait Rule<S, O, E, P>
where
    S: Clone,
    O: Operation<S>,
    P: Copy + Ord,
{
    fn id(&self) -> &'static str;
    fn priority(&self) -> P;

    fn before(
        &self,
        _state: &S,
        _event: &mut E,
        _tx: &mut Transaction<O>,
    ) {}

    fn after(
        &self,
        _state: &S,
        _event: &E,
        _tx: &mut Transaction<O>,
    ) {}
}

//
// ============================================================
// Commit Frame
// ============================================================
//

#[derive(Clone)]
struct CommitFrame<S, O> {
    tx: Transaction<O>,
    state_hash_before: u64,
    state_hash_after: u64,
    lifetime_snapshot: HashMap<&'static str, RuleLifetime>,
    enabled_snapshot: HashSet<&'static str>,
    _marker: std::marker::PhantomData<S>,
}

//
// ============================================================
// Engine
// ============================================================
//

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