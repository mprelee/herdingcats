//! The HerdingCats dispatch engine.
//!
//! [`Engine<E>`] is the central coordinator of the library. It holds a committed
//! game state and an ordered list of behaviors. Call [`Engine::dispatch`] to
//! evaluate an input through all behaviors in deterministic order and atomically
//! commit the resulting state change.

use std::borrow::Cow;
use crate::spec::EngineSpec;
use crate::behavior::{Behavior, BehaviorResult};
use crate::outcome::{EngineError, Frame, Outcome};
use crate::reversibility::Reversibility;
use crate::apply::Apply;

/// The HerdingCats dispatch engine.
///
/// Holds the committed game state and an ordered list of behaviors. Call
/// [`dispatch`](Engine::dispatch) to evaluate an input through all behaviors
/// in deterministic order and atomically commit the resulting state change.
///
/// # Type parameter
///
/// `E` must implement [`EngineSpec`], bundling all associated game types.
///
/// # Example
///
/// ```
/// use herdingcats::{Engine, EngineSpec, Apply, Behavior, BehaviorResult, Reversibility};
///
/// struct MySpec;
///
/// #[derive(Debug, Clone)]
/// struct AppendDiff(u8);
///
/// impl Apply<MySpec> for AppendDiff {
///     fn apply(&self, state: &mut Vec<u8>) -> Vec<String> {
///         state.push(self.0);
///         vec![format!("appended {}", self.0)]
///     }
/// }
///
/// impl EngineSpec for MySpec {
///     type State = Vec<u8>;
///     type Input = u8;
///     type Diff = AppendDiff;
///     type Trace = String;
///     type NonCommittedInfo = String;
///     type OrderKey = u32;
/// }
///
/// struct AppendBehavior;
///
/// impl Behavior<MySpec> for AppendBehavior {
///     fn name(&self) -> &'static str { "append" }
///     fn order_key(&self) -> u32 { 0 }
///     fn evaluate(&self, input: &u8, _state: &Vec<u8>) -> BehaviorResult<AppendDiff, String> {
///         BehaviorResult::Continue(vec![AppendDiff(*input)])
///     }
/// }
///
/// let engine = Engine::<MySpec>::new(vec![], vec![Box::new(AppendBehavior)]);
/// assert_eq!(engine.state(), &vec![]);
/// ```
pub struct Engine<E: EngineSpec> {
    state: E::State,
    behaviors: Vec<Box<dyn Behavior<E>>>,
    // Phase 3 will populate these stacks with snapshot-based undo/redo.
    #[allow(dead_code)]
    undo_stack: Vec<E::State>,
    #[allow(dead_code)]
    redo_stack: Vec<E::State>,
}

impl<E: EngineSpec> Engine<E> {
    /// Construct a new engine with the given initial state and behavior set.
    ///
    /// Behaviors are sorted once by `(order_key, name)` — deterministic total
    /// order, no address-based tiebreaking.
    pub fn new(state: E::State, mut behaviors: Vec<Box<dyn Behavior<E>>>) -> Self {
        behaviors.sort_by(|a, b| {
            a.order_key()
                .cmp(&b.order_key())
                .then_with(|| a.name().cmp(b.name()))
        });
        Engine {
            state,
            behaviors,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Read-only reference to the current committed game state.
    pub fn state(&self) -> &E::State {
        &self.state
    }

    /// Evaluate `input` through all behaviors in deterministic order and atomically
    /// commit the result.
    ///
    /// # CoW semantics
    ///
    /// State is not cloned until the first diff is applied. A no-op dispatch
    /// (no behavior produces diffs) never touches the allocator.
    ///
    /// # Atomicity
    ///
    /// `self.state` is only updated if at least one diff was produced. If dispatch
    /// returns `NoChange`, `InvalidInput`, `Disallowed`, or `Aborted`, `self.state`
    /// is identical to its pre-dispatch value.
    pub fn dispatch(
        &mut self,
        input: E::Input,
        reversibility: Reversibility,
    ) -> Result<Outcome<Frame<E>, E::NonCommittedInfo>, EngineError> {
        // Cow::Borrowed — zero cost, no clone yet.
        let mut working: Cow<'_, E::State> = Cow::Borrowed(&self.state);
        let mut diffs: Vec<E::Diff> = Vec::new();
        let mut traces: Vec<E::Trace> = Vec::new();

        for behavior in &self.behaviors {
            match behavior.evaluate(&input, &*working) {
                BehaviorResult::Stop(info) => {
                    return Ok(Outcome::Aborted(info));
                }
                BehaviorResult::Continue(new_diffs) => {
                    for diff in new_diffs {
                        // to_mut() clones state on FIRST call only (Cow::Borrowed → Owned).
                        let new_traces = diff.apply(working.to_mut());
                        traces.extend(new_traces);
                        diffs.push(diff);
                    }
                }
            }
        }

        if diffs.is_empty() {
            return Ok(Outcome::NoChange);
        }

        // Atomic commit — self.state is replaced exactly once, at the end.
        self.state = working.into_owned();

        let frame = Frame {
            input,
            diffs,
            traces,
            reversibility,
        };
        Ok(Outcome::Committed(frame))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apply::Apply;
    use crate::spec::EngineSpec;

    // -----------------------------------------------------------------------
    // Test infrastructure
    // -----------------------------------------------------------------------

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

    /// A behavior that records its name into State (as bytes) so we can verify
    /// sort order by inspecting the committed state.
    struct TracingBehavior {
        key: u32,
        behavior_name: &'static str,
        /// The byte value to emit as a diff when evaluated.
        diff_byte: u8,
    }

    impl Behavior<TestSpec> for TracingBehavior {
        fn name(&self) -> &'static str {
            self.behavior_name
        }

        fn order_key(&self) -> u32 {
            self.key
        }

        fn evaluate(&self, _input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
            BehaviorResult::Continue(vec![self.diff_byte])
        }
    }

    /// A behavior that always emits no diffs (no-op).
    struct NoOpBehavior;

    impl Behavior<TestSpec> for NoOpBehavior {
        fn name(&self) -> &'static str {
            "noop"
        }

        fn order_key(&self) -> u32 {
            0
        }

        fn evaluate(&self, _input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
            BehaviorResult::Continue(vec![])
        }
    }

    /// A behavior that always calls Stop.
    struct StopBehavior;

    impl Behavior<TestSpec> for StopBehavior {
        fn name(&self) -> &'static str {
            "stopper"
        }

        fn order_key(&self) -> u32 {
            0
        }

        fn evaluate(&self, _input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
            BehaviorResult::Stop("halted".to_string())
        }
    }

    /// A behavior that emits a diff equal to its input.
    struct EchoBehavior {
        key: u32,
        behavior_name: &'static str,
    }

    impl Behavior<TestSpec> for EchoBehavior {
        fn name(&self) -> &'static str {
            self.behavior_name
        }

        fn order_key(&self) -> u32 {
            self.key
        }

        fn evaluate(&self, input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
            BehaviorResult::Continue(vec![*input])
        }
    }

    /// A behavior that reads the current working state length and emits it as a diff,
    /// allowing verification that later behaviors see earlier diffs applied.
    struct StateReadingBehavior;

    impl Behavior<TestSpec> for StateReadingBehavior {
        fn name(&self) -> &'static str {
            "state_reader"
        }

        fn order_key(&self) -> u32 {
            10
        }

        fn evaluate(&self, _input: &u8, state: &Vec<u8>) -> BehaviorResult<u8, String> {
            // Emit the current state length — if an earlier behavior applied a diff,
            // this will be non-zero even if the initial state was empty.
            BehaviorResult::Continue(vec![state.len() as u8])
        }
    }

    // -----------------------------------------------------------------------
    // Task 1 tests: Engine struct + new() + state()
    // -----------------------------------------------------------------------

    #[test]
    fn engine_new_sorts_behaviors_by_order_key_then_name() {
        // Behaviors provided in out-of-order: keys [2, 0, 1], names ["c", "a", "b"]
        // Expected sort: (0,"a"), (1,"b"), (2,"c") → diff bytes [10, 20, 30]
        let behaviors: Vec<Box<dyn Behavior<TestSpec>>> = vec![
            Box::new(TracingBehavior { key: 2, behavior_name: "c", diff_byte: 30 }),
            Box::new(TracingBehavior { key: 0, behavior_name: "a", diff_byte: 10 }),
            Box::new(TracingBehavior { key: 1, behavior_name: "b", diff_byte: 20 }),
        ];
        let mut engine = Engine::<TestSpec>::new(vec![], behaviors);
        let outcome = engine.dispatch(0u8, Reversibility::Reversible).unwrap();
        if let Outcome::Committed(frame) = outcome {
            assert_eq!(frame.diffs, vec![10u8, 20u8, 30u8],
                "behaviors must be evaluated in (order_key, name) sort order");
        } else {
            panic!("expected Committed outcome");
        }
    }

    #[test]
    fn engine_state_returns_ref_to_committed_state() {
        let initial = vec![1u8, 2u8, 3u8];
        let engine = Engine::<TestSpec>::new(initial.clone(), vec![]);
        assert_eq!(engine.state(), &initial);
    }

    #[test]
    fn engine_struct_has_placeholder_history_fields() {
        // Compile-time test: Engine must have undo_stack and redo_stack fields.
        // If they did not exist, this function (which uses them via new()) would not compile.
        // We verify indirectly: constructing an engine succeeds and state is accessible.
        let engine = Engine::<TestSpec>::new(vec![42u8], vec![]);
        assert_eq!(engine.state(), &vec![42u8]);
        // The undo_stack and redo_stack fields exist on the struct (verified by compilation).
    }

    // -----------------------------------------------------------------------
    // Task 2 tests: dispatch() — CoW, ordering, stop, frame, reversibility
    // -----------------------------------------------------------------------

    #[test]
    fn cow_no_clone_on_no_op_dispatch() {
        let initial: Vec<u8> = vec![1, 2, 3];
        let mut engine = Engine::<TestSpec>::new(initial, vec![Box::new(NoOpBehavior)]);

        // Compare the inner heap buffer pointer — the allocation address — not the Vec
        // struct address. A no-op dispatch must not reallocate the state buffer.
        let heap_ptr_before = engine.state().as_ptr();
        let outcome = engine.dispatch(0u8, Reversibility::Reversible).unwrap();

        // A no-op dispatch must return NoChange without touching the allocator.
        assert!(matches!(outcome, Outcome::NoChange));

        let heap_ptr_after = engine.state().as_ptr();
        assert!(
            std::ptr::eq(heap_ptr_before, heap_ptr_after),
            "heap buffer pointer must not change on a no-op dispatch (CoW guarantee)"
        );
    }

    #[test]
    fn cow_clones_on_first_diff() {
        let initial: Vec<u8> = vec![1, 2, 3];
        let mut engine = Engine::<TestSpec>::new(
            initial,
            vec![Box::new(EchoBehavior { key: 0, behavior_name: "echo" })],
        );

        // Compare the inner heap buffer pointer. After a diff is applied, Cow clones
        // the Vec (new heap allocation), so the buffer pointer must differ.
        let heap_ptr_before = engine.state().as_ptr();
        let outcome = engine.dispatch(42u8, Reversibility::Reversible).unwrap();

        assert!(matches!(outcome, Outcome::Committed(_)));

        let heap_ptr_after = engine.state().as_ptr();
        assert!(
            !std::ptr::eq(heap_ptr_before, heap_ptr_after),
            "heap buffer pointer must change when a diff is applied (clone occurred)"
        );
    }

    #[test]
    fn dispatch_evaluates_in_deterministic_order() {
        // Two behaviors: key=1 emits 200, key=0 emits 100 → frame.diffs must be [100, 200].
        let behaviors: Vec<Box<dyn Behavior<TestSpec>>> = vec![
            Box::new(TracingBehavior { key: 1, behavior_name: "second", diff_byte: 200 }),
            Box::new(TracingBehavior { key: 0, behavior_name: "first",  diff_byte: 100 }),
        ];
        let mut engine = Engine::<TestSpec>::new(vec![], behaviors);
        let outcome = engine.dispatch(0u8, Reversibility::Reversible).unwrap();
        if let Outcome::Committed(frame) = outcome {
            assert_eq!(frame.diffs, vec![100u8, 200u8]);
        } else {
            panic!("expected Committed");
        }
    }

    #[test]
    fn later_behavior_sees_earlier_diffs() {
        // EchoBehavior (key=0) emits the input byte, appending it to state.
        // StateReadingBehavior (key=10) reads the current state length and emits it.
        // Initial state is empty; after EchoBehavior applies one diff, length == 1.
        // So StateReadingBehavior should emit 1u8.
        let behaviors: Vec<Box<dyn Behavior<TestSpec>>> = vec![
            Box::new(EchoBehavior { key: 0, behavior_name: "echo" }),
            Box::new(StateReadingBehavior),
        ];
        let mut engine = Engine::<TestSpec>::new(vec![], behaviors);
        let outcome = engine.dispatch(99u8, Reversibility::Reversible).unwrap();
        if let Outcome::Committed(frame) = outcome {
            // frame.diffs: [99 (echo), 1 (state length after echo)]
            assert_eq!(frame.diffs.len(), 2);
            assert_eq!(frame.diffs[0], 99u8);
            assert_eq!(frame.diffs[1], 1u8,
                "StateReadingBehavior must see state after EchoBehavior applied its diff");
        } else {
            panic!("expected Committed");
        }
    }

    #[test]
    fn trace_appended_at_diff_application() {
        // Each u8 diff emits "applied N" trace when applied.
        // Two diffs → two traces in the same order.
        let behaviors: Vec<Box<dyn Behavior<TestSpec>>> = vec![
            Box::new(TracingBehavior { key: 0, behavior_name: "first",  diff_byte: 10 }),
            Box::new(TracingBehavior { key: 1, behavior_name: "second", diff_byte: 20 }),
        ];
        let mut engine = Engine::<TestSpec>::new(vec![], behaviors);
        let outcome = engine.dispatch(0u8, Reversibility::Reversible).unwrap();
        if let Outcome::Committed(frame) = outcome {
            assert_eq!(frame.traces, vec!["applied 10".to_string(), "applied 20".to_string()]);
        } else {
            panic!("expected Committed");
        }
    }

    #[test]
    fn stop_halts_dispatch() {
        // StopBehavior (key=0) fires first and returns Stop. No frame should be committed.
        let behaviors: Vec<Box<dyn Behavior<TestSpec>>> = vec![
            Box::new(StopBehavior),
            Box::new(EchoBehavior { key: 99, behavior_name: "echo" }),
        ];
        let mut engine = Engine::<TestSpec>::new(vec![], behaviors);
        let outcome = engine.dispatch(42u8, Reversibility::Reversible).unwrap();
        assert!(
            matches!(outcome, Outcome::Aborted(ref msg) if msg == "halted"),
            "Stop must return Aborted with the Stop payload"
        );
        // State must be unchanged.
        assert_eq!(engine.state(), &vec![] as &Vec<u8>);
    }

    #[test]
    fn no_frame_on_no_diffs() {
        let mut engine = Engine::<TestSpec>::new(vec![], vec![Box::new(NoOpBehavior)]);
        let outcome = engine.dispatch(0u8, Reversibility::Reversible).unwrap();
        assert!(
            matches!(outcome, Outcome::NoChange),
            "dispatch with zero diffs must return NoChange"
        );
    }

    #[test]
    fn frame_contains_input_diffs_trace() {
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![Box::new(EchoBehavior { key: 0, behavior_name: "echo" })],
        );
        let outcome = engine
            .dispatch(77u8, Reversibility::Irreversible)
            .unwrap();
        if let Outcome::Committed(frame) = outcome {
            assert_eq!(frame.input, 77u8);
            assert!(!frame.diffs.is_empty());
            assert!(!frame.traces.is_empty());
            assert_eq!(frame.reversibility, Reversibility::Irreversible);
        } else {
            panic!("expected Committed");
        }
    }

    #[test]
    fn dispatch_requires_reversibility_param() {
        // Structural test: dispatch() signature requires Reversibility as 2nd arg.
        // If the param were missing or defaulted, this call would fail to compile.
        let mut engine = Engine::<TestSpec>::new(vec![], vec![]);
        let _ = engine.dispatch(0u8, Reversibility::Reversible);
        let _ = engine.dispatch(0u8, Reversibility::Irreversible);
    }
}
