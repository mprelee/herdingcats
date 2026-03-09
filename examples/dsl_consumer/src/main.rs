use herdingcats::{Engine, Operation, Rule, RuleLifetime, Transaction};

include!(concat!(env!("OUT_DIR"), "/generated_rules.rs"));

#[derive(Clone, Debug, PartialEq)]
struct DemoState {
    scoring_mode: String,
    home_points: i64,
    log_count: i64,
}

#[derive(Clone)]
enum DemoEvent {
    TouchdownScored { team: String },
}

#[derive(Clone, Debug, PartialEq)]
enum DemoOp {
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
        }
    }
}

fn main() {
    let mut engine: Engine<DemoState, DemoOp, DemoEvent, u8> = Engine::new(DemoState {
        scoring_mode: String::from("touchdown_plus_one"),
        home_points: 6,
        log_count: 0,
    });
    engine.add_rule(HandwrittenAuditRule, RuleLifetime::Permanent);
    generated_rules::register_generated_rules(&mut engine);

    engine.dispatch(
        DemoEvent::TouchdownScored {
            team: String::from("home"),
        },
        Transaction::new(),
    );

    assert_eq!(engine.state.home_points, 7);
    assert_eq!(engine.state.log_count, 1);
    println!("home_points={} log_count={}", engine.state.home_points, engine.state.log_count);
}
