//! Outcome types for the HerdingCats dispatch pipeline.
//!
//! Every call to `Engine::dispatch`, `Engine::undo`, or `Engine::redo` returns
//! an [`Outcome`]. The `F` type parameter is typically [`Frame<E>`] — the
//! canonical committed record of a single transition — and `N` is
//! `E::NonCommittedInfo`, the payload for outcomes that did not touch the
//! history stack.
//!
//! [`EngineError`] is a separate type used for unrecoverable engine-level
//! failures; it is distinct from `Outcome` and returned via `Result<Outcome, EngineError>`.

use crate::spec::EngineSpec;

/// The canonical committed record of a single dispatch transition.
///
/// A `Frame<E>` is produced exactly once per successful `dispatch` call and
/// stored in the history stack. It bundles the originating input, the aggregate
/// diff describing the state change, and any per-dispatch trace metadata.
///
/// `F = Frame<E>` is the conventional type argument for the frame-carrying
/// [`Outcome`] variants (`Committed`, `Undone`, `Redone`).
#[derive(Debug, Clone, PartialEq)]
pub struct Frame<E: EngineSpec> {
    /// The input that triggered this dispatch.
    pub input: E::Input,
    /// The aggregate diff describing how state changed.
    pub diff: E::Diff,
    /// Per-dispatch trace metadata (e.g. annotations, timing, debug info).
    pub trace: E::Trace,
}

/// The result of a single `dispatch`, `undo`, or `redo` call.
///
/// # Type parameters
///
/// - `F` — the frame type. Conventionally `Frame<E>` for the concrete engine.
/// - `N` — the non-committed info type. Conventionally `E::NonCommittedInfo`.
///
/// All 7 variants are a stable public contract; this enum is **not**
/// `#[non_exhaustive]`. Downstream code may match exhaustively.
///
/// The `#[must_use]` attribute ensures callers do not silently discard outcomes.
#[must_use = "dispatch outcomes must be handled"]
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome<F, N> {
    /// Dispatch succeeded. The frame has been committed to the history stack.
    Committed(F),

    /// Undo succeeded. The frame describes the transition that was undone.
    Undone(F),

    /// Redo succeeded. The frame describes the transition that was redone.
    Redone(F),

    /// Dispatch produced no diffs from any behavior. State is unchanged;
    /// nothing was committed.
    NoChange,

    /// No behavior accepted the input; it is semantically invalid for the
    /// current state. The payload provides context for the caller.
    InvalidInput(N),

    /// A behavior explicitly rejected the input (e.g. rule violation, illegal
    /// move). The payload provides context for the caller.
    Disallowed(N),

    /// A behavior halted dispatch (e.g. a fatal precondition failed). No diffs
    /// were applied and nothing was committed. The payload provides context.
    Aborted(N),
}

/// Unrecoverable engine-level errors, distinct from normal [`Outcome`] variants.
///
/// `EngineError` is marked `#[non_exhaustive]`: future versions of the library
/// may add variants. Downstream code **must** include a wildcard arm:
///
/// ```
/// # use herdingcats::EngineError;
/// # let err = EngineError::BehaviorPanic;
/// match err {
///     EngineError::BehaviorPanic => { /* ... */ }
///     EngineError::InvalidState(msg) => { eprintln!("invalid state: {msg}"); }
///     EngineError::CorruptHistory => { /* ... */ }
///     _ => { /* future variants */ }
/// }
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum EngineError {
    /// A behavior panicked during evaluation. The engine caught the panic and
    /// reports it here instead of propagating it.
    BehaviorPanic,

    /// The engine detected internally inconsistent state. The string payload
    /// provides a diagnostic message.
    InvalidState(String),

    /// The undo/redo history stack is corrupted (e.g. mismatched snapshot
    /// indices). This indicates a bug in the engine itself.
    CorruptHistory,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::EngineSpec;

    #[derive(Debug, Clone, PartialEq)]
    struct TestSpec;

    impl EngineSpec for TestSpec {
        type State = Vec<u8>;
        type Input = u8;
        type Diff = u8;
        type Trace = String;
        type NonCommittedInfo = String;
        type OrderKey = u32;
    }

    fn make_frame() -> Frame<TestSpec> {
        Frame {
            input: 42u8,
            diff: 1u8,
            trace: "t".to_string(),
        }
    }

    #[test]
    fn frame_is_constructable_cloneable_and_eq() {
        let f = make_frame();
        let g = f.clone();
        assert_eq!(f, g);
        assert_eq!(f.input, 42u8);
        assert_eq!(f.diff, 1u8);
        assert_eq!(f.trace, "t");
    }

    #[test]
    fn all_7_outcome_variants_are_constructable_and_exhaustively_matchable() {
        use Outcome::*;

        let frame = make_frame();
        let nci = "info".to_string();

        let variants: Vec<Outcome<Frame<TestSpec>, String>> = vec![
            Committed(frame.clone()),
            Undone(frame.clone()),
            Redone(frame.clone()),
            NoChange,
            InvalidInput(nci.clone()),
            Disallowed(nci.clone()),
            Aborted(nci.clone()),
        ];

        // Exhaustive match — must compile with no wildcard arm
        for outcome in variants {
            let _label = match outcome {
                Committed(_) => "committed",
                Undone(_) => "undone",
                Redone(_) => "redone",
                NoChange => "no_change",
                InvalidInput(_) => "invalid_input",
                Disallowed(_) => "disallowed",
                Aborted(_) => "aborted",
            };
        }
    }

    #[test]
    fn outcome_committed_carries_frame() {
        let f = make_frame();
        let o: Outcome<Frame<TestSpec>, String> = Outcome::Committed(f.clone());
        if let Outcome::Committed(returned) = o {
            assert_eq!(returned, f);
        } else {
            panic!("expected Committed");
        }
    }

    #[test]
    fn outcome_no_change_has_no_payload() {
        let o: Outcome<Frame<TestSpec>, String> = Outcome::NoChange;
        assert!(matches!(o, Outcome::NoChange));
    }

    #[test]
    fn all_3_engine_error_variants_are_constructable() {
        let _e1 = EngineError::BehaviorPanic;
        let _e2 = EngineError::InvalidState("bad state".to_string());
        let _e3 = EngineError::CorruptHistory;

        // Exhaustive-ish match with wildcard required by #[non_exhaustive]
        let errs = vec![
            EngineError::BehaviorPanic,
            EngineError::InvalidState("msg".to_string()),
            EngineError::CorruptHistory,
        ];
        for err in errs {
            let _label = match err {
                EngineError::BehaviorPanic => "panic",
                EngineError::InvalidState(ref msg) => {
                    assert!(!msg.is_empty());
                    "invalid_state"
                }
                EngineError::CorruptHistory => "corrupt",
            };
        }
    }

    #[test]
    fn engine_error_and_outcome_are_distinct_types() {
        // Type-level check: these are different named types. This test exists
        // to document the invariant; the compiler enforces it.
        fn takes_outcome(_: Outcome<Frame<TestSpec>, String>) {}
        fn takes_error(_: EngineError) {}

        takes_outcome(Outcome::NoChange);
        takes_error(EngineError::BehaviorPanic);
    }
}
