// ============================================================
// Transaction
// ============================================================

/// A mutation proposal built by [`Rule`](crate::Rule)s during event dispatch.
///
/// A `Transaction` is not applied directly by user code. Instead, rules populate
/// it with operations during their [`before`](crate::Rule::before) hook, and the
/// engine applies those operations to state after all rules have run. Each field
/// controls a different aspect of how the engine handles the commit:
///
/// - `ops` — the ordered list of operations to apply to state, in sequence.
/// - `irreversible` — when `true` (the default), the commit is pushed onto the
///   undo stack; when `false`, it is applied but never recorded for undo.
/// - `deterministic` — when `true` (the default), each op's `hash_bytes` is
///   mixed into the engine's `replay_hash` fingerprint; set `false` for
///   cosmetic or non-game-logic mutations (animations, sound triggers, etc.).
/// - `cancelled` — any rule can set this to `true` during `before()` to abort
///   the entire transaction; the engine skips `apply` and discards the frame.
#[derive(Clone)]
pub struct Transaction<O> {
    /// The operations to apply to state in order when this transaction commits.
    pub ops: Vec<O>,
    /// Whether this commit is recorded on the undo stack. Defaults to `true`.
    pub irreversible: bool,
    /// Whether each op's `hash_bytes` is mixed into `replay_hash`. Defaults to `true`.
    pub deterministic: bool,
    /// Set to `true` by any rule to abort this transaction before ops are applied.
    pub cancelled: bool,
}

impl<O> Transaction<O> {
    /// Create a new, empty transaction with the default settings.
    ///
    /// Defaults: `ops` is empty, `irreversible` is `true`, `deterministic` is
    /// `true`, `cancelled` is `false`. Populate `ops` via rules or directly
    /// before passing the transaction to [`Engine::dispatch`](crate::Engine::dispatch).
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::Transaction;
    ///
    /// // A Transaction<i32> where the op type is i32 (placeholder type here)
    /// let tx: Transaction<i32> = Transaction::new();
    /// assert!(tx.ops.is_empty());
    /// assert!(tx.irreversible);
    /// assert!(tx.deterministic);
    /// assert!(!tx.cancelled);
    /// ```
    pub fn new() -> Self {
        Self {
            ops: vec![],
            irreversible: true,
            deterministic: true,
            cancelled: false,
        }
    }
}

// ============================================================
// Rule Lifetime
// ============================================================

/// Controls how long a [`Rule`](crate::Rule) stays active after [`Engine::add_rule`](crate::Engine::add_rule).
///
/// Rules are not always permanent. `RuleLifetime` lets you express time-limited
/// game effects: a rule that applies for the next three turns, or a rule that
/// fires only once. The engine tracks the remaining count and automatically
/// disables the rule when it expires.
#[derive(Clone, Copy, Debug)]
pub enum RuleLifetime {
    /// The rule stays active indefinitely. Use this for core game rules that
    /// always apply (e.g., movement validation, win condition checks).
    Permanent,
    /// The rule is disabled after `n` calls to [`Engine::dispatch`](crate::Engine::dispatch),
    /// regardless of whether the rule's `before` hook ran. Use this to model
    /// turn-limited effects ("this buff lasts three turns").
    Turns(u32),
    /// The rule is disabled after its `before` hook runs `n` times. Unlike
    /// `Turns`, this counts only dispatch calls where the rule was actually
    /// triggered. Use this to model one-shot or limited-trigger reactions.
    Triggers(u32),
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_new_defaults() {
        let tx: Transaction<()> = Transaction::new();
        assert!(tx.ops.is_empty());
        assert!(tx.irreversible);
        assert!(tx.deterministic);
        assert!(!tx.cancelled);
    }

    #[test]
    fn transaction_cancelled_can_be_set() {
        let mut tx: Transaction<()> = Transaction::new();
        tx.cancelled = true;
        assert!(tx.cancelled);
    }
}
