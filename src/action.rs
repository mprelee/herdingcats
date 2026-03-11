// ============================================================
// Action
// ============================================================

/// A mutation proposal built by [`Behavior`](crate::Behavior)s during event dispatch.
///
/// An `Action` is not applied directly by user code. Instead, behaviors populate
/// it with mutations during their [`before`](crate::Behavior::before) hook, and the
/// engine applies those mutations to state after all behaviors have run. Each field
/// controls a different aspect of how the engine handles the commit:
///
/// - `mutations` — the ordered list of mutations to apply to state, in sequence.
/// - `cancelled` — any behavior can set this to `true` during `before()` to abort
///   the entire action; the engine skips `apply` and discards the frame.
#[derive(Clone)]
pub struct Action<M> {
    /// The mutations to apply to state in order when this action commits.
    pub mutations: Vec<M>,
    /// Set to `true` by any behavior to abort this action before mutations are applied.
    pub cancelled: bool,
}

impl<M> Action<M> {
    /// Create a new, empty action with the default settings.
    ///
    /// Defaults: `mutations` is empty, `cancelled` is `false`.
    /// Populate `mutations` via behaviors or directly before passing the action to
    /// [`Engine::dispatch_with`](crate::Engine::dispatch_with).
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::Action;
    ///
    /// // An Action<i32> where the mutation type is i32 (placeholder type here)
    /// let tx: Action<i32> = Action::new();
    /// assert!(tx.mutations.is_empty());
    /// assert!(!tx.cancelled);
    /// ```
    pub fn new() -> Self {
        Self {
            mutations: vec![],
            cancelled: false,
        }
    }
}

impl<M> Default for Action<M> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_new_defaults() {
        let tx: Action<()> = Action::new();
        assert!(tx.mutations.is_empty());
        assert!(!tx.cancelled);
    }

    #[test]
    fn action_has_mutations_and_cancelled_only() {
        let tx: Action<()> = Action::new();
        assert!(tx.mutations.is_empty());
        assert!(!tx.cancelled);
    }

    #[test]
    fn action_cancelled_can_be_set() {
        let mut tx: Action<()> = Action::new();
        tx.cancelled = true;
        assert!(tx.cancelled);
    }
}
