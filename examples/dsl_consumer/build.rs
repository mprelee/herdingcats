use std::collections::HashSet;
use std::env;
use std::path::PathBuf;

use herdingcats_codegen::bindings::{BackendConfig, BindingConfig};
use herdingcats_codegen::{generate_runtime_source, load_and_parse, lower_with_bindings, write_source};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let rules_path = manifest_dir.join("rules/example.cats");
    println!("cargo:rerun-if-changed={}", rules_path.display());

    let ast = load_and_parse(&rules_path).expect("parse rules");
    let bindings = BindingConfig {
        allowed_event_variants: HashSet::from([String::from("TouchdownScored")]),
        allowed_event_fields: HashSet::from([String::from("team")]),
        allowed_state_paths: HashSet::from([String::from("state.scoring_mode")]),
        allowed_helper_bindings: HashSet::new(),
        allowed_operations: HashSet::from([String::from("AwardPoints")]),
    };
    let ir = lower_with_bindings(&ast, &bindings).expect("lower rules");
    let backend = BackendConfig::new("DemoState", "DemoEvent", "DemoOp", "u8");
    let source = generate_runtime_source(&ir, &backend);

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    write_source(&out_dir, "generated_rules.rs", &source).expect("write generated source");
}
