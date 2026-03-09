#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleSetIr {
    pub rules: Vec<RuleSpec>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleSpec {
    pub id: String,
    pub priority: u32,
    pub lifetime: LifetimeSpec,
    pub event: EventMatcher,
    pub guards: Vec<GuardPredicate>,
    pub effects: Vec<EffectSpec>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LifetimeSpec {
    Permanent,
    Turns(u32),
    Triggers(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventMatcher {
    pub variant: String,
    pub bindings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardPredicate {
    pub expression: String,
    pub bindings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EffectSpec {
    Emit(EmitSpec),
    Cancel,
    SetDeterministicFalse,
    SetIrreversibleFalse,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmitSpec {
    pub operation: String,
    pub args: Vec<NamedValueSpec>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedValueSpec {
    pub name: String,
    pub value: ValueSpec,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValueSpec {
    Binding(String),
    Integer(i64),
    String(String),
    Boolean(bool),
}
