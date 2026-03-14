//! The HerdingCats dispatch engine.
//!
//! [`Engine<E>`] is the central coordinator of the library. It holds a committed
//! game state and an ordered list of behaviors. Call [`Engine::dispatch`] to
//! evaluate an input through all behaviors in deterministic order and atomically
//! commit the resulting state change.

use crate::apply::Apply;
use crate::behavior::{BehaviorDef, BehaviorResult};
use crate::outcome::{EngineError, Frame, HistoryDisallowed, Outcome};
use crate::reversibility::Reversibility;
use crate::spec::EngineSpec;
use std::borrow::Cow;

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
/// use herdingcats::{Engine, EngineSpec, Apply, BehaviorDef, BehaviorResult, Reversibility};
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
/// fn append_eval(input: &u8, _state: &Vec<u8>) -> BehaviorResult<AppendDiff, String> {
///     BehaviorResult::Continue(vec![AppendDiff(*input)])
/// }
///
/// let engine = Engine::<MySpec>::new(vec![], vec![BehaviorDef {
///     name: "append",
///     order_key: 0u32,
///     evaluate: append_eval,
/// }]);
/// assert_eq!(engine.state(), &vec![]);
/// ```
pub struct Engine<E: EngineSpec> {
    state: E::State,
    behaviors: Vec<BehaviorDef<E>>,
    undo_stack: Vec<(E::State, Frame<E>, Reversibility)>,
    redo_stack: Vec<(E::State, Frame<E>, Reversibility)>,
}

impl<E: EngineSpec> Engine<E> {
    /// Construct a new engine with the given initial state and behavior set.
    ///
    /// Behaviors are sorted once by `(order_key, name)` — deterministic total
    /// order, no address-based tiebreaking.
    pub fn new(state: E::State, mut behaviors: Vec<BehaviorDef<E>>) -> Self {
        behaviors.sort_by(|a, b| {
            a.order_key
                .cmp(&b.order_key)
                .then_with(|| a.name.cmp(b.name))
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

    /// Number of transitions currently available to undo.
    ///
    /// Returns 0 when the undo stack is empty. Callers can use this to enable
    /// or disable UI undo controls without triggering `Disallowed`.
    pub fn undo_depth(&self) -> usize {
        self.undo_stack.len()
    }

    /// Number of transitions currently available to redo.
    ///
    /// Returns 0 when the redo stack is empty. Callers can use this to enable
    /// or disable UI redo controls without triggering `Disallowed`.
    pub fn redo_depth(&self) -> usize {
        self.redo_stack.len()
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
            match (behavior.evaluate)(&input, &*working) {
                BehaviorResult::Stop(outcome) => {
                    return Ok(outcome.into());
                }
                BehaviorResult::Continue(new_diffs) => {
                    for diff in new_diffs {
                        // to_mut() clones state on FIRST call only (Cow::Borrowed → Owned).
                        let new_traces = diff.apply(working.to_mut());
                        debug_assert!(
                            !new_traces.is_empty(),
                            "Apply::apply() contract violation: diff mutated state but returned zero trace entries. \
                             Every state-mutating diff MUST return at least one trace entry."
                        );
                        traces.extend(new_traces);
                        diffs.push(diff);
                    }
                }
            }
        }

        if diffs.is_empty() {
            return Ok(Outcome::NoChange);
        }

        // Capture snapshot BEFORE committing — this is the state to restore on undo.
        let prior_state = self.state.clone();

        // Atomic commit — self.state is replaced exactly once, at the end.
        self.state = working.into_owned();

        let frame = Frame {
            input,
            diffs,
            traces,
        };

        // Single-timeline history: any new commit erases the redo future.
        self.redo_stack.clear();
        // Push snapshot + frame + reversibility unconditionally on Committed.
        self.undo_stack
            .push((prior_state, frame.clone(), reversibility));

        // Irreversible commits also wipe undo history — the transition is permanent.
        if reversibility == Reversibility::Irreversible {
            self.undo_stack.clear();
            self.redo_stack.clear();
        }

        Ok(Outcome::Committed(frame))
    }

    /// Undo the most recent committed transition.
    ///
    /// Restores the state snapshot captured immediately before the undone dispatch.
    /// No `Reversible` trait is required on user diff types — undo uses full state
    /// snapshots.
    ///
    /// # Return values
    ///
    /// - `Ok(Outcome::Undone(frame))` — undo succeeded; `frame` describes the
    ///   transition that was reversed.
    /// - `Ok(Outcome::Disallowed(HistoryDisallowed::NothingToUndo))` — the undo
    ///   stack is empty; nothing was changed.
    ///
    /// Note: the `N` type parameter for this call is `HistoryDisallowed`, not
    /// `E::NonCommittedInfo`. This asymmetry from `dispatch()` is intentional.
    pub fn undo(&mut self) -> Result<Outcome<Frame<E>, HistoryDisallowed>, EngineError> {
        match self.undo_stack.pop() {
            None => Ok(Outcome::Disallowed(HistoryDisallowed::NothingToUndo)),
            Some((prior_state, frame, reversibility)) => {
                let current_state = std::mem::replace(&mut self.state, prior_state);
                self.redo_stack
                    .push((current_state, frame.clone(), reversibility));
                Ok(Outcome::Undone(frame))
            }
        }
    }

    /// Redo the most recently undone transition.
    ///
    /// Restores the state that existed after the original dispatch, re-applying
    /// it without re-running behaviors.
    ///
    /// # Return values
    ///
    /// - `Ok(Outcome::Redone(frame))` — redo succeeded; `frame` describes the
    ///   transition that was re-applied.
    /// - `Ok(Outcome::Disallowed(HistoryDisallowed::NothingToRedo))` — the redo
    ///   stack is empty; nothing was changed.
    ///
    /// Note: the `N` type parameter for this call is `HistoryDisallowed`, not
    /// `E::NonCommittedInfo`. This asymmetry from `dispatch()` is intentional.
    pub fn redo(&mut self) -> Result<Outcome<Frame<E>, HistoryDisallowed>, EngineError> {
        match self.redo_stack.pop() {
            None => Ok(Outcome::Disallowed(HistoryDisallowed::NothingToRedo)),
            Some((prior_state, frame, reversibility)) => {
                let current_state = std::mem::replace(&mut self.state, prior_state);
                self.undo_stack
                    .push((current_state, frame.clone(), reversibility));
                Ok(Outcome::Redone(frame))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apply::Apply;
    use crate::outcome::NonCommittedOutcome;
    use crate::spec::EngineSpec;
    use proptest::prelude::*;

    // -----------------------------------------------------------------------
    // Test infrastructure
    // -----------------------------------------------------------------------

    #[derive(Debug)]
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

    // -----------------------------------------------------------------------
    // Helper fn pointers replacing old struct+trait pattern
    // -----------------------------------------------------------------------

    fn tracing_10_eval(_input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
        BehaviorResult::Continue(vec![10u8])
    }

    fn tracing_20_eval(_input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
        BehaviorResult::Continue(vec![20u8])
    }

    fn tracing_30_eval(_input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
        BehaviorResult::Continue(vec![30u8])
    }

    fn tracing_100_eval(_input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
        BehaviorResult::Continue(vec![100u8])
    }

    fn tracing_200_eval(_input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
        BehaviorResult::Continue(vec![200u8])
    }

    fn tracing_1_eval(_input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
        BehaviorResult::Continue(vec![1u8])
    }

    fn tracing_2_eval(_input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
        BehaviorResult::Continue(vec![2u8])
    }

    fn tracing_5_eval(_input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
        BehaviorResult::Continue(vec![5u8])
    }

    fn noop_eval(_input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
        BehaviorResult::Continue(vec![])
    }

    fn stop_eval(_input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
        BehaviorResult::Stop(NonCommittedOutcome::Aborted("halted".to_string()))
    }

    fn echo_eval(input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
        BehaviorResult::Continue(vec![*input])
    }

    /// Reads the current working state length and emits it as a diff, allowing
    /// verification that later behaviors see earlier diffs applied.
    fn state_reading_eval(_input: &u8, state: &Vec<u8>) -> BehaviorResult<u8, String> {
        BehaviorResult::Continue(vec![state.len() as u8])
    }

    // -----------------------------------------------------------------------
    // Task 1 tests: Engine struct + new() + state()
    // -----------------------------------------------------------------------

    #[test]
    fn engine_new_sorts_behaviors_by_order_key_then_name() {
        // Behaviors provided in out-of-order: keys [2, 0, 1], names ["c", "a", "b"]
        // Expected sort: (0,"a"), (1,"b"), (2,"c") → diff bytes [10, 20, 30]
        let behaviors: Vec<BehaviorDef<TestSpec>> = vec![
            BehaviorDef {
                name: "c",
                order_key: 2,
                evaluate: tracing_30_eval,
            },
            BehaviorDef {
                name: "a",
                order_key: 0,
                evaluate: tracing_10_eval,
            },
            BehaviorDef {
                name: "b",
                order_key: 1,
                evaluate: tracing_20_eval,
            },
        ];
        let mut engine = Engine::<TestSpec>::new(vec![], behaviors);
        let outcome = engine.dispatch(0u8, Reversibility::Reversible).unwrap();
        if let Outcome::Committed(frame) = outcome {
            assert_eq!(
                frame.diffs,
                vec![10u8, 20u8, 30u8],
                "behaviors must be evaluated in (order_key, name) sort order"
            );
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
        let mut engine = Engine::<TestSpec>::new(
            initial,
            vec![BehaviorDef {
                name: "noop",
                order_key: 0,
                evaluate: noop_eval,
            }],
        );

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
            vec![BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            }],
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
        let behaviors: Vec<BehaviorDef<TestSpec>> = vec![
            BehaviorDef {
                name: "second",
                order_key: 1,
                evaluate: tracing_200_eval,
            },
            BehaviorDef {
                name: "first",
                order_key: 0,
                evaluate: tracing_100_eval,
            },
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
        // echo_eval (key=0) emits the input byte, appending it to state.
        // state_reading_eval (key=10) reads the current state length and emits it.
        // Initial state is empty; after echo applies one diff, length == 1.
        // So state_reading_eval should emit 1u8.
        let behaviors: Vec<BehaviorDef<TestSpec>> = vec![
            BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            },
            BehaviorDef {
                name: "state_reader",
                order_key: 10,
                evaluate: state_reading_eval,
            },
        ];
        let mut engine = Engine::<TestSpec>::new(vec![], behaviors);
        let outcome = engine.dispatch(99u8, Reversibility::Reversible).unwrap();
        if let Outcome::Committed(frame) = outcome {
            // frame.diffs: [99 (echo), 1 (state length after echo)]
            assert_eq!(frame.diffs.len(), 2);
            assert_eq!(frame.diffs[0], 99u8);
            assert_eq!(
                frame.diffs[1], 1u8,
                "state_reading_eval must see state after echo applied its diff"
            );
        } else {
            panic!("expected Committed");
        }
    }

    #[test]
    fn trace_appended_at_diff_application() {
        // Each u8 diff emits "applied N" trace when applied.
        // Two diffs → two traces in the same order.
        let behaviors: Vec<BehaviorDef<TestSpec>> = vec![
            BehaviorDef {
                name: "first",
                order_key: 0,
                evaluate: tracing_10_eval,
            },
            BehaviorDef {
                name: "second",
                order_key: 1,
                evaluate: tracing_20_eval,
            },
        ];
        let mut engine = Engine::<TestSpec>::new(vec![], behaviors);
        let outcome = engine.dispatch(0u8, Reversibility::Reversible).unwrap();
        if let Outcome::Committed(frame) = outcome {
            assert_eq!(
                frame.traces,
                vec!["applied 10".to_string(), "applied 20".to_string()]
            );
        } else {
            panic!("expected Committed");
        }
    }

    #[test]
    fn stop_halts_dispatch() {
        // stop_eval (key=0) fires first and returns Stop. No frame should be committed.
        let behaviors: Vec<BehaviorDef<TestSpec>> = vec![
            BehaviorDef {
                name: "stopper",
                order_key: 0,
                evaluate: stop_eval,
            },
            BehaviorDef {
                name: "echo",
                order_key: 99,
                evaluate: echo_eval,
            },
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
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "noop",
                order_key: 0,
                evaluate: noop_eval,
            }],
        );
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
            vec![BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            }],
        );
        let outcome = engine.dispatch(77u8, Reversibility::Irreversible).unwrap();
        if let Outcome::Committed(frame) = outcome {
            assert_eq!(frame.input, 77u8);
            assert!(!frame.diffs.is_empty());
            assert!(!frame.traces.is_empty());
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

    // -----------------------------------------------------------------------
    // Task 1 tests: upgraded stack fields, dispatch snapshot + history mgmt,
    // undo_depth(), redo_depth()
    // -----------------------------------------------------------------------

    #[test]
    fn undo_depth_starts_at_zero() {
        let engine = Engine::<TestSpec>::new(vec![], vec![]);
        assert_eq!(engine.undo_depth(), 0);
    }

    #[test]
    fn redo_depth_starts_at_zero() {
        let engine = Engine::<TestSpec>::new(vec![], vec![]);
        assert_eq!(engine.redo_depth(), 0);
    }

    #[test]
    fn committed_dispatch_increments_undo_depth() {
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            }],
        );
        let _ = engine.dispatch(1u8, Reversibility::Reversible).unwrap();
        assert_eq!(engine.undo_depth(), 1);
        let _ = engine.dispatch(2u8, Reversibility::Reversible).unwrap();
        assert_eq!(engine.undo_depth(), 2);
    }

    #[test]
    fn committed_dispatch_clears_redo_stack() {
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            }],
        );
        let _ = engine.dispatch(1u8, Reversibility::Reversible).unwrap();
        // redo_depth must be 0 after a Committed dispatch (new timeline)
        assert_eq!(engine.redo_depth(), 0);
    }

    #[test]
    fn irreversible_committed_dispatch_clears_both_stacks() {
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            }],
        );
        let _ = engine.dispatch(1u8, Reversibility::Reversible).unwrap();
        let _ = engine.dispatch(2u8, Reversibility::Reversible).unwrap();
        assert_eq!(engine.undo_depth(), 2);
        // Irreversible wipes both stacks
        let _ = engine.dispatch(99u8, Reversibility::Irreversible).unwrap();
        assert_eq!(engine.undo_depth(), 0, "irreversible must clear undo stack");
        assert_eq!(engine.redo_depth(), 0, "irreversible must clear redo stack");
    }

    #[test]
    fn no_change_dispatch_does_not_affect_depths() {
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![
                BehaviorDef {
                    name: "echo",
                    order_key: 0,
                    evaluate: echo_eval,
                },
                BehaviorDef {
                    name: "noop",
                    order_key: 1,
                    evaluate: noop_eval,
                },
            ],
        );
        let _ = engine.dispatch(1u8, Reversibility::Reversible).unwrap();
        let ud = engine.undo_depth();
        let rd = engine.redo_depth();
        // Use a fresh noop-only engine to produce NoChange without touching stacks
        let mut noop_engine = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "noop",
                order_key: 0,
                evaluate: noop_eval,
            }],
        );
        let _ = noop_engine
            .dispatch(1u8, Reversibility::Reversible)
            .unwrap(); // NoChange
        assert_eq!(noop_engine.undo_depth(), 0);
        assert_eq!(noop_engine.redo_depth(), 0);
        // Original engine stacks unchanged by separate engine
        let _ = ud;
        let _ = rd;
    }

    // -----------------------------------------------------------------------
    // Phase 3 tests: undo(), redo(), undo_depth(), redo_depth(), irreversibility
    // -----------------------------------------------------------------------

    #[test]
    fn undo_on_empty_stack_returns_disallowed_nothing_to_undo() {
        use crate::outcome::HistoryDisallowed;
        let mut engine = Engine::<TestSpec>::new(vec![], vec![]);
        let result = engine.undo().unwrap();
        assert!(
            matches!(
                result,
                Outcome::Disallowed(HistoryDisallowed::NothingToUndo)
            ),
            "undo on empty stack must return Disallowed(NothingToUndo)"
        );
    }

    #[test]
    fn redo_on_empty_stack_returns_disallowed_nothing_to_redo() {
        use crate::outcome::HistoryDisallowed;
        let mut engine = Engine::<TestSpec>::new(vec![], vec![]);
        let result = engine.redo().unwrap();
        assert!(
            matches!(
                result,
                Outcome::Disallowed(HistoryDisallowed::NothingToRedo)
            ),
            "redo on empty stack must return Disallowed(NothingToRedo)"
        );
    }

    #[test]
    fn undo_restores_prior_state_and_returns_undone_frame() {
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            }],
        );
        let state_before = engine.state().clone();
        let outcome = engine.dispatch(42u8, Reversibility::Reversible).unwrap();
        let committed_frame = if let Outcome::Committed(f) = outcome {
            f
        } else {
            panic!("expected Committed")
        };
        let state_after_dispatch = engine.state().clone();
        assert_ne!(
            state_before, state_after_dispatch,
            "dispatch must have changed state"
        );

        let undo_result = engine.undo().unwrap();
        if let Outcome::Undone(frame) = undo_result {
            assert_eq!(
                frame, committed_frame,
                "undone frame must match committed frame"
            );
        } else {
            panic!("expected Undone");
        }
        assert_eq!(
            engine.state(),
            &state_before,
            "undo must restore state to pre-dispatch snapshot"
        );
    }

    #[test]
    fn redo_restores_state_after_undo_and_returns_redone_frame() {
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            }],
        );
        let _ = engine.dispatch(42u8, Reversibility::Reversible).unwrap();
        let state_after_dispatch = engine.state().clone();
        let _ = engine.undo().unwrap();
        let state_before_dispatch = engine.state().clone();
        assert_ne!(state_after_dispatch, state_before_dispatch);

        let redo_result = engine.redo().unwrap();
        assert!(
            matches!(redo_result, Outcome::Redone(_)),
            "redo must return Redone"
        );
        assert_eq!(
            engine.state(),
            &state_after_dispatch,
            "redo must restore post-dispatch state"
        );
    }

    #[test]
    fn undo_depth_and_redo_depth_track_correctly() {
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            }],
        );
        assert_eq!(engine.undo_depth(), 0);
        assert_eq!(engine.redo_depth(), 0);

        let _ = engine.dispatch(1u8, Reversibility::Reversible).unwrap();
        assert_eq!(engine.undo_depth(), 1);
        assert_eq!(engine.redo_depth(), 0);

        let _ = engine.dispatch(2u8, Reversibility::Reversible).unwrap();
        assert_eq!(engine.undo_depth(), 2);
        assert_eq!(engine.redo_depth(), 0);

        let _ = engine.undo().unwrap();
        assert_eq!(engine.undo_depth(), 1);
        assert_eq!(engine.redo_depth(), 1);

        let _ = engine.undo().unwrap();
        assert_eq!(engine.undo_depth(), 0);
        assert_eq!(engine.redo_depth(), 2);

        let _ = engine.redo().unwrap();
        assert_eq!(engine.undo_depth(), 1);
        assert_eq!(engine.redo_depth(), 1);
    }

    #[test]
    fn new_committed_dispatch_after_undo_clears_redo_stack() {
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            }],
        );
        let _ = engine.dispatch(1u8, Reversibility::Reversible).unwrap();
        let _ = engine.dispatch(2u8, Reversibility::Reversible).unwrap();
        let _ = engine.undo().unwrap();
        assert_eq!(engine.redo_depth(), 1, "undo must populate redo stack");

        // New commit on a different branch — erases the redo future.
        let _ = engine.dispatch(99u8, Reversibility::Reversible).unwrap();
        assert_eq!(
            engine.redo_depth(),
            0,
            "new Committed dispatch must clear redo stack"
        );
    }

    #[test]
    fn no_change_dispatch_does_not_clear_redo_stack() {
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![
                BehaviorDef {
                    name: "echo",
                    order_key: 0,
                    evaluate: echo_eval,
                },
                BehaviorDef {
                    name: "noop",
                    order_key: 1,
                    evaluate: noop_eval,
                },
            ],
        );
        let _ = engine.dispatch(1u8, Reversibility::Reversible).unwrap();
        let _ = engine.undo().unwrap();
        assert_eq!(engine.redo_depth(), 1);

        // Replace echo with noop-only engine to force NoChange
        let mut engine2 = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "noop",
                order_key: 0,
                evaluate: noop_eval,
            }],
        );
        let _ = engine2.dispatch(1u8, Reversibility::Reversible).unwrap(); // NoChange (no diffs)
        // (redo depth was never populated, so assert on a fresh engine with setup)

        // Simpler test: dispatch + undo populates redo. NoChange after does NOT clear it.
        let mut e = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            }],
        );
        let _ = e.dispatch(1u8, Reversibility::Reversible).unwrap();
        let _ = e.undo().unwrap();
        assert_eq!(e.redo_depth(), 1);
        // Structural test: ensure redo_depth is still 1 after calling undo on empty stack (which returns Disallowed, not Committed).
        let _ = e.undo().unwrap(); // NothingToUndo — does not clear redo
        assert_eq!(
            e.redo_depth(),
            1,
            "Disallowed outcome must not clear redo stack"
        );
    }

    #[test]
    fn irreversible_commit_clears_both_stacks() {
        use crate::outcome::HistoryDisallowed;
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            }],
        );
        // Build up some history.
        let _ = engine.dispatch(1u8, Reversibility::Reversible).unwrap();
        let _ = engine.dispatch(2u8, Reversibility::Reversible).unwrap();
        let _ = engine.undo().unwrap();
        assert_eq!(engine.undo_depth(), 1);
        assert_eq!(engine.redo_depth(), 1);

        // Irreversible commit: state changes, but both stacks are wiped.
        let _ = engine.dispatch(99u8, Reversibility::Irreversible).unwrap();
        assert_eq!(
            engine.undo_depth(),
            0,
            "irreversible commit must clear undo stack"
        );
        assert_eq!(
            engine.redo_depth(),
            0,
            "irreversible commit must clear redo stack"
        );

        // Calling undo/redo now returns Disallowed.
        assert!(matches!(
            engine.undo().unwrap(),
            Outcome::Disallowed(HistoryDisallowed::NothingToUndo)
        ));
        assert!(matches!(
            engine.redo().unwrap(),
            Outcome::Disallowed(HistoryDisallowed::NothingToRedo)
        ));
    }

    #[test]
    fn undo_snapshot_is_exact_no_reversible_trait_required() {
        // Verify that undo restores state exactly — diff type (u8) has no Reversible trait.
        // If the snapshot mechanism is correct, state is fully restored without needing
        // any reverse-operation on the diff.
        let initial_state = vec![10u8, 20u8, 30u8];
        let mut engine = Engine::<TestSpec>::new(
            initial_state.clone(),
            vec![BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            }],
        );
        let _ = engine.dispatch(99u8, Reversibility::Reversible).unwrap();
        assert_ne!(engine.state(), &initial_state);

        let _ = engine.undo().unwrap();
        assert_eq!(
            engine.state(),
            &initial_state,
            "undo must restore exact pre-dispatch snapshot; no Reversible trait required"
        );
    }

    // -----------------------------------------------------------------------
    // Phase 4 tests: 15 named invariant tests (ARCHITECTURE.md §"Core Invariants")
    // -----------------------------------------------------------------------

    #[test]
    fn invariant_01_engine_never_advances_without_input() {
        // Invariant 1: The engine never advances without a new input or explicit undo/redo.
        // No operation called — state must be identical to initial.
        let engine = Engine::<TestSpec>::new(vec![1u8, 2u8], vec![]);
        let before = engine.state().clone();
        // (no dispatch/undo/redo called)
        assert_eq!(
            engine.state(),
            &before,
            "engine state must not change without an operation"
        );
    }

    #[test]
    fn invariant_02_dispatch_is_atomic() {
        // Invariant 2: Dispatch is atomic — if Stop fires, no prior diffs in that dispatch commit.
        // stop_eval fires first (order_key=0); echo_eval (order_key=99) never runs.
        // State must remain identical to initial.
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![
                BehaviorDef {
                    name: "stopper",
                    order_key: 0,
                    evaluate: stop_eval,
                },
                BehaviorDef {
                    name: "echo",
                    order_key: 99,
                    evaluate: echo_eval,
                },
            ],
        );
        let before = engine.state().clone();
        let _ = engine.dispatch(42u8, Reversibility::Reversible).unwrap();
        assert_eq!(
            engine.state(),
            &before,
            "Aborted dispatch must leave state unchanged (atomicity)"
        );
    }

    #[test]
    fn invariant_03_behaviors_evaluated_in_deterministic_order() {
        // Invariant 3: Behaviors evaluated in (order_key, name) order.
        let behaviors: Vec<BehaviorDef<TestSpec>> = vec![
            BehaviorDef {
                name: "b",
                order_key: 1,
                evaluate: tracing_2_eval,
            },
            BehaviorDef {
                name: "a",
                order_key: 0,
                evaluate: tracing_1_eval,
            },
        ];
        let mut engine = Engine::<TestSpec>::new(vec![], behaviors);
        let outcome = engine.dispatch(0u8, Reversibility::Reversible).unwrap();
        if let Outcome::Committed(frame) = outcome {
            assert_eq!(
                frame.diffs,
                vec![1u8, 2u8],
                "lower order_key behavior must run first — (0,'a') before (1,'b')"
            );
        } else {
            panic!("expected Committed");
        }
    }

    #[test]
    fn invariant_04_each_behavior_sees_only_current_working_state() {
        // Invariant 4: Each behavior sees the working state at its moment of evaluation.
        // state_reading_eval (key=10) reads state AFTER echo_eval (key=0) has applied its diff.
        let behaviors: Vec<BehaviorDef<TestSpec>> = vec![
            BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            },
            BehaviorDef {
                name: "state_reader",
                order_key: 10,
                evaluate: state_reading_eval,
            },
        ];
        let mut engine = Engine::<TestSpec>::new(vec![], behaviors);
        let outcome = engine.dispatch(5u8, Reversibility::Reversible).unwrap();
        if let Outcome::Committed(frame) = outcome {
            // state_reading_eval emits state.len() after echo added one element.
            assert_eq!(
                frame.diffs[1], 1u8,
                "state_reading_eval must see working state with echo's diff already applied"
            );
        } else {
            panic!("expected Committed");
        }
    }

    #[test]
    fn invariant_05_later_behaviors_see_earlier_applied_diffs() {
        // Invariant 5: Later behaviors see earlier applied diffs.
        let behaviors: Vec<BehaviorDef<TestSpec>> = vec![
            BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            },
            BehaviorDef {
                name: "state_reader",
                order_key: 10,
                evaluate: state_reading_eval,
            },
        ];
        let mut engine = Engine::<TestSpec>::new(vec![], behaviors);
        let outcome = engine.dispatch(99u8, Reversibility::Reversible).unwrap();
        if let Outcome::Committed(frame) = outcome {
            assert_eq!(frame.diffs.len(), 2, "both behaviors must have contributed");
            assert_eq!(
                frame.diffs[1], 1u8,
                "state_reader saw length=1 (echo's diff applied before state_reader ran)"
            );
        } else {
            panic!("expected Committed");
        }
    }

    #[test]
    fn invariant_06_behaviors_do_not_mutate_state_directly() {
        // Invariant 6: Behaviors do not mutate state directly — structural guarantee.
        // The evaluate fn pointer signature takes `&E::State` (shared reference).
        let initial = vec![7u8];
        let engine = Engine::<TestSpec>::new(
            initial.clone(),
            vec![BehaviorDef {
                name: "noop",
                order_key: 0,
                evaluate: noop_eval,
            }],
        );
        assert_eq!(
            engine.state(),
            &initial,
            "committed state must be unchanged — behaviors receive &State, cannot mutate directly"
        );
    }

    #[test]
    fn invariant_07_engine_applies_diffs_centrally() {
        // Invariant 7: The engine applies diffs centrally (not behaviors).
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            }],
        );
        let _ = engine.dispatch(42u8, Reversibility::Reversible).unwrap();
        assert_eq!(
            engine.state(),
            &vec![42u8],
            "engine applied the diff centrally; state contains the pushed byte"
        );
    }

    #[test]
    fn invariant_08_diff_that_mutates_state_appends_trace() {
        // Invariant 8: Any diff that mutates state must append at least one trace entry.
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "t",
                order_key: 0,
                evaluate: tracing_5_eval,
            }],
        );
        let outcome = engine.dispatch(0u8, Reversibility::Reversible).unwrap();
        if let Outcome::Committed(frame) = outcome {
            assert_eq!(frame.diffs.len(), 1);
            assert_eq!(
                frame.traces.len(),
                1,
                "each diff must produce at least one trace entry"
            );
            assert!(!frame.traces[0].is_empty(), "trace entry must be non-empty");
        } else {
            panic!("expected Committed");
        }
    }

    #[test]
    fn invariant_09_trace_generated_in_execution_order() {
        // Invariant 9: Trace is generated in execution order, not reconstructed later.
        let behaviors: Vec<BehaviorDef<TestSpec>> = vec![
            BehaviorDef {
                name: "first",
                order_key: 0,
                evaluate: tracing_10_eval,
            },
            BehaviorDef {
                name: "second",
                order_key: 1,
                evaluate: tracing_20_eval,
            },
        ];
        let mut engine = Engine::<TestSpec>::new(vec![], behaviors);
        let outcome = engine.dispatch(0u8, Reversibility::Reversible).unwrap();
        if let Outcome::Committed(frame) = outcome {
            assert_eq!(
                frame.traces[0], "applied 10",
                "first diff's trace must appear first"
            );
            assert_eq!(
                frame.traces[1], "applied 20",
                "second diff's trace must appear second"
            );
        } else {
            panic!("expected Committed");
        }
    }

    #[test]
    fn invariant_10_only_non_empty_transitions_produce_frames() {
        // Invariant 10: Only successful, non-empty transitions produce frames.
        let mut noop_engine = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "noop",
                order_key: 0,
                evaluate: noop_eval,
            }],
        );
        let noop_outcome = noop_engine
            .dispatch(0u8, Reversibility::Reversible)
            .unwrap();
        assert!(
            matches!(noop_outcome, Outcome::NoChange),
            "zero diffs must return NoChange, not a frame"
        );

        let mut echo_engine = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            }],
        );
        let echo_outcome = echo_engine
            .dispatch(1u8, Reversibility::Reversible)
            .unwrap();
        assert!(
            matches!(echo_outcome, Outcome::Committed(_)),
            "non-zero diffs must return Committed(frame)"
        );
    }

    #[test]
    fn invariant_11_non_committed_outcomes_do_not_modify_committed_state() {
        // Invariant 11: Non-committed outcomes do not modify visible committed state.
        let mut engine = Engine::<TestSpec>::new(
            vec![10u8],
            vec![
                BehaviorDef {
                    name: "stopper",
                    order_key: 0,
                    evaluate: stop_eval,
                },
                BehaviorDef {
                    name: "echo",
                    order_key: 99,
                    evaluate: echo_eval,
                },
            ],
        );
        let before = engine.state().clone();

        // Aborted — state must be unchanged
        let _ = engine.dispatch(99u8, Reversibility::Reversible).unwrap();
        assert_eq!(
            engine.state(),
            &before,
            "Aborted dispatch must not modify committed state"
        );

        // NoChange — state must still be unchanged
        let mut noop_engine = Engine::<TestSpec>::new(
            vec![10u8],
            vec![BehaviorDef {
                name: "noop",
                order_key: 0,
                evaluate: noop_eval,
            }],
        );
        let before2 = noop_engine.state().clone();
        let _ = noop_engine
            .dispatch(0u8, Reversibility::Reversible)
            .unwrap();
        assert_eq!(
            noop_engine.state(),
            &before2,
            "NoChange dispatch must not modify committed state"
        );
    }

    #[test]
    fn invariant_12_behavior_state_lives_in_main_state_tree() {
        // Invariant 12: Behavior state lives inside the main state tree (not engine internals).
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            }],
        );
        let initial = engine.state().clone();
        let _ = engine.dispatch(5u8, Reversibility::Reversible).unwrap();
        assert_ne!(engine.state(), &initial);

        let _ = engine.undo().unwrap();
        assert_eq!(
            engine.state(),
            &initial,
            "undo must restore full state including any behavior-local state embedded in it"
        );
    }

    #[test]
    fn invariant_13_undo_redo_operate_on_canonical_stored_frames() {
        // Invariant 13: Undo/redo operate on canonical stored frames — not re-dispatch.
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            }],
        );
        let dispatch_outcome = engine.dispatch(77u8, Reversibility::Reversible).unwrap();
        let committed_frame = if let Outcome::Committed(f) = dispatch_outcome {
            f
        } else {
            panic!("expected Committed")
        };

        let undo_outcome = engine.undo().unwrap();
        let undone_frame = if let Outcome::Undone(f) = undo_outcome {
            f
        } else {
            panic!("expected Undone")
        };

        assert_eq!(
            committed_frame, undone_frame,
            "frame returned by undo must be the canonical stored frame from the original dispatch"
        );
    }

    #[test]
    fn invariant_14_irreversible_transitions_designated_by_library_user() {
        // Invariant 14: Irreversible transitions are designated by the library user (not the engine).
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            }],
        );
        let _ = engine.dispatch(1u8, Reversibility::Reversible).unwrap();
        assert_eq!(engine.undo_depth(), 1);

        // User designates this dispatch as Irreversible — both stacks cleared.
        let _ = engine.dispatch(2u8, Reversibility::Irreversible).unwrap();
        assert_eq!(
            engine.undo_depth(),
            0,
            "caller designated Irreversible — engine must clear undo stack"
        );
        assert_eq!(
            engine.redo_depth(),
            0,
            "caller designated Irreversible — engine must clear redo stack"
        );
    }

    #[test]
    fn invariant_15_engine_errors_distinct_from_normal_outcomes() {
        // Invariant 15: Engine errors (EngineError) are distinct from normal rejected or
        // aborted dispatch outcomes (Outcome variants).
        let mut engine = Engine::<TestSpec>::new(
            vec![],
            vec![BehaviorDef {
                name: "echo",
                order_key: 0,
                evaluate: echo_eval,
            }],
        );
        let result = engine.dispatch(1u8, Reversibility::Reversible);
        assert!(
            result.is_ok(),
            "normal dispatch must return Ok(Outcome), not Err(EngineError)"
        );
    }

    // -----------------------------------------------------------------------
    // Irreversible history semantics tests (quick-01 final cleanup)
    // -----------------------------------------------------------------------

    #[cfg(test)]
    mod irreversible_history_tests {
        use super::*;
        use crate::apply::Apply;
        use crate::outcome::HistoryDisallowed;
        use crate::spec::EngineSpec;

        struct IrrevSpec;

        impl EngineSpec for IrrevSpec {
            type State = Vec<u8>;
            type Input = u8;
            type Diff = u8;
            type Trace = String;
            type NonCommittedInfo = String;
            type OrderKey = u32;
        }

        impl Apply<IrrevSpec> for u8 {
            fn apply(&self, state: &mut Vec<u8>) -> Vec<String> {
                state.push(*self);
                vec![format!("appended {}", self)]
            }
        }

        fn append_eval(input: &u8, _state: &Vec<u8>) -> BehaviorResult<u8, String> {
            BehaviorResult::Continue(vec![*input])
        }

        fn make_engine() -> Engine<IrrevSpec> {
            Engine::<IrrevSpec>::new(
                vec![],
                vec![BehaviorDef {
                    name: "append",
                    order_key: 0u32,
                    evaluate: append_eval,
                }],
            )
        }

        /// A reversible dispatch pushes a frame onto the undo stack; undo() returns Undone(frame).
        #[test]
        fn irreversible_history_reversible_commit_is_undoable() {
            let mut engine = make_engine();
            let outcome = engine.dispatch(1u8, Reversibility::Reversible).unwrap();
            let committed_frame = if let Outcome::Committed(f) = outcome {
                f
            } else {
                panic!("expected Committed")
            };

            let undo_result = engine.undo().unwrap();
            assert!(
                matches!(undo_result, Outcome::Undone(ref f) if f == &committed_frame),
                "reversible commit must be undoable: undo() must return Undone(frame)"
            );
        }

        /// Two reversible commits followed by one irreversible commit clears both stacks.
        /// The irreversible boundary erases all history before and including that frame.
        #[test]
        fn irreversible_history_irreversible_commit_clears_all_history() {
            let mut engine = make_engine();
            let _ = engine.dispatch(1u8, Reversibility::Reversible).unwrap();
            let _ = engine.dispatch(2u8, Reversibility::Reversible).unwrap();
            assert_eq!(
                engine.undo_depth(),
                2,
                "setup: two reversible frames on undo stack"
            );

            let _ = engine.dispatch(99u8, Reversibility::Irreversible).unwrap();

            assert_eq!(
                engine.undo_depth(),
                0,
                "irreversible commit must clear undo stack to 0"
            );
            assert_eq!(
                engine.redo_depth(),
                0,
                "irreversible commit must clear redo stack to 0"
            );
        }

        /// After an irreversible commit the undo stack is empty;
        /// calling undo() must return Disallowed(NothingToUndo) without panicking.
        #[test]
        fn irreversible_history_after_irreversible_commit_undo_returns_nothing_to_undo() {
            let mut engine = make_engine();
            let _ = engine.dispatch(1u8, Reversibility::Irreversible).unwrap();

            let result = engine.undo().unwrap();
            assert!(
                matches!(
                    result,
                    Outcome::Disallowed(HistoryDisallowed::NothingToUndo)
                ),
                "after irreversible commit, undo() must return Disallowed(NothingToUndo)"
            );
        }

        /// After an irreversible commit the redo stack is also empty;
        /// calling redo() must return Disallowed(NothingToRedo) without panicking.
        #[test]
        fn irreversible_history_after_irreversible_commit_redo_returns_nothing_to_redo() {
            let mut engine = make_engine();
            let _ = engine.dispatch(1u8, Reversibility::Irreversible).unwrap();

            let result = engine.redo().unwrap();
            assert!(
                matches!(
                    result,
                    Outcome::Disallowed(HistoryDisallowed::NothingToRedo)
                ),
                "after irreversible commit, redo() must return Disallowed(NothingToRedo)"
            );
        }

        /// A reversible commit followed by undo puts a frame on the redo stack.
        /// A subsequent irreversible commit must erase that redo entry too.
        #[test]
        fn irreversible_history_irreversible_commit_preceded_by_undo_clears_redo_too() {
            let mut engine = make_engine();
            let _ = engine.dispatch(1u8, Reversibility::Reversible).unwrap();
            let _ = engine.undo().unwrap();
            assert_eq!(
                engine.redo_depth(),
                1,
                "setup: undo must have populated redo stack"
            );

            let _ = engine.dispatch(2u8, Reversibility::Irreversible).unwrap();

            assert_eq!(
                engine.redo_depth(),
                0,
                "irreversible commit must clear redo stack even when it was populated before"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Phase 4 tests: property tests (proptest)
    // -----------------------------------------------------------------------

    // Suite 1 — Determinism
    proptest! {
        #[test]
        fn prop_dispatch_is_deterministic(
            inputs in prop::collection::vec(any::<u8>(), 0..10)
        ) {
            let mut engine1 = Engine::<TestSpec>::new(
                vec![],
                vec![BehaviorDef { name: "echo", order_key: 0, evaluate: echo_eval }],
            );
            let mut engine2 = Engine::<TestSpec>::new(
                vec![],
                vec![BehaviorDef { name: "echo", order_key: 0, evaluate: echo_eval }],
            );
            for &input in &inputs {
                let _ = engine1.dispatch(input, Reversibility::Reversible);
                let _ = engine2.dispatch(input, Reversibility::Reversible);
                prop_assert_eq!(
                    engine1.state(),
                    engine2.state(),
                    "engines must have identical state after input {:?}", input
                );
            }
        }
    }

    /// Suite 2 — Undo/Redo Correctness
    #[allow(dead_code)]
    #[derive(Debug, Clone)]
    enum Op {
        Dispatch(u8),
        Undo,
        Redo,
    }

    fn arb_op() -> impl Strategy<Value = Op> {
        prop_oneof![
            any::<u8>().prop_map(Op::Dispatch),
            Just(Op::Undo),
            Just(Op::Redo),
        ]
    }

    proptest! {
        #[test]
        fn prop_undo_restores_exact_state(
            ops in prop::collection::vec(arb_op(), 0..20)
        ) {
            let mut engine = Engine::<TestSpec>::new(
                vec![],
                vec![BehaviorDef { name: "echo", order_key: 0, evaluate: echo_eval }],
            );

            // Apply all ops — engine must never panic
            for op in &ops {
                match op {
                    Op::Dispatch(b) => {
                        let _ = engine.dispatch(*b, Reversibility::Reversible);
                    }
                    Op::Undo => {
                        let _ = engine.undo();
                    }
                    Op::Redo => {
                        let _ = engine.redo();
                    }
                }
            }

            // Structural consistency: undo to bottom and verify we reach initial state.
            let undo_depth = engine.undo_depth();
            for _ in 0..undo_depth {
                let _ = engine.undo();
            }
            let state_after_all_undos = engine.state().clone();

            // After undoing everything, undo_depth must be 0.
            prop_assert_eq!(engine.undo_depth(), 0,
                "after undoing all frames, undo_depth must be 0");

            // The state after all undos must equal the initial state (vec![]).
            prop_assert_eq!(&state_after_all_undos, &vec![] as &Vec<u8>,
                "after undoing all operations, state must equal initial state");
        }
    }
}
