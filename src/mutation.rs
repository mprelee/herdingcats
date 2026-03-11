// ============================================================
// Mutation Trait
// ============================================================

/// A reversible mutation that can be applied to and removed from a state value.
///
/// A `Mutation` is the atomic unit of change in this engine. Behaviors collect
/// mutations into [`Action`](crate::Action)s during event dispatch;
/// the engine then applies them in order. Every mutation must encode its own
/// inverse: `undo` must restore the state to exactly what it was before `apply`
/// was called, making the undo stack possible without storing full state snapshots.
/// Each mutation also contributes to the engine's `replay_hash` fingerprint via
/// `hash_bytes`, enabling two engine instances to verify they have executed the
/// same deterministic sequence of moves.
pub trait Mutation<S>: Clone {
    /// Apply this mutation to `state`, advancing it by one logical step.
    ///
    /// `apply` must be the exact inverse of [`undo`](Mutation::undo): calling
    /// `apply` followed immediately by `undo` must leave `state` unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::Mutation;
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
    /// let m = CounterOp::Inc;
    /// let mut state = 0i32;
    /// m.apply(&mut state);
    /// assert_eq!(state, 1);
    /// ```
    fn apply(&self, state: &mut S);

    /// Reverse this mutation, restoring `state` to what it was before `apply`.
    ///
    /// The contract is strict: `apply(undo(s)) == s` and `undo(apply(s)) == s`
    /// for any reachable state. Mutations that cannot be reversed (e.g., lossy
    /// mutations) should store the prior value in the mutation itself — see
    /// the `Reset { prior: i32 }` pattern in the engine tests.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::Mutation;
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
    /// let m = CounterOp::Inc;
    /// let mut state = 1i32;
    /// m.undo(&mut state);
    /// assert_eq!(state, 0);
    /// ```
    fn undo(&self, state: &mut S);

    /// Return a deterministic byte sequence that uniquely identifies this
    /// mutation variant and its data.
    ///
    /// The engine concatenates the `hash_bytes()` of all mutations in an action
    /// into a single sequence, applies fnv1a to that sequence, and folds the
    /// result into its running `replay_hash`. This means the byte sequence
    /// must be **pure and stable**: the same logical mutation must always return
    /// the same bytes across runs. Different mutation variants — and variants
    /// with different data (e.g., `Reset { prior: 3 }` vs `Reset { prior: 7 }`)
    /// — must return different bytes so the fingerprint is collision-resistant.
    ///
    /// # Examples
    ///
    /// ```
    /// use herdingcats::Mutation;
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
    /// let m = CounterOp::Inc;
    /// // hash_bytes is deterministic: identical calls return identical bytes
    /// assert_eq!(m.hash_bytes(), m.hash_bytes());
    /// assert!(!m.hash_bytes().is_empty());
    /// ```
    fn hash_bytes(&self) -> Vec<u8>;

    /// Returns `true` by default. Override to return `false` for mutations that
    /// represent irreversible state changes (e.g., dice rolls, card draws).
    /// The engine uses this at commit time — if any mutation in an Action returns
    /// `false`, the entire Action is treated as irreversible and the undo stack
    /// is cleared.
    ///
    /// # Examples
    ///
    /// ```
    /// # use herdingcats::{Mutation, Behavior, Action, Engine};
    /// #[derive(Clone)]
    /// enum DiceOp { Roll }
    ///
    /// impl Mutation<u32> for DiceOp {
    ///     fn apply(&self, state: &mut u32) { *state = 42; }
    ///     fn undo(&self, _state: &mut u32) {}
    ///     fn hash_bytes(&self) -> Vec<u8>  { vec![0xD1] }
    ///     fn is_reversible(&self) -> bool  { false }
    /// }
    ///
    /// struct PassBehavior;
    ///
    /// impl Behavior<u32, DiceOp, (), u8> for PassBehavior {
    ///     fn id(&self) -> &'static str { "pass" }
    ///     fn priority(&self) -> u8    { 0 }
    /// }
    ///
    /// let mut engine = Engine::new(0u32);
    /// engine.add_behavior(PassBehavior);
    ///
    /// let mut tx = Action::new();
    /// tx.mutations.push(DiceOp::Roll);
    /// engine.dispatch_with((), tx);
    ///
    /// // DiceOp::Roll is not reversible, so the undo stack was cleared
    /// assert!(!engine.can_undo());
    /// ```
    fn is_reversible(&self) -> bool {
        true
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct Inc;

    impl Mutation<i32> for Inc {
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
    fn mutation_apply_undo_invert() {
        let op = Inc;
        let mut state = 0i32;
        op.apply(&mut state);
        assert_eq!(state, 1);
        op.undo(&mut state);
        assert_eq!(state, 0);
    }

    #[test]
    fn is_reversible_default_true() {
        let op = Inc;
        assert!(op.is_reversible());
    }

    #[derive(Clone)]
    struct IrreversibleOp;

    impl Mutation<i32> for IrreversibleOp {
        fn apply(&self, state: &mut i32) {
            *state = 0;
        }
        fn undo(&self, _state: &mut i32) {}
        fn hash_bytes(&self) -> Vec<u8> {
            vec![99]
        }
        fn is_reversible(&self) -> bool {
            false
        }
    }

    #[test]
    fn is_reversible_override_false() {
        let op = IrreversibleOp;
        assert!(!op.is_reversible());
    }
}
