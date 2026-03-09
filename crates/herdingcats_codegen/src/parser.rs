use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

use crate::ast::{
    EffectAst, EmitAst, EventPatternAst, GuardAst, LifetimeAst, NamedValueAst, RuleAst, RuleFileAst,
    SourceSpan, TxFlagAst, ValueAst,
};
use crate::diagnostics::Diagnostic;

#[derive(Parser)]
#[grammar = "../../../docs/dsl/grammar-scope.pest"]
struct DslParser;

pub fn parse_rule_file(contents: &str) -> Result<RuleFileAst, Diagnostic> {
    let mut pairs = DslParser::parse(Rule::file, contents)
        .map_err(|err| Diagnostic::parse(err.to_string()))?;
    let file = pairs.next().ok_or_else(|| Diagnostic::parse("missing file root"))?;

    let mut rules = Vec::new();
    for pair in file.into_inner() {
        if pair.as_rule() == Rule::rule_block {
            rules.push(parse_rule_block(pair)?);
        }
    }

    Ok(RuleFileAst { rules })
}

fn parse_rule_block(pair: Pair<'_, Rule>) -> Result<RuleAst, Diagnostic> {
    let span = span_from_pair(&pair);
    let mut inner = pair.into_inner();

    let id = inner
        .next()
        .ok_or_else(|| Diagnostic::parse("rule is missing id"))?
        .as_str()
        .trim_matches('"')
        .to_string();

    let mut priority = None;
    let mut lifetime = None;
    let mut event = None;
    let mut guards = Vec::new();
    let mut effects = Vec::new();

    for item in inner {
        match item.as_rule() {
            Rule::priority_decl => {
                let value = item
                    .into_inner()
                    .next()
                    .ok_or_else(|| Diagnostic::parse("priority missing integer"))?
                    .as_str()
                    .parse::<u32>()
                    .map_err(|_| Diagnostic::parse("invalid priority integer"))?;
                priority = Some(value);
            }
            Rule::lifetime_decl => {
                lifetime = Some(parse_lifetime(item)?);
            }
            Rule::event_decl => {
                event = Some(parse_event(item)?);
            }
            Rule::guard_decl => guards.push(parse_guard(item)?),
            Rule::before_block => {
                for effect in item.into_inner() {
                    effects.push(parse_effect(effect)?);
                }
            }
            _ => {}
        }
    }

    let event = event.ok_or_else(|| Diagnostic::parse(format!("rule \"{id}\" missing event")))?;

    Ok(RuleAst {
        id,
        priority,
        lifetime,
        event,
        guards,
        effects,
        span,
    })
}

fn parse_lifetime(pair: Pair<'_, Rule>) -> Result<LifetimeAst, Diagnostic> {
    let text = pair.as_str().trim();
    if text == "lifetime permanent" {
        return Ok(LifetimeAst::Permanent);
    }

    if let Some(rest) = text.strip_prefix("lifetime turns ") {
        let count = rest
            .trim()
            .parse::<u32>()
            .map_err(|_| Diagnostic::parse("invalid turns integer"))?;
        return Ok(LifetimeAst::Turns(count));
    }

    if let Some(rest) = text.strip_prefix("lifetime triggers ") {
        let count = rest
            .trim()
            .parse::<u32>()
            .map_err(|_| Diagnostic::parse("invalid triggers integer"))?;
        return Ok(LifetimeAst::Triggers(count));
    }

    Err(Diagnostic::parse("unsupported lifetime form"))
}

fn parse_event(pair: Pair<'_, Rule>) -> Result<EventPatternAst, Diagnostic> {
    let pattern = pair
        .into_inner()
        .next()
        .ok_or_else(|| Diagnostic::parse("missing event pattern"))?;
    let mut inner = pattern.into_inner();

    let variant = inner
        .next()
        .ok_or_else(|| Diagnostic::parse("missing event variant"))?
        .as_str()
        .to_string();
    let bindings = inner
        .next()
        .map(parse_ident_list)
        .transpose()?
        .unwrap_or_default();

    Ok(EventPatternAst { variant, bindings })
}

fn parse_guard(pair: Pair<'_, Rule>) -> Result<GuardAst, Diagnostic> {
    let span = span_from_pair(&pair);
    let expression_pair = pair
        .into_inner()
        .next()
        .ok_or_else(|| Diagnostic::parse("missing guard expression"))?;
    let expression = expression_pair.as_str().trim().to_string();
    let referenced_bindings = collect_bindings(expression_pair);

    Ok(GuardAst {
        expression,
        referenced_bindings,
        span,
    })
}

fn parse_effect(pair: Pair<'_, Rule>) -> Result<EffectAst, Diagnostic> {
    match pair.as_rule() {
        Rule::emit_stmt => Ok(EffectAst::Emit(parse_emit(pair)?)),
        Rule::cancel_stmt => Ok(EffectAst::Cancel),
        Rule::tx_flag_stmt => Ok(EffectAst::SetTxFlag(parse_tx_flag(pair)?)),
        _ => Err(Diagnostic::parse("unsupported effect statement")),
    }
}

fn parse_emit(pair: Pair<'_, Rule>) -> Result<EmitAst, Diagnostic> {
    let span = span_from_pair(&pair);
    let mut inner = pair.into_inner();
    let operation = inner
        .next()
        .ok_or_else(|| Diagnostic::parse("emit missing operation"))?
        .as_str()
        .to_string();
    let args = inner
        .next()
        .map(parse_named_args)
        .transpose()?
        .unwrap_or_default();

    Ok(EmitAst {
        operation,
        args,
        span,
    })
}

fn parse_tx_flag(pair: Pair<'_, Rule>) -> Result<TxFlagAst, Diagnostic> {
    let mut seen = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::ident {
            seen.push(inner.as_str().to_string());
        }
    }

    match seen.as_slice() {
        [tx, flag] if tx == "tx" && flag == "irreversible" => Ok(TxFlagAst::IrreversibleFalse),
        [tx, flag] if tx == "tx" && flag == "deterministic" => Ok(TxFlagAst::DeterministicFalse),
        _ => Err(Diagnostic::parse("unsupported tx flag assignment")),
    }
}

fn parse_ident_list(pair: Pair<'_, Rule>) -> Result<Vec<String>, Diagnostic> {
    let mut bindings = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::ident {
            bindings.push(inner.as_str().to_string());
        }
    }
    Ok(bindings)
}

fn parse_named_args(pair: Pair<'_, Rule>) -> Result<Vec<NamedValueAst>, Diagnostic> {
    let mut args = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::named_arg {
            let mut arg_inner = inner.into_inner();
            let name = arg_inner
                .next()
                .ok_or_else(|| Diagnostic::parse("named arg missing name"))?
                .as_str()
                .to_string();
            let value = parse_value(
                arg_inner
                    .next()
                    .ok_or_else(|| Diagnostic::parse("named arg missing value"))?,
            )?;
            args.push(NamedValueAst { name, value });
        }
    }
    Ok(args)
}

fn parse_value(pair: Pair<'_, Rule>) -> Result<ValueAst, Diagnostic> {
    let inner = if pair.as_rule() == Rule::value {
        pair.into_inner()
            .next()
            .ok_or_else(|| Diagnostic::parse("value missing inner node"))?
    } else {
        pair
    };

    match inner.as_rule() {
        Rule::binding => Ok(ValueAst::Binding(inner.as_str().to_string())),
        Rule::integer => Ok(ValueAst::Integer(
            inner
                .as_str()
                .parse::<i64>()
                .map_err(|_| Diagnostic::parse("invalid integer literal"))?,
        )),
        Rule::string => Ok(ValueAst::String(inner.as_str().trim_matches('"').to_string())),
        Rule::boolean => Ok(ValueAst::Boolean(inner.as_str() == "true")),
        Rule::literal => parse_value(
            inner
                .into_inner()
                .next()
                .ok_or_else(|| Diagnostic::parse("literal missing inner node"))?,
        ),
        _ => Err(Diagnostic::parse("unsupported value")),
    }
}

fn collect_bindings(pair: Pair<'_, Rule>) -> Vec<String> {
    let mut bindings = Vec::new();
    collect_bindings_recursive(pair, &mut bindings);
    bindings.sort();
    bindings.dedup();
    bindings
}

fn collect_bindings_recursive(pair: Pair<'_, Rule>, bindings: &mut Vec<String>) {
    if pair.as_rule() == Rule::binding {
        bindings.push(pair.as_str().to_string());
    }
    for inner in pair.into_inner() {
        collect_bindings_recursive(inner, bindings);
    }
}

fn span_from_pair(pair: &Pair<'_, Rule>) -> SourceSpan {
    let span = pair.as_span();
    SourceSpan {
        start: span.start(),
        end: span.end(),
    }
}
