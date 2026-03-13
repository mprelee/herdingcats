/// Bundles all game-specific associated types behind a single type parameter,
/// eliminating generic explosion in function signatures.
///
/// # Summary
///
/// Instead of writing `fn foo<S, I, D, T, O, K>(...)`, library code writes
/// `fn foo<E: EngineSpec>(...)`.  Users define a unit struct and implement
/// this trait once to wire together all their game types.
///
/// # Example
///
/// ```
/// use herdingcats::EngineSpec;
///
/// struct MySpec;
///
/// impl EngineSpec for MySpec {
///     type State = Vec<String>;
///     type Input = String;
///     type Diff = String;
///     type Trace = String;
///     type NonCommittedInfo = String;
///     type OrderKey = u32;
/// }
/// ```
pub trait EngineSpec {
    /// The game state type. Must support cloning (for CoW snapshots),
    /// debug formatting, and default construction (for engine initialisation).
    type State: Clone + std::fmt::Debug + Default;

    /// A player input (move, action, command). Must support cloning and debug.
    type Input: Clone + std::fmt::Debug;

    /// A single, atomic state mutation produced by a behavior during dispatch.
    /// Must support cloning and debug.
    type Diff: Clone + std::fmt::Debug;

    /// Per-diff metadata appended to `Frame` at the moment a diff is applied.
    /// Must support cloning and debug.
    type Trace: Clone + std::fmt::Debug;

    /// Payload carried by non-committed `Outcome` variants (`InvalidInput`,
    /// `Disallowed`, `Aborted`). Must support cloning and debug.
    type NonCommittedInfo: Clone + std::fmt::Debug;

    /// The type used to sort behaviors deterministically. Implements `Ord`;
    /// behaviors are ordered by `(order_key, name)` — no address-based tiebreaking.
    type OrderKey: Ord;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSpec;

    impl EngineSpec for TestSpec {
        type State = Vec<u8>;
        type Input = u8;
        type Diff = u8;
        type Trace = String;
        type NonCommittedInfo = String;
        type OrderKey = u32;
    }

    #[test]
    fn engine_spec_associated_types_satisfy_bounds() {
        // State: Clone + Debug + Default
        let state = <TestSpec as EngineSpec>::State::default();
        let _cloned = state.clone();
        let _debug = format!("{:?}", state);

        // Input: Clone + Debug
        let input: <TestSpec as EngineSpec>::Input = 42u8;
        let _cloned_input = input.clone();
        let _debug_input = format!("{:?}", input);

        // Diff: Clone + Debug
        let diff: <TestSpec as EngineSpec>::Diff = 1u8;
        let _cloned_diff = diff.clone();

        // Trace: Clone + Debug
        let trace: <TestSpec as EngineSpec>::Trace = String::from("t");
        let _cloned_trace = trace.clone();

        // NonCommittedInfo: Clone + Debug
        let nci: <TestSpec as EngineSpec>::NonCommittedInfo = String::from("n");
        let _cloned_nci = nci.clone();

        // OrderKey: Ord
        let k1: <TestSpec as EngineSpec>::OrderKey = 1u32;
        let k2: <TestSpec as EngineSpec>::OrderKey = 2u32;
        assert!(k1 < k2);
    }
}
