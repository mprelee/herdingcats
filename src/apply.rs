use crate::spec::EngineSpec;

/// Applies this diff to the working state and returns trace entries generated
/// by this mutation.
///
/// Application and trace generation are combined in one call: this structurally
/// prevents a diff from being applied without emitting its corresponding trace.
///
/// # Example
///
/// ```
/// use herdingcats::{Apply, EngineSpec};
///
/// struct MySpec;
/// impl EngineSpec for MySpec {
///     type State = Vec<u8>;
///     type Input = u8;
///     type Diff = u8;
///     type Trace = String;
///     type NonCommittedInfo = String;
///     type OrderKey = u32;
/// }
///
/// impl Apply<MySpec> for u8 {
///     fn apply(&self, state: &mut Vec<u8>) -> Vec<String> {
///         state.push(*self);
///         vec![format!("pushed {}", self)]
///     }
/// }
/// ```
pub trait Apply<E: EngineSpec> {
    /// Mutate `state` with this diff and return any trace entries produced.
    ///
    /// Each call MUST return at least one trace entry describing the mutation. The engine does not accept empty trace from a state-mutating diff.
    /// The engine enforces this contract with a `debug_assert!` in dispatch — violations panic in debug/test builds.
    fn apply(&self, state: &mut E::State) -> Vec<E::Trace>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct AppendByte(u8);

    struct TestSpec;

    impl EngineSpec for TestSpec {
        type State = Vec<u8>;
        type Input = u8;
        type Diff = AppendByte;
        type Trace = String;
        type NonCommittedInfo = String;
        type OrderKey = u32;
    }

    impl Apply<TestSpec> for AppendByte {
        fn apply(&self, state: &mut Vec<u8>) -> Vec<String> {
            state.push(self.0);
            vec!["applied".to_string()]
        }
    }

    struct NoOpDiff;

    impl Apply<TestSpec> for NoOpDiff {
        fn apply(&self, _state: &mut Vec<u8>) -> Vec<String> {
            vec![]
        }
    }

    #[test]
    fn apply_trait_compiles_and_returns_traces() {
        let diff = AppendByte(42);
        let mut state: Vec<u8> = vec![];
        let traces = diff.apply(&mut state);
        assert_eq!(state, vec![42u8]);
        assert_eq!(traces, vec!["applied".to_string()]);
    }

    #[test]
    fn apply_returns_empty_traces_for_no_op() {
        let diff = NoOpDiff;
        let mut state: Vec<u8> = vec![1, 2, 3];
        let traces = diff.apply(&mut state);
        assert_eq!(state, vec![1u8, 2u8, 3u8]); // state unchanged
        assert!(traces.is_empty());
    }

    // Contract tests: named trace_contract_* to make contract intent explicit and
    // distinguishable from compilation/behavior tests above.

    #[test]
    fn trace_contract_mutating_diff_returns_at_least_one_entry() {
        // A diff that mutates state MUST return at least one trace entry.
        // This test documents and enforces that contract.
        let diff = AppendByte(42);
        let mut state: Vec<u8> = vec![];
        let traces = diff.apply(&mut state);
        assert!(
            !traces.is_empty(),
            "a mutating Apply::apply call must return at least one trace entry"
        );
    }

    #[test]
    fn trace_contract_noop_diff_may_return_zero_entries() {
        // A diff that performs no state mutation is permitted to return zero trace entries.
        // This documents the allowed lower bound for no-op diffs.
        let diff = NoOpDiff;
        let mut state: Vec<u8> = vec![1, 2, 3];
        let traces = diff.apply(&mut state);
        // State must not have changed (this is a genuine no-op).
        assert_eq!(state, vec![1u8, 2u8, 3u8]);
        assert!(
            traces.is_empty(),
            "a no-op Apply::apply call may return zero trace entries"
        );
    }
}
