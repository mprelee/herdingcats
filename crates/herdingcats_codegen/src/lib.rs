pub mod ast;
pub mod bindings;
pub mod diagnostics;
pub mod ir;
pub mod parser;
pub mod validate;

use std::fs;
use std::path::Path;

use ast::{RuleFileAst, SourceFile};
use bindings::BindingConfig;
use diagnostics::Diagnostic;
use ir::RuleSetIr;
use parser::parse_rule_file;
use validate::lower_to_ir;

pub fn parse_str(contents: &str) -> Result<RuleFileAst, Diagnostic> {
    parse_rule_file(contents)
}

pub fn load_source(path: impl AsRef<Path>) -> Result<SourceFile, Diagnostic> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .map_err(|err| Diagnostic::io(format!("failed reading {}: {err}", path.display())))?;
    Ok(SourceFile {
        path: Some(path.to_path_buf()),
        contents,
    })
}

pub fn load_and_parse(path: impl AsRef<Path>) -> Result<RuleFileAst, Diagnostic> {
    let source = load_source(path)?;
    parse_str(&source.contents)
}

pub fn lower_with_bindings(
    ast: &RuleFileAst,
    bindings: &BindingConfig,
) -> Result<RuleSetIr, Diagnostic> {
    lower_to_ir(ast, bindings)
}
