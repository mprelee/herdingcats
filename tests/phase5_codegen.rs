use std::collections::HashSet;

use herdingcats_codegen::bindings::BindingConfig;
use herdingcats_codegen::ir::{EffectSpec, LifetimeSpec, ValueSpec};
use herdingcats_codegen::{generate_source, lower_with_bindings, parse_str, write_source};

fn base_bindings() -> BindingConfig {
    BindingConfig {
        allowed_event_variants: HashSet::from([String::from("TouchdownScored")]),
        allowed_event_fields: HashSet::from([String::from("team")]),
        allowed_state_paths: HashSet::from([String::from("state.scoring_mode")]),
        allowed_helper_bindings: HashSet::new(),
        allowed_operations: HashSet::from([String::from("AwardPoints")]),
    }
}

#[test]
fn parse_and_lower_success() {
    let src = r#"
rule "scoring.touchdown_bonus" {
  priority 10
  lifetime permanent
  on TouchdownScored(team)
  when state.scoring_mode == "touchdown_plus_one"
  before {
    emit AwardPoints(team: team, points: 1)
  }
}
"#;

    let ast = parse_str(src).expect("should parse");
    let ir = lower_with_bindings(&ast, &base_bindings()).expect("should lower");

    assert_eq!(ir.rules.len(), 1);
    let rule = &ir.rules[0];
    assert_eq!(rule.id, "scoring.touchdown_bonus");
    assert_eq!(rule.priority, 10);
    assert_eq!(rule.lifetime, LifetimeSpec::Permanent);
    assert_eq!(rule.event.variant, "TouchdownScored");
    assert_eq!(rule.event.bindings, vec!["team"]);
    match &rule.effects[0] {
        EffectSpec::Emit(emit) => {
            assert_eq!(emit.operation, "AwardPoints");
            assert_eq!(emit.args.len(), 2);
            assert_eq!(emit.args[0].name, "team");
            assert_eq!(emit.args[0].value, ValueSpec::Binding(String::from("team")));
        }
        other => panic!("expected emit effect, found {other:?}"),
    }
}

#[test]
fn duplicate_rule_ids_fail() {
    let src = r#"
rule "dup" {
  on TouchdownScored(team)
  before { emit AwardPoints(team: team, points: 1) }
}

rule "dup" {
  on TouchdownScored(team)
  before { emit AwardPoints(team: team, points: 1) }
}
"#;

    let ast = parse_str(src).expect("should parse");
    let err = lower_with_bindings(&ast, &base_bindings()).expect_err("duplicate id should fail");
    assert!(err.to_string().contains("duplicate rule id"));
}

#[test]
fn unapproved_binding_fails() {
    let src = r#"
rule "bad.binding" {
  on TouchdownScored(team)
  when state.rules.field_goal_bonus_enabled == true
  before { emit AwardPoints(team: team, points: 1) }
}
"#;

    let ast = parse_str(src).expect("should parse");
    let err = lower_with_bindings(&ast, &base_bindings()).expect_err("binding should fail");
    assert!(err.to_string().contains("unapproved binding"));
}

#[test]
fn zero_turns_lifetime_fails() {
    let src = r#"
rule "bad.lifetime" {
  lifetime turns 0
  on TouchdownScored(team)
  before { emit AwardPoints(team: team, points: 1) }
}
"#;

    let ast = parse_str(src).expect("should parse");
    let err = lower_with_bindings(&ast, &base_bindings()).expect_err("zero lifetime should fail");
    assert!(err.to_string().contains("invalid turns lifetime 0"));
}

#[test]
fn generated_source_is_deterministic() {
    let src = r#"
rule "scoring.touchdown_bonus" {
  priority 10
  lifetime permanent
  on TouchdownScored(team)
  when state.scoring_mode == "touchdown_plus_one"
  before {
    emit AwardPoints(team: team, points: 1)
  }
}
"#;

    let ast = parse_str(src).expect("should parse");
    let ir = lower_with_bindings(&ast, &base_bindings()).expect("should lower");
    let generated_once = generate_source(&ir);
    let generated_twice = generate_source(&ir);

    assert_eq!(generated_once, generated_twice);
    assert!(generated_once.contains("register_generated_rules"));
    assert!(generated_once.contains("GeneratedRuleDescriptor"));
    assert!(generated_once.contains("AwardPoints"));
}

#[test]
fn write_source_writes_generated_module() {
    let dir = std::env::temp_dir().join(format!(
        "herdingcats_codegen_test_{}",
        std::process::id()
    ));
    let path = write_source(&dir, "generated_rules.rs", "// generated").expect("should write");
    let contents = std::fs::read_to_string(&path).expect("should read generated file");

    assert_eq!(contents, "// generated");
    std::fs::remove_file(&path).ok();
    std::fs::remove_dir(&dir).ok();
}
