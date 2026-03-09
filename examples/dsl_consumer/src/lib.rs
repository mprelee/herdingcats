use herdingcats::{Engine, Operation, Rule, RuleLifetime, Transaction};

include!(concat!(env!("OUT_DIR"), "/generated_rules.rs"));

#[derive(Clone, Debug, PartialEq)]
pub struct DemoState {
    pub scoring_mode: String,
    pub home_points: i64,
    pub log_count: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DemoEvent {
    TouchdownScored { team: String },
    AuditTick,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DemoOp {
    Generated(generated_rules::GeneratedOp),
    HandwrittenLog,
}

impl Operation<DemoState> for DemoOp {
    fn apply(&self, state: &mut DemoState) {
        match self {
            DemoOp::Generated(generated_rules::GeneratedOp::AwardPoints { team, points }) => {
                if team == "home" {
                    state.home_points += *points;
                }
            }
            DemoOp::HandwrittenLog => state.log_count += 1,
        }
    }

    fn undo(&self, state: &mut DemoState) {
        match self {
            DemoOp::Generated(generated_rules::GeneratedOp::AwardPoints { team, points }) => {
                if team == "home" {
                    state.home_points -= *points;
                }
            }
            DemoOp::HandwrittenLog => state.log_count -= 1,
        }
    }

    fn hash_bytes(&self) -> Vec<u8> {
        match self {
            DemoOp::Generated(generated_rules::GeneratedOp::AwardPoints { team, points }) => {
                format!("award:{team}:{points}").into_bytes()
            }
            DemoOp::HandwrittenLog => b"handwritten-log".to_vec(),
        }
    }
}

pub type DemoEngine = Engine<DemoState, DemoOp, DemoEvent, u8>;

pub fn generated_award_points(team: &str, points: i64) -> DemoOp {
    DemoOp::Generated(generated_rules::GeneratedOp::AwardPoints {
        team: team.to_string(),
        points,
    })
}

pub fn new_engine(scoring_mode: &str, home_points: i64) -> DemoEngine {
    let mut engine = Engine::new(DemoState {
        scoring_mode: scoring_mode.to_string(),
        home_points,
        log_count: 0,
    });
    engine.add_rule(HandwrittenAuditRule, RuleLifetime::Permanent);
    generated_rules::register_generated_rules(&mut engine);
    engine
}

pub fn dispatch_touchdown(engine: &mut DemoEngine, team: &str) {
    engine.dispatch(
        DemoEvent::TouchdownScored {
            team: team.to_string(),
        },
        Transaction::new(),
    );
}

pub fn run_demo() -> DemoState {
    let mut engine = new_engine("touchdown_plus_one", 6);
    dispatch_touchdown(&mut engine, "home");

    assert_eq!(engine.state.home_points, 7);
    assert_eq!(engine.state.log_count, 1);
    engine.state
}

struct HandwrittenAuditRule;

impl Rule<DemoState, DemoOp, DemoEvent, u8> for HandwrittenAuditRule {
    fn id(&self) -> &'static str {
        "handwritten_audit"
    }

    fn priority(&self) -> u8 {
        1
    }

    fn before(&self, _state: &DemoState, event: &mut DemoEvent, tx: &mut Transaction<DemoOp>) {
        match event {
            DemoEvent::TouchdownScored { .. } => tx.ops.push(DemoOp::HandwrittenLog),
            DemoEvent::AuditTick => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn generated_op_hash_bytes_are_stable() {
        let first = generated_award_points("home", 1);
        let second = generated_award_points("home", 1);
        let different = generated_award_points("away", 1);

        assert_eq!(first.hash_bytes(), second.hash_bytes());
        assert_ne!(first.hash_bytes(), different.hash_bytes());
    }

    #[test]
    fn generated_roundtrip_restores_state_and_replay_hash() {
        let mut engine: DemoEngine = Engine::new(DemoState {
            scoring_mode: "manual".to_string(),
            home_points: 6,
            log_count: 0,
        });
        let state_before = engine.read();
        let hash_before = engine.replay_hash();

        let mut tx = Transaction::new();
        tx.ops.push(generated_award_points("home", 1));
        engine.dispatch(
            DemoEvent::TouchdownScored {
                team: "home".to_string(),
            },
            tx,
        );

        assert_eq!(engine.state.home_points, 7);
        engine.undo();
        assert_eq!(engine.read(), state_before);
        assert_eq!(engine.replay_hash(), hash_before);
    }

    #[test]
    fn generated_cancelled_dispatch_keeps_state_and_replay_hash_unchanged() {
        let mut engine = new_engine("cancel_touchdown", 6);
        let state_before = engine.read();
        let hash_before = engine.replay_hash();

        dispatch_touchdown(&mut engine, "home");

        assert_eq!(engine.read(), state_before);
        assert_eq!(engine.replay_hash(), hash_before);
    }

    proptest! {
        #[test]
        fn generated_rule_roundtrip_restores_state_and_replay_hash(start in 0i64..100) {
            let mut engine = new_engine("touchdown_plus_one", start);
            let state_before = engine.read();
            let hash_before = engine.replay_hash();

            dispatch_touchdown(&mut engine, "home");

            prop_assert_eq!(engine.state.home_points, start + 1);
            prop_assert_eq!(engine.state.log_count, 1);

            engine.undo();

            prop_assert_eq!(engine.read(), state_before);
            prop_assert_eq!(engine.replay_hash(), hash_before);
        }
    }
}
