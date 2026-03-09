pub mod ast;
pub mod bindings;
pub mod codegen;
pub mod diagnostics;
pub mod ir;
pub mod parser;
pub mod render;
pub mod validate;

use std::fs;
use std::path::Path;

use ast::{RuleFileAst, SourceFile};
use bindings::{BackendConfig, BindingConfig};
use codegen::generate_module;
use diagnostics::Diagnostic;
use ir::RuleSetIr;
use parser::parse_rule_file;
use render::write_generated_module;
use validate::lower_to_ir;

pub fn parse_str(contents: &str) -> Result<RuleFileAst, Diagnostic> {
    parse_rule_file(contents)
}

pub fn load_source(path: impl AsRef<Path>) -> Result<SourceFile, Diagnostic> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path).map_err(|err| {
        Diagnostic::io(format!("failed reading {}: {err}", path.display()))
            .with_source_path(path.display().to_string())
            .with_help("ensure the authored DSL file exists and is readable during build.rs")
    })?;
    Ok(SourceFile {
        path: Some(path.to_path_buf()),
        contents,
    })
}

pub fn load_and_parse(path: impl AsRef<Path>) -> Result<RuleFileAst, Diagnostic> {
    let path = path.as_ref();
    let source = load_source(path)?;
    parse_str(&source.contents).map_err(|diagnostic| {
        diagnostic
            .with_source_path(path.display().to_string())
            .with_help("fix the DSL syntax so the file parses as a v1.1 build-time rule set")
    })
}

pub fn lower_with_bindings(
    ast: &RuleFileAst,
    bindings: &BindingConfig,
) -> Result<RuleSetIr, Diagnostic> {
    lower_to_ir(ast, bindings)
}

pub fn generate_source(ir: &RuleSetIr) -> String {
    generate_module(ir)
}

pub fn generate_runtime_source(ir: &RuleSetIr, backend: &BackendConfig) -> String {
    codegen::generate_runtime_module(ir, backend)
}

pub fn write_source(
    out_dir: impl AsRef<Path>,
    file_name: &str,
    source: &str,
) -> Result<std::path::PathBuf, Diagnostic> {
    write_generated_module(out_dir, file_name, source)
}
