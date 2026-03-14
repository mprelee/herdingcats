//! # Backgammon Irreversibility Demo
//!
//! Demonstrates the HerdingCats `Reversibility::Irreversible` dispatch mode using
//! a simplified backgammon scenario.
//!
//! ## What to look for in the output
//!
//! 1. Two `MovePiece` dispatches — both `Committed`, undo stack grows to depth 2.
//! 2. `undo()` after the second move — returns `Undone`, stack depth drops to 1.
//! 3. `RollDice` dispatch with `Reversibility::Irreversible` — commits successfully,
//!    but **clears BOTH undo and redo stacks**. undo_depth and redo_depth are both 0.
//! 4. `undo()` after the irreversible roll — returns `Disallowed(NothingToUndo)`.
//!    The dice roll permanently erased all history.
//!
//! Run with: cargo run --example backgammon

use herdingcats::{
    Apply, Behavior, BehaviorResult, Engine, EngineError, EngineSpec, Frame,
    HistoryDisallowed, Outcome, Reversibility,
};

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

struct BackgammonSpec;

#[derive(Debug, Clone, Default)]
struct BackgammonState {
    /// Current dice values (None if no roll yet).
    dice: Option<(u8, u8)>,
    /// White checker position (simplified: one piece, starts at 1).
    white_pos: u8,
    /// Black checker position (simplified: one piece, starts at 24).
    #[allow(dead_code)]
    black_pos: u8,
}

#[derive(Debug, Clone)]
enum BackgammonInput {
    RollDice { d1: u8, d2: u8 },
    MovePiece { from: u8, to: u8 },
}

#[derive(Debug, Clone)]
enum BackgammonDiff {
    SetDice(u8, u8),
    MoveWhite { from: u8, to: u8 },
}

// ---------------------------------------------------------------------------
// Apply impl — how each diff mutates state
// ---------------------------------------------------------------------------

impl Apply<BackgammonSpec> for BackgammonDiff {
    fn apply(&self, state: &mut BackgammonState) -> Vec<String> {
        match self {
            BackgammonDiff::SetDice(d1, d2) => {
                state.dice = Some((*d1, *d2));
                vec![format!("dice set to ({}, {})", d1, d2)]
            }
            BackgammonDiff::MoveWhite { from, to } => {
                state.white_pos = *to;
                vec![format!("white moved from {} to {}", from, to)]
            }
        }
    }
}

// ---------------------------------------------------------------------------
// EngineSpec
// ---------------------------------------------------------------------------

impl EngineSpec for BackgammonSpec {
    type State = BackgammonState;
    type Input = BackgammonInput;
    type Diff = BackgammonDiff;
    type Trace = String;
    type NonCommittedInfo = String;
    type OrderKey = u32;
}

// ---------------------------------------------------------------------------
// Behaviors
// ---------------------------------------------------------------------------

struct RollDiceBehavior;

impl Behavior<BackgammonSpec> for RollDiceBehavior {
    fn name(&self) -> &'static str {
        "RollDice"
    }
    fn order_key(&self) -> u32 {
        0
    }
    fn evaluate(
        &self,
        input: &BackgammonInput,
        _state: &BackgammonState,
    ) -> BehaviorResult<BackgammonDiff, String> {
        match input {
            BackgammonInput::RollDice { d1, d2 } => {
                BehaviorResult::Continue(vec![BackgammonDiff::SetDice(*d1, *d2)])
            }
            BackgammonInput::MovePiece { .. } => BehaviorResult::Continue(vec![]),
        }
    }
}

struct MovePieceBehavior;

impl Behavior<BackgammonSpec> for MovePieceBehavior {
    fn name(&self) -> &'static str {
        "MovePiece"
    }
    fn order_key(&self) -> u32 {
        1
    }
    fn evaluate(
        &self,
        input: &BackgammonInput,
        _state: &BackgammonState,
    ) -> BehaviorResult<BackgammonDiff, String> {
        match input {
            BackgammonInput::MovePiece { from, to } => {
                BehaviorResult::Continue(vec![BackgammonDiff::MoveWhite {
                    from: *from,
                    to: *to,
                }])
            }
            BackgammonInput::RollDice { .. } => BehaviorResult::Continue(vec![]),
        }
    }
}

// ---------------------------------------------------------------------------
// Output helpers
// ---------------------------------------------------------------------------

fn print_dispatch(
    label: &str,
    reversibility_label: &str,
    result: &Result<Outcome<Frame<BackgammonSpec>, String>, EngineError>,
) {
    match result {
        Ok(Outcome::Committed(_)) => {
            println!(
                "[dispatch] {} => Committed{}",
                label, reversibility_label
            )
        }
        Ok(Outcome::Aborted(reason)) => {
            println!("[dispatch] {} => Aborted({})", label, reason)
        }
        Ok(Outcome::NoChange) => println!("[dispatch] {} => NoChange", label),
        Ok(Outcome::InvalidInput(r)) => {
            println!("[dispatch] {} => InvalidInput({})", label, r)
        }
        Ok(Outcome::Disallowed(r)) => {
            println!("[dispatch] {} => Disallowed({})", label, r)
        }
        Ok(Outcome::Undone(_)) | Ok(Outcome::Redone(_)) => {
            unreachable!("dispatch never returns Undone/Redone")
        }
        Err(e) => println!("[dispatch] {} => EngineError({:?})", label, e),
    }
}

fn print_undo(
    result: &Result<Outcome<Frame<BackgammonSpec>, HistoryDisallowed>, EngineError>,
) {
    match result {
        Ok(Outcome::Undone(frame)) => {
            println!("[undo] => Undone (was: {:?})", frame.input)
        }
        Ok(Outcome::Disallowed(HistoryDisallowed::NothingToUndo)) => println!(
            "[undo] => Disallowed(NothingToUndo)  <-- undo history was cleared by irreversible RollDice"
        ),
        Ok(other) => println!("[undo] => unexpected: {:?}", other as *const _),
        Err(e) => println!("[undo] => EngineError({:?})", e),
    }
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    let initial_state = BackgammonState {
        dice: None,
        white_pos: 1,
        black_pos: 24,
    };

    let mut engine = Engine::<BackgammonSpec>::new(
        initial_state,
        vec![Box::new(RollDiceBehavior), Box::new(MovePieceBehavior)],
    );

    println!("=== HerdingCats: Backgammon Irreversibility Demo ===");
    println!();

    // Step 1: Move white piece (reversible)
    let r = engine.dispatch(
        BackgammonInput::MovePiece { from: 1, to: 5 },
        Reversibility::Reversible,
    );
    print_dispatch("MovePiece(1->5)", "", &r);
    println!(
        "  undo_depth={}, redo_depth={}",
        engine.undo_depth(),
        engine.redo_depth()
    );

    // Step 2: Move white piece again (reversible)
    let r = engine.dispatch(
        BackgammonInput::MovePiece { from: 5, to: 8 },
        Reversibility::Reversible,
    );
    print_dispatch("MovePiece(5->8)", "", &r);
    println!(
        "  undo_depth={}, redo_depth={}",
        engine.undo_depth(),
        engine.redo_depth()
    );

    // Step 3: Undo the second move
    let r = engine.undo();
    print_undo(&r);
    println!(
        "  undo_depth={}, redo_depth={}",
        engine.undo_depth(),
        engine.redo_depth()
    );

    // Step 4: Roll dice — IRREVERSIBLE — clears all history
    println!();
    println!("Rolling dice with Reversibility::Irreversible...");
    let r = engine.dispatch(
        BackgammonInput::RollDice { d1: 3, d2: 5 },
        Reversibility::Irreversible,
    );
    print_dispatch("RollDice(3,5)", "   [IRREVERSIBLE — history cleared]", &r);
    println!(
        "  undo_depth={}, redo_depth={}",
        engine.undo_depth(),
        engine.redo_depth()
    );

    // Step 5: Try to undo — should return Disallowed(NothingToUndo)
    println!();
    println!("Attempting undo after irreversible RollDice...");
    let r = engine.undo();
    print_undo(&r);

    println!();
    println!("Demo complete. Key insight: irreversible transitions permanently erase undo history.");
}
