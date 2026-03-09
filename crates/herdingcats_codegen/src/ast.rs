use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFile {
    pub path: Option<PathBuf>,
    pub contents: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleFileAst {
    pub rules: Vec<RuleAst>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleAst {
    pub id: String,
    pub priority: Option<u32>,
    pub lifetime: Option<LifetimeAst>,
    pub event: EventPatternAst,
    pub guards: Vec<GuardAst>,
    pub effects: Vec<EffectAst>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LifetimeAst {
    Permanent,
    Turns(u32),
    Triggers(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventPatternAst {
    pub variant: String,
    pub bindings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardAst {
    pub expression: String,
    pub referenced_bindings: Vec<String>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EffectAst {
    Emit(EmitAst),
    Cancel,
    SetTxFlag(TxFlagAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmitAst {
    pub operation: String,
    pub args: Vec<NamedValueAst>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedValueAst {
    pub name: String,
    pub value: ValueAst,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValueAst {
    Binding(String),
    Integer(i64),
    String(String),
    Boolean(bool),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TxFlagAst {
    IrreversibleFalse,
    DeterministicFalse,
}
