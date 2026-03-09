use std::collections::HashSet;

use crate::ast::{EffectAst, LifetimeAst, RuleFileAst, TxFlagAst, ValueAst};
use crate::bindings::BindingConfig;
use crate::diagnostics::Diagnostic;
use crate::ir::{
    EffectSpec, EmitSpec, EventMatcher, GuardPredicate, LifetimeSpec, NamedValueSpec, RuleSetIr,
    RuleSpec, ValueSpec,
};

pub fn lower_to_ir(ast: &RuleFileAst, bindings: &BindingConfig) -> Result<RuleSetIr, Diagnostic> {
    let mut seen_ids = HashSet::new();
    let mut rules = Vec::new();

    for rule in &ast.rules {
        if !seen_ids.insert(rule.id.clone()) {
            return Err(Diagnostic::validation(format!(
                "duplicate rule id: {}",
                rule.id
            )));
        }

        if !bindings.allowed_event_variants.contains(&rule.event.variant) {
            return Err(Diagnostic::validation(format!(
                "rule {} references unapproved event variant {}",
                rule.id, rule.event.variant
            )));
        }

        let event_bindings: HashSet<String> = rule.event.bindings.iter().cloned().collect();
        for binding in &rule.event.bindings {
            if !bindings.allowed_event_fields.contains(binding) {
                return Err(Diagnostic::validation(format!(
                    "rule {} uses unapproved event binding {}",
                    rule.id, binding
                )));
            }
        }

        for guard in &rule.guards {
            for binding in &guard.referenced_bindings {
                if !bindings.allows_binding(binding, &event_bindings) {
                    return Err(Diagnostic::validation(format!(
                        "rule {} references unapproved binding {}",
                        rule.id, binding
                    )));
                }
            }
        }

        let lifetime = match rule.lifetime.as_ref().unwrap_or(&LifetimeAst::Permanent) {
            LifetimeAst::Permanent => LifetimeSpec::Permanent,
            LifetimeAst::Turns(n) if *n > 0 => LifetimeSpec::Turns(*n),
            LifetimeAst::Triggers(n) if *n > 0 => LifetimeSpec::Triggers(*n),
            LifetimeAst::Turns(_) => {
                return Err(Diagnostic::validation(format!(
                    "rule {} has invalid turns lifetime 0",
                    rule.id
                )));
            }
            LifetimeAst::Triggers(_) => {
                return Err(Diagnostic::validation(format!(
                    "rule {} has invalid triggers lifetime 0",
                    rule.id
                )));
            }
        };

        let mut effects = Vec::new();
        for effect in &rule.effects {
            let effect = match effect {
                EffectAst::Emit(emit) => {
                    if !bindings.allowed_operations.contains(&emit.operation) {
                        return Err(Diagnostic::validation(format!(
                            "rule {} emits unapproved operation {}",
                            rule.id, emit.operation
                        )));
                    }

                    let mut args = Vec::new();
                    for arg in &emit.args {
                        let value = match &arg.value {
                            ValueAst::Binding(binding) => {
                                if !bindings.allows_binding(binding, &event_bindings) {
                                    return Err(Diagnostic::validation(format!(
                                        "rule {} references unapproved binding {} in emitted operation",
                                        rule.id, binding
                                    )));
                                }
                                ValueSpec::Binding(binding.clone())
                            }
                            ValueAst::Integer(value) => ValueSpec::Integer(*value),
                            ValueAst::String(value) => ValueSpec::String(value.clone()),
                            ValueAst::Boolean(value) => ValueSpec::Boolean(*value),
                        };
                        args.push(NamedValueSpec {
                            name: arg.name.clone(),
                            value,
                        });
                    }

                    EffectSpec::Emit(EmitSpec {
                        operation: emit.operation.clone(),
                        args,
                    })
                }
                EffectAst::Cancel => EffectSpec::Cancel,
                EffectAst::SetTxFlag(TxFlagAst::DeterministicFalse) => {
                    EffectSpec::SetDeterministicFalse
                }
                EffectAst::SetTxFlag(TxFlagAst::IrreversibleFalse) => {
                    EffectSpec::SetIrreversibleFalse
                }
            };
            effects.push(effect);
        }

        if effects.is_empty() {
            return Err(Diagnostic::validation(format!(
                "rule {} must contain at least one effect",
                rule.id
            )));
        }

        rules.push(RuleSpec {
            id: rule.id.clone(),
            priority: rule.priority.unwrap_or(0),
            lifetime,
            event: EventMatcher {
                variant: rule.event.variant.clone(),
                bindings: rule.event.bindings.clone(),
            },
            guards: rule
                .guards
                .iter()
                .map(|guard| GuardPredicate {
                    expression: guard.expression.clone(),
                    bindings: guard.referenced_bindings.clone(),
                })
                .collect(),
            effects,
        });
    }

    rules.sort_by(|left, right| left.id.cmp(&right.id));

    Ok(RuleSetIr { rules })
}
