use herdingcats::*;
use std::fmt;

//
// ------------------------------------------------------------
// Priority
// ------------------------------------------------------------
//

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[allow(dead_code)]
enum BackgammonPriority {
    Default,
}

//
// ------------------------------------------------------------
// BgState
// ------------------------------------------------------------
//

#[derive(Clone, Debug, PartialEq)]
struct BgState {
    /// board[0..=23]: points (positive = White count, negative = Black count)
    /// board[24]: White bar (positive count)
    /// board[25]: Black bar (negative count — Black checkers on bar stored as negative i8)
    board: [i8; 26],
    white_home: u8,
    black_home: u8,
    dice: [u8; 2],
    dice_used: [bool; 2],
}

impl BgState {
    fn new() -> Self {
        let mut board = [0i8; 26];

        // Standard starting position (White moves toward higher indices)
        // White: +2 on point 23, +5 on point 12, +3 on point 7, +5 on point 5
        board[23] = 2;
        board[12] = 5;
        board[7] = 3;
        board[5] = 5;

        // Black (negative values): mirror positions
        // Black: -2 on point 0, +5 on point 11, -3 on point 16, -5 on point 18
        board[0] = -2;
        board[11] = 5;
        board[16] = -3;
        board[18] = -5;

        BgState {
            board,
            white_home: 0,
            black_home: 0,
            dice: [0, 0],
            dice_used: [false, false],
        }
    }
}

//
// ------------------------------------------------------------
// BackgammonOp
// ------------------------------------------------------------
//

#[derive(Clone)]
#[allow(dead_code)]
enum BackgammonOp {
    /// Roll dice: store d1 and d2 into state.dice
    RollDiceOp { d1: u8, d2: u8 },
    /// Move a checker from `from` to `to`.
    /// If captured=true, the opponent's blot on `to` is hit and sent to their bar.
    /// player_sign: +1 for White (moves toward higher indices), -1 for Black
    MoveOp {
        from: usize,
        to: usize,
        captured: bool,
        die_index: usize,
        player_sign: i8,
    },
    /// Re-enter a checker from the bar onto the board.
    /// bar_idx: 24 for White bar, 25 for Black bar
    ReEnterOp {
        bar_idx: usize,
        to: usize,
        die_index: usize,
        player_sign: i8,
    },
    /// Bear off a checker from `from` to home.
    /// player_sign: +1 for White, -1 for Black
    BearOffOp {
        from: usize,
        die_index: usize,
        player_sign: i8,
    },
}

impl Operation<BgState> for BackgammonOp {
    fn apply(&self, state: &mut BgState) {
        match self {
            BackgammonOp::RollDiceOp { d1, d2 } => {
                state.dice[0] = *d1;
                state.dice[1] = *d2;
                // dice_used remains unchanged
            }

            BackgammonOp::MoveOp {
                from,
                to,
                captured,
                die_index,
                player_sign,
            } => {
                // Move checker from source
                state.board[*from] -= player_sign;
                // Place on destination
                if *captured {
                    // Opponent's blot is on `to`; capture it and send to bar
                    // Opponent sign is opposite
                    let opp_sign = -player_sign;
                    // Place player's checker (overwrites the blot)
                    state.board[*to] = *player_sign;
                    // Send opponent to their bar
                    if opp_sign > 0 {
                        // White goes to bar[24]
                        state.board[24] += 1;
                    } else {
                        // Black goes to bar[25] (stored as negative)
                        state.board[25] -= 1;
                    }
                } else {
                    state.board[*to] += player_sign;
                }
                state.dice_used[*die_index] = true;
            }

            BackgammonOp::ReEnterOp {
                bar_idx,
                to,
                die_index,
                player_sign,
            } => {
                // Remove from bar
                state.board[*bar_idx] -= player_sign;
                // Place on board
                state.board[*to] += player_sign;
                state.dice_used[*die_index] = true;
            }

            BackgammonOp::BearOffOp {
                from,
                die_index,
                player_sign,
            } => {
                // Remove checker from board
                state.board[*from] -= player_sign;
                // Increment home counter (never writes board[26])
                if *player_sign > 0 {
                    state.white_home += 1;
                } else {
                    state.black_home += 1;
                }
                state.dice_used[*die_index] = true;
            }
        }
    }

    fn undo(&self, state: &mut BgState) {
        match self {
            BackgammonOp::RollDiceOp { .. } => {
                unreachable!("RollDiceOp cannot be undone")
            }

            BackgammonOp::MoveOp {
                from,
                to,
                captured,
                die_index,
                player_sign,
            } => {
                // Restore die
                state.dice_used[*die_index] = false;

                if *captured {
                    let opp_sign = -player_sign;
                    // Restore opponent from bar back to `to`
                    if opp_sign > 0 {
                        state.board[24] -= 1;
                    } else {
                        state.board[25] += 1;
                    }
                    // Restore destination to opponent's blot
                    state.board[*to] = opp_sign;
                } else {
                    state.board[*to] -= player_sign;
                }
                // Restore checker to source
                state.board[*from] += player_sign;
            }

            BackgammonOp::ReEnterOp {
                bar_idx,
                to,
                die_index,
                player_sign,
            } => {
                state.dice_used[*die_index] = false;
                // Remove from board
                state.board[*to] -= player_sign;
                // Return to bar
                state.board[*bar_idx] += player_sign;
            }

            BackgammonOp::BearOffOp {
                from,
                die_index,
                player_sign,
            } => {
                state.dice_used[*die_index] = false;
                // Return checker to board
                state.board[*from] += player_sign;
                // Decrement home counter
                if *player_sign > 0 {
                    state.white_home -= 1;
                } else {
                    state.black_home -= 1;
                }
            }
        }
    }

    fn hash_bytes(&self) -> Vec<u8> {
        match self {
            BackgammonOp::RollDiceOp { d1, d2 } => {
                vec![0, *d1, *d2]
            }
            BackgammonOp::MoveOp {
                from,
                to,
                captured,
                die_index,
                player_sign,
            } => {
                vec![
                    1,
                    *from as u8,
                    *to as u8,
                    *captured as u8,
                    *die_index as u8,
                    (*player_sign as i8 + 1) as u8,
                ]
            }
            BackgammonOp::ReEnterOp {
                bar_idx,
                to,
                die_index,
                player_sign,
            } => {
                vec![
                    2,
                    *bar_idx as u8,
                    *to as u8,
                    *die_index as u8,
                    (*player_sign as i8 + 1) as u8,
                ]
            }
            BackgammonOp::BearOffOp {
                from,
                die_index,
                player_sign,
            } => {
                vec![
                    3,
                    *from as u8,
                    *die_index as u8,
                    (*player_sign as i8 + 1) as u8,
                ]
            }
        }
    }
}

//
// ------------------------------------------------------------
// Helpers
// ------------------------------------------------------------
//

fn checker_count(state: &BgState) -> u32 {
    let board_sum: u32 = state.board.iter().map(|x| x.unsigned_abs() as u32).sum();
    board_sum + state.white_home as u32 + state.black_home as u32
}

//
// ------------------------------------------------------------
// Display (stub — full display in Plan 02)
// ------------------------------------------------------------
//

impl fmt::Display for BgState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BgState(white_home={}, black_home={})", self.white_home, self.black_home)
    }
}

//
// ------------------------------------------------------------
// Main (stub — full main in Plan 02)
// ------------------------------------------------------------
//

fn main() {
    let state = BgState::new();
    println!("Backgammon example - checker count: {}", checker_count(&state));
}

//
// ------------------------------------------------------------
// Tests
// ------------------------------------------------------------
//

#[cfg(test)]
mod tests {
    use super::*;

    // ---- checker_count ----

    #[test]
    fn checker_count_standard_start() {
        let state = BgState::new();
        assert_eq!(checker_count(&state), 30);
    }

    // ---- RollDiceOp ----

    #[test]
    fn roll_dice_op_apply_sets_dice() {
        let mut state = BgState::new();
        state.dice_used[0] = false;
        state.dice_used[1] = false;
        let op = BackgammonOp::RollDiceOp { d1: 3, d2: 5 };
        op.apply(&mut state);
        assert_eq!(state.dice[0], 3);
        assert_eq!(state.dice[1], 5);
        // dice_used unchanged
        assert_eq!(state.dice_used[0], false);
        assert_eq!(state.dice_used[1], false);
    }

    // ---- MoveOp (place on empty) roundtrip ----

    #[test]
    fn move_op_place_empty_roundtrip() {
        let mut state = BgState::new();
        // White has 2 on point 23, move one to point 22
        let before = state.clone();
        let op = BackgammonOp::MoveOp {
            from: 23,
            to: 22,
            captured: false,
            die_index: 0,
            player_sign: 1,
        };
        op.apply(&mut state);
        assert_eq!(state.board[23], 1); // one fewer at source
        assert_eq!(state.board[22], 1); // one added at destination
        assert_eq!(state.dice_used[0], true);
        assert_eq!(checker_count(&state), 30);

        op.undo(&mut state);
        assert_eq!(state, before);
        assert_eq!(checker_count(&state), 30);
    }

    // ---- MoveOp (hit blot) roundtrip ----

    #[test]
    fn move_op_hit_blot_roundtrip() {
        // Use a minimal state (not the full starting position) to avoid checker count issues
        let mut state = BgState {
            board: [0i8; 26],
            white_home: 0,
            black_home: 0,
            dice: [0, 0],
            dice_used: [false, false],
        };
        // Set up: White on point 20, Black blot on point 19
        state.board[20] = 1;  // White checker
        state.board[19] = -1; // Black blot
        let before = state.clone();

        let op = BackgammonOp::MoveOp {
            from: 20,
            to: 19,
            captured: true,
            die_index: 1,
            player_sign: 1, // White
        };
        let count_before = checker_count(&state);
        op.apply(&mut state);
        // White now on 19, Black blot sent to bar
        assert_eq!(state.board[19], 1);   // White on destination
        assert_eq!(state.board[20], 0);   // Source cleared
        assert_eq!(state.board[25], -1);  // Black's bar got one checker (negative)
        assert_eq!(state.dice_used[1], true);
        assert_eq!(checker_count(&state), count_before);

        op.undo(&mut state);
        assert_eq!(state, before);
        assert_eq!(checker_count(&state), count_before);
    }

    // ---- ReEnterOp roundtrip ----

    #[test]
    fn reenter_op_roundtrip() {
        // Use a minimal state to avoid checker count mismatch
        let mut state = BgState {
            board: [0i8; 26],
            white_home: 0,
            black_home: 0,
            dice: [0, 0],
            dice_used: [false, false],
        };
        // White has a checker on bar (board[24])
        state.board[24] = 1; // One white on bar
        let before = state.clone();

        let count_before = checker_count(&state);
        let op = BackgammonOp::ReEnterOp {
            bar_idx: 24,
            to: 2,
            die_index: 0,
            player_sign: 1, // White
        };
        op.apply(&mut state);
        assert_eq!(state.board[24], 0);  // Removed from bar
        assert_eq!(state.board[2], 1);   // Placed on board
        assert_eq!(state.dice_used[0], true);
        assert_eq!(checker_count(&state), count_before);

        op.undo(&mut state);
        assert_eq!(state, before);
        assert_eq!(checker_count(&state), count_before);
    }

    // ---- BearOffOp roundtrip ----

    #[test]
    fn bear_off_op_roundtrip_white() {
        // Use a minimal state to avoid checker count mismatch
        let mut state = BgState {
            board: [0i8; 26],
            white_home: 0,
            black_home: 0,
            dice: [0, 0],
            dice_used: [false, false],
        };
        // White has a checker on point 1 (near home)
        state.board[1] = 1;
        let before = state.clone();

        let count_before = checker_count(&state);
        let op = BackgammonOp::BearOffOp {
            from: 1,
            die_index: 0,
            player_sign: 1, // White
        };
        op.apply(&mut state);
        assert_eq!(state.board[1], 0);    // Removed from board
        assert_eq!(state.white_home, 1);  // White home incremented
        assert_eq!(state.dice_used[0], true);
        // board[26] does not exist — we only use board[0..=25]
        assert_eq!(checker_count(&state), count_before);

        op.undo(&mut state);
        assert_eq!(state, before);
        assert_eq!(checker_count(&state), count_before);
    }

    #[test]
    fn bear_off_op_roundtrip_black() {
        // Use a minimal state to avoid checker count mismatch
        let mut state = BgState {
            board: [0i8; 26],
            white_home: 0,
            black_home: 0,
            dice: [0, 0],
            dice_used: [false, false],
        };
        // Black has a checker on point 22 (near Black's home)
        state.board[22] = -1;
        let before = state.clone();

        let count_before = checker_count(&state);
        let op = BackgammonOp::BearOffOp {
            from: 22,
            die_index: 1,
            player_sign: -1, // Black
        };
        op.apply(&mut state);
        assert_eq!(state.board[22], 0);   // Removed from board
        assert_eq!(state.black_home, 1);  // Black home incremented
        assert_eq!(state.dice_used[1], true);
        assert_eq!(checker_count(&state), count_before);

        op.undo(&mut state);
        assert_eq!(state, before);
        assert_eq!(checker_count(&state), count_before);
    }

    // ---- die_index restoration (pitfall 3) ----

    #[test]
    fn move_op_undo_restores_die_unconditionally() {
        let mut state = BgState::new();
        state.dice_used[0] = false;
        let op = BackgammonOp::MoveOp {
            from: 23,
            to: 22,
            captured: false,
            die_index: 0,
            player_sign: 1,
        };
        op.apply(&mut state);
        assert_eq!(state.dice_used[0], true);
        op.undo(&mut state);
        // Must be false unconditionally after undo
        assert_eq!(state.dice_used[0], false);
    }

    // ---- checker_count preserved over any apply+undo ----

    #[test]
    fn checker_count_preserved_move_op() {
        let mut state = BgState::new();
        let op = BackgammonOp::MoveOp {
            from: 5,
            to: 4,
            captured: false,
            die_index: 0,
            player_sign: 1,
        };
        op.apply(&mut state);
        assert_eq!(checker_count(&state), 30);
        op.undo(&mut state);
        assert_eq!(checker_count(&state), 30);
    }

    #[test]
    fn checker_count_preserved_bear_off() {
        let mut state = BgState {
            board: [0i8; 26],
            white_home: 0,
            black_home: 0,
            dice: [0, 0],
            dice_used: [false, false],
        };
        state.board[1] = 1;
        let count_before = checker_count(&state);
        let op = BackgammonOp::BearOffOp {
            from: 1,
            die_index: 0,
            player_sign: 1,
        };
        op.apply(&mut state);
        assert_eq!(checker_count(&state), count_before);
        op.undo(&mut state);
        assert_eq!(checker_count(&state), count_before);
    }
}
