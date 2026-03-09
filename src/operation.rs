// ============================================================
// Operation Trait
// ============================================================

/// A reversible mutation that can be applied to and removed from a state value.
///
/// An `Operation` is the atomic unit of change in this engine. Rules collect
/// operations into [`Transaction`](crate::Transaction)s during event dispatch;
/// the engine then applies them in order. Every operation must encode its own
/// inverse: `undo` must restore the state to exactly what it was before `apply`
/// was called, making the undo stack possible without storing full state snapshots.
/// Each operation also contributes to the engine's `replay_hash` fingerprint via
/// `hash_bytes`, enabling two engine instances to verify they have executed the
/// same deterministic sequence of moves.
pub trait Operation<S>: Clone {
    /// Apply this operation to `state`, advancing it by one logical step.
    ///
    /// `apply` must be the exact inverse of [`undo`](Operation::undo): calling
    /// `apply` followed immediately by `undo` must leave `state` unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::Operation;
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
    /// let op = CounterOp::Inc;
    /// let mut state = 0i32;
    /// op.apply(&mut state);
    /// assert_eq!(state, 1);
    /// ```
    fn apply(&self, state: &mut S);

    /// Reverse this operation, restoring `state` to what it was before `apply`.
    ///
    /// The contract is strict: `apply(undo(s)) == s` and `undo(apply(s)) == s`
    /// for any reachable state. Operations that cannot be reversed (e.g., lossy
    /// mutations) should store the prior value in the operation itself — see
    /// the `Reset { prior: i32 }` pattern in the engine tests.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::Operation;
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
    /// let op = CounterOp::Inc;
    /// let mut state = 1i32;
    /// op.undo(&mut state);
    /// assert_eq!(state, 0);
    /// ```
    fn undo(&self, state: &mut S);

    /// Return a deterministic byte sequence that uniquely identifies this
    /// operation variant and its data.
    ///
    /// The engine XORs `fnv1a_hash(hash_bytes())` into its running `replay_hash`
    /// for every committed, deterministic operation. This means the byte sequence
    /// must be **pure and stable**: the same logical operation must always return
    /// the same bytes across runs. Different operation variants — and variants
    /// with different data (e.g., `Reset { prior: 3 }` vs `Reset { prior: 7 }`)
    /// — must return different bytes so the fingerprint is collision-resistant.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::Operation;
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
    /// let op = CounterOp::Inc;
    /// // hash_bytes is deterministic: identical calls return identical bytes
    /// assert_eq!(op.hash_bytes(), op.hash_bytes());
    /// assert!(!op.hash_bytes().is_empty());
    /// ```
    fn hash_bytes(&self) -> Vec<u8>;
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct Inc;

    impl Operation<i32> for Inc {
        fn apply(&self, state: &mut i32) {
            *state += 1;
        }
        fn undo(&self, state: &mut i32) {
            *state -= 1;
        }
        fn hash_bytes(&self) -> Vec<u8> {
            vec![0]
        }
    }

    #[test]
    fn operation_apply_undo_invert() {
        let op = Inc;
        let mut state = 0i32;
        op.apply(&mut state);
        assert_eq!(state, 1);
        op.undo(&mut state);
        assert_eq!(state, 0);
    }
}
