use herdingcats::*;
use std::fmt;

//
// ------------------------------------------------------------
// Priority
// ------------------------------------------------------------
//

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum GamePriority {
    Default,
}

//
// ------------------------------------------------------------
// Game State
// ------------------------------------------------------------
//

#[derive(Clone, Debug)]
struct Game {
    board: [Cell; 9],
    current: Player,
    winner: Option<Player>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Player {
    X,
    O,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Cell {
    Empty,
    X,
    O,
}

impl Game {
    fn new() -> Self {
        Self {
            board: [Cell::Empty; 9],
            current: Player::X,
            winner: None,
        }
    }
}

//
// ------------------------------------------------------------
// Operation
// ------------------------------------------------------------
//

#[derive(Clone)]
enum Op {
    Place { idx: usize, player: Player },
    SetWinner { player: Player },
    SwitchTurn,
}

impl Operation<Game> for Op {
    fn apply(&self, state: &mut Game) {
        match self {
            Op::Place { idx, player } => {
                state.board[*idx] = match player {
                    Player::X => Cell::X,
                    Player::O => Cell::O,
                };
            }
            Op::SetWinner { player } => {
                state.winner = Some(*player);
            }
            Op::SwitchTurn => {
                state.current = match state.current {
                    Player::X => Player::O,
                    Player::O => Player::X,
                };
            }
        }
    }

    fn undo(&self, state: &mut Game) {
        match self {
            Op::Place { idx, .. } => {
                state.board[*idx] = Cell::Empty;
            }
            Op::SetWinner { .. } => {
                state.winner = None;
            }
            Op::SwitchTurn => {
                state.current = match state.current {
                    Player::X => Player::O,
                    Player::O => Player::X,
                };
            }
        }
    }

    fn hash_bytes(&self) -> Vec<u8> {
        match self {
            Op::Place { idx, player } => {
                let p = match player {
                    Player::X => 1u8,
                    Player::O => 2u8,
                };
                vec![0, *idx as u8, p]
            }
            Op::SetWinner { player } => {
                let p = match player {
                    Player::X => 1u8,
                    Player::O => 2u8,
                };
                vec![1, p]
            }
            Op::SwitchTurn => vec![2],
        }
    }
}

//
// ------------------------------------------------------------
// Event
// ------------------------------------------------------------
//

#[derive(Clone)]
enum GameEvent {
    Play { idx: usize },
}

//
// ------------------------------------------------------------
// Rules
// ------------------------------------------------------------
//

struct PlayRule;

impl Rule<Game, Op, GameEvent, GamePriority> for PlayRule {
    fn id(&self) -> &'static str {
        "play"
    }

    fn priority(&self) -> GamePriority {
        GamePriority::Default
    }

    fn before(&self, state: &Game, event: &mut GameEvent, tx: &mut Transaction<Op>) {
        if state.winner.is_some() {
            tx.cancelled = true;
            return;
        }

        let GameEvent::Play { idx } = event;

        if state.board[*idx] != Cell::Empty {
            tx.cancelled = true;
            return;
        }

        tx.ops.push(Op::Place {
            idx: *idx,
            player: state.current,
        });

        tx.ops.push(Op::SwitchTurn);
    }
}

struct WinRule;

impl Rule<Game, Op, GameEvent, GamePriority> for WinRule {
    fn id(&self) -> &'static str {
        "win_check"
    }

    fn priority(&self) -> GamePriority {
        GamePriority::Default
    }

    fn after(&self, state: &Game, _event: &GameEvent, tx: &mut Transaction<Op>) {
        let lines = [
            [0, 1, 2],
            [3, 4, 5],
            [6, 7, 8],
            [0, 3, 6],
            [1, 4, 7],
            [2, 5, 8],
            [0, 4, 8],
            [2, 4, 6],
        ];

        for line in lines {
            let [a, b, c] = line;
            let cells = [state.board[a], state.board[b], state.board[c]];

            if cells[0] != Cell::Empty && cells[0] == cells[1] && cells[1] == cells[2] {
                let winner = match cells[0] {
                    Cell::X => Player::X,
                    Cell::O => Player::O,
                    _ => unreachable!(),
                };

                tx.ops.push(Op::SetWinner { player: winner });
            }
        }
    }
}

//
// ------------------------------------------------------------
// Display
// ------------------------------------------------------------
//

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..9 {
            let symbol = match self.board[i] {
                Cell::Empty => ".",
                Cell::X => "X",
                Cell::O => "O",
            };

            write!(f, "{} ", symbol)?;

            if i % 3 == 2 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

//
// ------------------------------------------------------------
// Main
// ------------------------------------------------------------
//

fn main() {
    let mut engine = Engine::<Game, Op, GameEvent, GamePriority>::new(Game::new());

    engine.add_rule(PlayRule, RuleLifetime::Permanent);
    engine.add_rule(WinRule, RuleLifetime::Permanent);

    let moves = [0, 3, 1, 4, 2];

    for m in moves {
        let tx = Transaction::new();

        engine.dispatch(GameEvent::Play { idx: m }, tx);

        println!("{}", engine.state);

        if let Some(winner) = engine.state.winner {
            println!("Winner: {:?}", winner);
            break;
        }
    }
}
