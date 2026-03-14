use crate::apply::Apply;

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
/// use herdingcats::{EngineSpec, Apply};
///
/// struct MySpec;
///
/// #[derive(Debug, Clone)]
/// struct MyDiff(u8);
///
/// impl Apply<MySpec> for MyDiff {
///     fn apply(&self, state: &mut Vec<u8>) -> Vec<String> {
///         state.push(self.0);
///         vec![format!("pushed {}", self.0)]
///     }
/// }
///
/// impl EngineSpec for MySpec {
///     type State = Vec<u8>;
///     type Input = String;
///     type Diff = MyDiff;
///     type Trace = String;
///     type NonCommittedInfo = String;
///     type OrderKey = u32;
/// }
/// ```
///
/// # `Diff` must implement `Apply<Self>`
///
/// The `Diff` associated type carries an `Apply<Self>` bound. This means
/// every diff type must be capable of applying itself to the game state.
/// A type that does not implement `Apply` cannot be used as `EngineSpec::Diff`:
///
/// ```compile_fail
/// use herdingcats::{EngineSpec, Apply};
///
/// struct BadSpec;
///
/// struct BadDiff; // intentionally missing Apply<BadSpec>
///
/// impl EngineSpec for BadSpec {
///     type State = Vec<u8>;
///     type Input = u8;
///     type Diff = BadDiff; // compile error: BadDiff does not impl Apply<BadSpec>
///     type Trace = String;
///     type NonCommittedInfo = String;
///     type OrderKey = u32;
/// }
/// ```
pub trait EngineSpec: Sized {
    /// The game state type. Must support cloning (for CoW snapshots) and
    /// debug formatting. Default construction is NOT required by the engine —
    /// `Engine::new()` takes the initial state as a parameter.
    type State: Clone + std::fmt::Debug;

    /// A player input (move, action, command). Must support cloning and debug.
    type Input: Clone + std::fmt::Debug;

    /// A single, atomic state mutation produced by a behavior during dispatch.
    /// Must support cloning, debug, and self-application via [`Apply`].
    type Diff: Clone + std::fmt::Debug + Apply<Self>;

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
    use crate::apply::Apply;

    struct TestSpec;

    impl EngineSpec for TestSpec {
        type State = Vec<u8>;
        type Input = u8;
        type Diff = u8;
        type Trace = String;
        type NonCommittedInfo = String;
        type OrderKey = u32;
    }

    // u8 satisfies the Apply<TestSpec> bound required by EngineSpec::Diff.
    impl Apply<TestSpec> for u8 {
        fn apply(&self, state: &mut Vec<u8>) -> Vec<String> {
            state.push(*self);
            vec![format!("applied {}", self)]
        }
    }

    #[test]
    fn engine_spec_associated_types_satisfy_bounds() {
        // State: Clone + Debug (no Default required)
        let state: <TestSpec as EngineSpec>::State = vec![];
        let _cloned = state.clone();
        let _debug = format!("{:?}", state);

        // Input: Clone + Debug
        let input: <TestSpec as EngineSpec>::Input = 42u8;
        #[allow(clippy::clone_on_copy)]
        let _cloned_input = input.clone();
        let _debug_input = format!("{:?}", input);

        // Diff: Clone + Debug
        let diff: <TestSpec as EngineSpec>::Diff = 1u8;
        #[allow(clippy::clone_on_copy)]
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
