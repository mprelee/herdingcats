use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};

use herdingcats_codegen::bindings::{BackendConfig, BindingConfig};
use herdingcats_codegen::diagnostics::{Diagnostic, DiagnosticKind};
use herdingcats_codegen::{generate_runtime_source, load_and_parse, lower_with_bindings, write_source};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    println!("cargo:rerun-if-env-changed=HERDINGCATS_DSL_RULES_PATH");

    let rules_path = env::var_os("HERDINGCATS_DSL_RULES_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.join("rules/example.cats"));
    println!("cargo:rerun-if-changed={}", rules_path.display());

    let ast = unwrap_codegen_result("parse authored rules", &rules_path, load_and_parse(&rules_path));
    let bindings = BindingConfig {
        allowed_event_variants: HashSet::from([String::from("TouchdownScored")]),
        allowed_event_fields: HashSet::from([String::from("team")]),
        allowed_state_paths: HashSet::from([String::from("state.scoring_mode")]),
        allowed_helper_bindings: HashSet::new(),
        allowed_operations: HashSet::from([String::from("AwardPoints")]),
    };
    let ir = unwrap_codegen_result(
        "validate authored rules",
        &rules_path,
        lower_with_bindings(&ast, &bindings),
    );
    let backend = BackendConfig::new("DemoState", "DemoEvent", "DemoOp", "u8");
    let source = generate_runtime_source(&ir, &backend);

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    unwrap_codegen_result(
        "write generated source",
        &rules_path,
        write_source(&out_dir, "generated_rules.rs", &source),
    );
}

fn unwrap_codegen_result<T>(stage: &str, rules_path: &Path, result: Result<T, Diagnostic>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!(
            "{stage} for {} failed: {err}\n{}",
            rules_path.display(),
            help_for_diagnostic(&err)
        ),
    }
}

#[allow(dead_code)]
fn help_for_diagnostic(diagnostic: &Diagnostic) -> &'static str {
    match diagnostic.kind {
        DiagnosticKind::Parse => {
            "help: fix the DSL syntax so the file parses as a v1.1 build-time rule set"
        }
        DiagnosticKind::Validation => {
            "help: stay within the v1.1 contract: approved bindings/operations, before()-only effects, and unique rule ids"
        }
        DiagnosticKind::Io => "help: ensure the authored DSL file exists and is readable during build.rs",
    }
}
