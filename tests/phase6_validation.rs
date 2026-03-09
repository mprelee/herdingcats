use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

fn consumer_manifest() -> &'static str {
    "examples/dsl_consumer/Cargo.toml"
}

fn run_consumer_cargo<I, S>(args: I) -> std::process::Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new("cargo")
        .args(args)
        .arg("--manifest-path")
        .arg(consumer_manifest())
        .output()
        .expect("should run nested consumer crate")
}

fn write_rule_fixture(name: &str, contents: &str) -> PathBuf {
    let unique = format!(
        "herdingcats_phase6_{}_{}_{}",
        name,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock should be monotonic enough")
            .as_nanos()
    );
    let dir = env::temp_dir().join(unique);
    fs::create_dir_all(&dir).expect("should create temp fixture dir");
    let path = dir.join("fixture.cats");
    fs::write(&path, contents).expect("should write temp rules fixture");
    path
}

fn build_consumer_with_rules(rules_path: &Path) -> std::process::Output {
    Command::new("cargo")
        .args(["build", "--quiet", "--manifest-path", consumer_manifest()])
        .env("HERDINGCATS_DSL_RULES_PATH", rules_path)
        .output()
        .expect("should build nested consumer crate")
}

#[test]
fn generated_op_validation_passes_in_real_consumer() {
    let output = run_consumer_cargo(["test", "--quiet", "generated_op"]);
    assert!(
        output.status.success(),
        "nested generated-op tests failed: stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn generated_roundtrip_validation_passes_in_real_consumer() {
    let output = run_consumer_cargo(["test", "--quiet", "generated_roundtrip"]);
    assert!(
        output.status.success(),
        "nested generated-roundtrip tests failed: stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn generated_rule_validation_passes_in_real_consumer() {
    let output = run_consumer_cargo(["test", "--quiet", "generated_rule"]);
    assert!(
        output.status.success(),
        "nested generated-rule tests failed: stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn diagnostics_parse_failure_is_actionable() {
    let rules_path = write_rule_fixture(
        "parse_failure",
        r#"
rule "broken" {
  on TouchdownScored(team)
  before {
    emit AwardPoints(team: team, points: )
  }
}
"#,
    );
    let output = build_consumer_with_rules(&rules_path);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "expected parse fixture to fail: stdout={}\nstderr={stderr}",
        String::from_utf8_lossy(&output.stdout),
    );
    assert!(stderr.contains("parse authored rules"));
    assert!(stderr.contains(&rules_path.display().to_string()));
    assert!(stderr.contains("Parse:"));
    assert!(stderr.contains("help:"));
    assert!(stderr.contains("fix the DSL syntax"));
}

#[test]
fn diagnostics_validation_failure_is_actionable() {
    let rules_path = write_rule_fixture(
        "duplicate_rule_id",
        r#"
rule "dup" {
  on TouchdownScored(team)
  before {
    emit AwardPoints(team: team, points: 1)
  }
}

rule "dup" {
  on TouchdownScored(team)
  before {
    emit AwardPoints(team: team, points: 2)
  }
}
"#,
    );
    let output = build_consumer_with_rules(&rules_path);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "expected validation fixture to fail: stdout={}\nstderr={stderr}",
        String::from_utf8_lossy(&output.stdout),
    );
    assert!(stderr.contains("validate authored rules"));
    assert!(stderr.contains(&rules_path.display().to_string()));
    assert!(stderr.contains("duplicate rule id: dup"));
    assert!(stderr.contains("[rule: dup]"));
    assert!(stderr.contains("help:"));
    assert!(stderr.contains("unique stable id"));
}
