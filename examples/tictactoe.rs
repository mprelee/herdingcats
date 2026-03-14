// HerdingCats: Tic-Tac-Toe Scripted Demo
//
// This file is a tutorial-quality example of the full HerdingCats public API.
// Read it top-to-bottom to understand how to:
//   - Define game types and implement EngineSpec
//   - Define BehaviorDef entries with fn pointer fields for each game rule
//   - Implement Apply for your Diff type
//   - Use Engine::dispatch, Engine::undo, and Engine::redo
//   - Handle all 7 Outcome variants exhaustively
//
// Run with: cargo run --example tictactoe

use herdingcats::{
    Apply, BehaviorDef, BehaviorResult, Engine, EngineError, EngineSpec, Frame, HistoryDisallowed,
    NonCommittedOutcome, Outcome, Reversibility,
};

// ── Game spec ────────────────────────────────────────────────────────────────

/// Unit struct that bundles all associated types for the tic-tac-toe game.
/// This is the single type parameter you thread through Engine, BehaviorDef, Apply, etc.
struct TicTacToeSpec;

// ── Game types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
enum Player {
    #[default]
    X,
    O,
}

impl Player {
    fn other(&self) -> Player {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Player::X => write!(f, "X"),
            Player::O => write!(f, "O"),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct TicTacToeState {
    board: [[Option<Player>; 3]; 3],
    current_player: Player,
    game_over: bool,
}

#[derive(Debug, Clone)]
enum TicTacToeInput {
    Place { row: usize, col: usize },
}

#[derive(Debug, Clone)]
enum TicTacToeDiff {
    PlaceMarker { row: usize, col: usize, player: Player },
    SwitchPlayer,
    SetGameOver,
}

// ── Apply: how each Diff mutates state and what traces it emits ───────────────

impl Apply<TicTacToeSpec> for TicTacToeDiff {
    fn apply(&self, state: &mut TicTacToeState) -> Vec<String> {
        match self {
            TicTacToeDiff::PlaceMarker { row, col, player } => {
                state.board[*row][*col] = Some(player.clone());
                vec![format!("placed {} at ({},{})", player, row, col)]
            }
            TicTacToeDiff::SwitchPlayer => {
                let next = state.current_player.other();
                state.current_player = next.clone();
                vec![format!("switched to {}", next)]
            }
            TicTacToeDiff::SetGameOver => {
                state.game_over = true;
                vec!["game over".to_string()]
            }
        }
    }
}

// ── EngineSpec: wire up all associated types ──────────────────────────────────

impl EngineSpec for TicTacToeSpec {
    type State = TicTacToeState;
    type Input = TicTacToeInput;
    type Diff = TicTacToeDiff;
    type Trace = String;
    type NonCommittedInfo = String;
    type OrderKey = u32;
}

// ── Behavior evaluate functions ───────────────────────────────────────────────

/// Behavior 1 (order 0): Guard against moves after the game ends.
/// Returns Stop(NonCommittedOutcome::Disallowed) if game_over is true.
fn validate_turn(
    _input: &TicTacToeInput,
    state: &TicTacToeState,
) -> BehaviorResult<TicTacToeDiff, String> {
    if state.game_over {
        BehaviorResult::Stop(NonCommittedOutcome::Disallowed("game is over".to_string()))
    } else {
        BehaviorResult::Continue(vec![])
    }
}

/// Behavior 2 (order 1): Validate that the target cell exists and is empty.
/// Out-of-bounds → InvalidInput (structurally malformed input).
/// Cell occupied → Disallowed (valid input, rejected by rule).
fn validate_cell(
    input: &TicTacToeInput,
    state: &TicTacToeState,
) -> BehaviorResult<TicTacToeDiff, String> {
    let TicTacToeInput::Place { row, col } = input;
    if *row > 2 || *col > 2 {
        return BehaviorResult::Stop(NonCommittedOutcome::InvalidInput("out of bounds".to_string()));
    }
    if state.board[*row][*col].is_some() {
        return BehaviorResult::Stop(NonCommittedOutcome::Disallowed("cell already occupied".to_string()));
    }
    BehaviorResult::Continue(vec![])
}

/// Behavior 3 (order 2): Place the current player's marker and switch turns.
/// Emits PlaceMarker + SwitchPlayer diffs.
fn place_marker(
    input: &TicTacToeInput,
    state: &TicTacToeState,
) -> BehaviorResult<TicTacToeDiff, String> {
    let TicTacToeInput::Place { row, col } = input;
    BehaviorResult::Continue(vec![
        TicTacToeDiff::PlaceMarker {
            row: *row,
            col: *col,
            player: state.current_player.clone(),
        },
        TicTacToeDiff::SwitchPlayer,
    ])
}

fn has_winner(board: &[[Option<Player>; 3]; 3], player: &Player) -> bool {
    // Rows
    for row_cells in board.iter() {
        if row_cells.iter().all(|cell| cell.as_ref() == Some(player)) {
            return true;
        }
    }
    // Columns
    for col in 0..3usize {
        if board.iter().all(|row_cells| row_cells[col].as_ref() == Some(player)) {
            return true;
        }
    }
    // Diagonals
    if (0..3).all(|i| board[i][i].as_ref() == Some(player)) {
        return true;
    }
    if (0..3).all(|i| board[i][2 - i].as_ref() == Some(player)) {
        return true;
    }
    false
}

/// Behavior 4 (order 3): Check for a winning position after placement.
/// If 3-in-a-row found, emits SetGameOver diff (committed atomically with the move).
fn check_win(
    input: &TicTacToeInput,
    state: &TicTacToeState,
) -> BehaviorResult<TicTacToeDiff, String> {
    // Check both players — PlaceMarker has already been accumulated but not yet applied.
    // Behaviors evaluate against the PRE-APPLY state. We use the input to find the
    // target cell and simulate the board state after placement.
    let placing_player = &state.current_player;
    let TicTacToeInput::Place { row, col } = input;

    // Build a hypothetical board with the marker placed
    let mut hypothetical = state.board.clone();
    hypothetical[*row][*col] = Some(placing_player.clone());

    if has_winner(&hypothetical, placing_player) {
        BehaviorResult::Continue(vec![TicTacToeDiff::SetGameOver])
    } else {
        BehaviorResult::Continue(vec![])
    }
}

// ── Output formatters ─────────────────────────────────────────────────────────

fn print_board(state: &TicTacToeState) {
    println!("  Board:");
    for row in &state.board {
        let row_str: String = row
            .iter()
            .map(|cell| match cell {
                Some(Player::X) => "X",
                Some(Player::O) => "O",
                None => ".",
            })
            .collect::<Vec<_>>()
            .join("|");
        println!("    {}", row_str);
    }
    println!(
        "  Current player: {}  game_over: {}",
        state.current_player, state.game_over
    );
}

fn print_dispatch(
    label: &str,
    result: &Result<Outcome<Frame<TicTacToeSpec>, String>, EngineError>,
) {
    match result {
        Ok(Outcome::Committed(frame)) => println!(
            "[dispatch] {} => Committed (diffs: {}, traces: {:?})",
            label,
            frame.diffs.len(),
            frame.traces
        ),
        Ok(Outcome::NoChange) => println!("[dispatch] {} => NoChange", label),
        Ok(Outcome::Aborted(reason)) => {
            println!("[dispatch] {} => Aborted({})", label, reason)
        }
        Ok(Outcome::InvalidInput(reason)) => {
            println!("[dispatch] {} => InvalidInput({})", label, reason)
        }
        Ok(Outcome::Disallowed(reason)) => {
            println!("[dispatch] {} => Disallowed({})", label, reason)
        }
        Ok(Outcome::Undone(_)) | Ok(Outcome::Redone(_)) => {
            unreachable!("dispatch never returns Undone or Redone")
        }
        Err(e) => println!("[dispatch] {} => EngineError({:?})", label, e),
    }
}

fn print_undo(
    result: &Result<Outcome<Frame<TicTacToeSpec>, HistoryDisallowed>, EngineError>,
) {
    match result {
        Ok(Outcome::Undone(frame)) => println!(
            "[undo]     => Undone (diffs: {}, traces: {:?})",
            frame.diffs.len(),
            frame.traces
        ),
        Ok(Outcome::Disallowed(reason)) => {
            println!("[undo]     => Disallowed({:?})", reason)
        }
        Ok(Outcome::Committed(_)) | Ok(Outcome::Redone(_)) | Ok(Outcome::NoChange)
        | Ok(Outcome::InvalidInput(_)) | Ok(Outcome::Aborted(_)) => {
            unreachable!("undo only returns Undone or Disallowed")
        }
        Err(e) => println!("[undo]     => EngineError({:?})", e),
    }
}

fn print_redo(
    result: &Result<Outcome<Frame<TicTacToeSpec>, HistoryDisallowed>, EngineError>,
) {
    match result {
        Ok(Outcome::Redone(frame)) => println!(
            "[redo]     => Redone (diffs: {}, traces: {:?})",
            frame.diffs.len(),
            frame.traces
        ),
        Ok(Outcome::Disallowed(reason)) => {
            println!("[redo]     => Disallowed({:?})", reason)
        }
        Ok(Outcome::Committed(_)) | Ok(Outcome::Undone(_)) | Ok(Outcome::NoChange)
        | Ok(Outcome::InvalidInput(_)) | Ok(Outcome::Aborted(_)) => {
            unreachable!("redo only returns Redone or Disallowed")
        }
        Err(e) => println!("[redo]     => EngineError({:?})", e),
    }
}

// ── Main: scripted demo sequence ──────────────────────────────────────────────

fn main() {
    println!("=== HerdingCats: Tic-Tac-Toe Demo ===");
    println!();
    println!("This demo exercises the full public API:");
    println!("  Engine::dispatch, Engine::undo, Engine::redo");
    println!("  All 7 Outcome variants covered in match arms");
    println!();

    // Build the engine with 4 behaviors.
    // Behaviors are sorted by (order_key, name) — evaluation order is deterministic.
    let behaviors: Vec<BehaviorDef<TicTacToeSpec>> = vec![
        BehaviorDef { name: "ValidateTurn",  order_key: 0, evaluate: validate_turn },
        BehaviorDef { name: "ValidateCell",  order_key: 1, evaluate: validate_cell },
        BehaviorDef { name: "PlaceMarker",   order_key: 2, evaluate: place_marker },
        BehaviorDef { name: "CheckWin",      order_key: 3, evaluate: check_win },
    ];
    let mut engine = Engine::<TicTacToeSpec>::new(TicTacToeState::default(), behaviors);

    // ── Step 1: X places at (0,0) — Committed ────────────────────────────────
    println!("Step 1: X places at (0,0)  [demonstrates: Committed]");
    let result = engine.dispatch(TicTacToeInput::Place { row: 0, col: 0 }, Reversibility::Reversible);
    print_dispatch("Place(0,0) by X", &result);
    print_board(engine.state());
    println!("  undo_depth={} redo_depth={}", engine.undo_depth(), engine.redo_depth());
    println!();

    // ── Step 2: O places at (1,1) — Committed ────────────────────────────────
    println!("Step 2: O places at (1,1)  [demonstrates: Committed]");
    let result = engine.dispatch(TicTacToeInput::Place { row: 1, col: 1 }, Reversibility::Reversible);
    print_dispatch("Place(1,1) by O", &result);
    print_board(engine.state());
    println!("  undo_depth={} redo_depth={}", engine.undo_depth(), engine.redo_depth());
    println!();

    // ── Step 3: X tries (0,0) again — Disallowed (cell already occupied) ────
    println!("Step 3: X tries (0,0) again  [demonstrates: Disallowed — ValidateCell fires]");
    let result = engine.dispatch(TicTacToeInput::Place { row: 0, col: 0 }, Reversibility::Reversible);
    print_dispatch("Place(0,0) again", &result);
    println!("  (board unchanged — Disallowed leaves state intact)");
    println!("  undo_depth={} redo_depth={}", engine.undo_depth(), engine.redo_depth());
    println!();

    // ── Step 4: undo O's move — Undone ───────────────────────────────────────
    println!("Step 4: undo  [demonstrates: Undone]");
    println!("  undo_depth={} redo_depth={}", engine.undo_depth(), engine.redo_depth());
    let result = engine.undo();
    print_undo(&result);
    print_board(engine.state());
    println!("  undo_depth={} redo_depth={}", engine.undo_depth(), engine.redo_depth());
    println!();

    // ── Step 5: redo O's move — Redone ───────────────────────────────────────
    println!("Step 5: redo  [demonstrates: Redone]");
    println!("  undo_depth={} redo_depth={}", engine.undo_depth(), engine.redo_depth());
    let result = engine.redo();
    print_redo(&result);
    print_board(engine.state());
    println!("  undo_depth={} redo_depth={}", engine.undo_depth(), engine.redo_depth());
    println!();

    // ── Step 6: X tries (3,3) — InvalidInput (out of bounds) ────────────────
    println!("Step 6: X tries (3,3)  [demonstrates: InvalidInput — ValidateCell out-of-bounds]");
    let result = engine.dispatch(TicTacToeInput::Place { row: 3, col: 3 }, Reversibility::Reversible);
    print_dispatch("Place(3,3) out-of-bounds", &result);
    println!("  undo_depth={} redo_depth={}", engine.undo_depth(), engine.redo_depth());
    println!();

    // ── Step 7: NoChange via a fresh zero-behavior engine ────────────────────
    //
    // NoChange is produced when all behaviors return Continue([]) with no diffs.
    // We demonstrate it with a minimal engine that has no behaviors at all.
    println!("Step 7: NoChange — dispatch into an engine with no behaviors");
    println!("  [demonstrates: NoChange — zero diffs from all behaviors]");
    {
        let mut no_behavior_engine =
            Engine::<TicTacToeSpec>::new(TicTacToeState::default(), vec![]);
        let result = no_behavior_engine.dispatch(
            TicTacToeInput::Place { row: 0, col: 0 },
            Reversibility::Reversible,
        );
        print_dispatch("Place(0,0) with no behaviors", &result);
    }
    println!();

    // ── Step 8: Play to X wins — Committed with SetGameOver ──────────────────
    println!("Step 8: Play to X winning position  [demonstrates: Committed + CheckWin SetGameOver]");
    // Resume from main engine: board has X@(0,0), O@(1,1). It is X's turn.
    // X: (0,1) → Committed
    let result = engine.dispatch(TicTacToeInput::Place { row: 0, col: 1 }, Reversibility::Reversible);
    print_dispatch("Place(0,1) by X", &result);
    // O: (2,0) → Committed
    let result = engine.dispatch(TicTacToeInput::Place { row: 2, col: 0 }, Reversibility::Reversible);
    print_dispatch("Place(2,0) by O", &result);
    // X: (0,2) → Committed + SetGameOver (X wins row 0)
    let result = engine.dispatch(TicTacToeInput::Place { row: 0, col: 2 }, Reversibility::Reversible);
    print_dispatch("Place(0,2) by X — wins row 0!", &result);
    print_board(engine.state());
    println!();

    // ── Step 9: Post-game dispatch — Disallowed (ValidateTurn: game_over) ───
    println!("Step 9: Dispatch after game over  [demonstrates: Disallowed — ValidateTurn fires]");
    let result = engine.dispatch(TicTacToeInput::Place { row: 2, col: 2 }, Reversibility::Reversible);
    print_dispatch("Place(2,2) after game over", &result);
    println!();

    // ── Step 10: undo on empty stack — Disallowed ────────────────────────────
    //
    // We demonstrate Disallowed by undoing past the beginning of history.
    println!("Step 10: Exhaust undo stack  [demonstrates: Disallowed(NothingToUndo)]");
    println!("  (draining undo stack...)");
    while engine.undo_depth() > 0 {
        let _ = engine.undo();
    }
    println!("  undo_depth={} (stack empty)", engine.undo_depth());
    let result = engine.undo();
    print_undo(&result);
    println!();

    // ── Final board state ─────────────────────────────────────────────────────
    println!("Final state (after all undos):");
    print_board(engine.state());
    println!();
    println!("Demo complete.");
    println!();
    println!("Outcome variants demonstrated at runtime:");
    println!("  Committed    — Steps 1, 2, 8");
    println!("  Disallowed   — Steps 3, 9, 10");
    println!("  InvalidInput — Step 6");
    println!("  Undone       — Step 4");
    println!("  Redone       — Step 5");
    println!("  NoChange     — Step 7");
}
