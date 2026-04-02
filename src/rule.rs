use std::hash::Hash;

use crate::delta::{Changeset, Delta, hash_of};
use crate::effect::Effect;

/// A signed integer priority used to order rules within a [`RuleEngine`].
///
/// Higher values run first. Three named constants cover common needs:
/// [`Priority::HIGH`] (100), [`Priority::DEFAULT`] (0), and [`Priority::LOW`]
/// (-100). Arbitrary values are valid — use them when you need finer-grained
/// ordering between the named tiers.
///
/// # Why signed?
///
/// Using `i32` rather than `u32` allows rules to be placed *below* the default
/// baseline — useful for housekeeping or cleanup rules that should always run
/// last, without needing to know the exact priority values of domain rules.
///
/// # Ordering semantics
///
/// `Priority` derives `Ord`, so it can be sorted directly. The engine sorts
/// rules highest-priority-first; `Priority(100)` evaluates before `Priority(0)`,
/// which evaluates before `Priority(-100)`. This matches the intuition that
/// "high priority = runs first."
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority(pub i32);

impl Priority {
    /// Low priority — runs after default and high-priority rules.
    ///
    /// Useful for cleanup rules, housekeeping, and anything that should see the
    /// fully-updated state from all higher-priority rules before firing.
    pub const LOW: Priority = Priority(-100);

    /// The default priority for rules that don't need specific ordering.
    ///
    /// Use this when the rule does not have known interactions with other rules
    /// and ordering does not matter.
    pub const DEFAULT: Priority = Priority(0);

    /// High priority — runs before default and low-priority rules.
    ///
    /// Use this for rules that should see the "raw" state before domain rules
    /// have modified it, or for rules whose output other rules depend on.
    pub const HIGH: Priority = Priority(100);
}

/// A rule that can fire on a state, optionally generating effects.
///
/// # The `matches` / `generate` separation
///
/// Rules are evaluated in two distinct phases:
///
/// 1. **[`matches`](Rule::matches)** — a cheap predicate: is this rule
///    applicable right now? This should be fast: a field comparison, a threshold
///    check, a boolean flag. It allocates nothing.
///
/// 2. **[`generate`](Rule::generate)** — potentially more expensive: given
///    that the rule matches, what effects should it produce? May allocate a
///    `Vec`, perform multi-field lookups, etc.
///
/// The engine always calls `matches` before `generate`. If `matches` returns
/// `false`, `generate` is never called. Structure your rules accordingly: put
/// the cheap guard in `matches` and the heavier computation in `generate`.
///
/// This split also makes rules easy to test in isolation — you can verify
/// matching logic independently of effect generation.
///
/// # Why `generate` returns `Vec<E>`
///
/// A single rule can produce multiple effects in one firing. Returning a `Vec`
/// handles all cases uniformly:
/// - Zero effects: the rule matched (its condition was true) but found nothing
///   to do. This is meaningful — it can be logged, and `matches` returning
///   `true` while `generate` returns `[]` is a valid, observable state.
/// - One effect: the common case.
/// - Multiple effects: a rule that heals HP *and* sets a cooldown produces two
///   deltas in the same tick, both stamped with the same rule ID.
///
/// # Why rules are pure observers
///
/// Rules take `&self` and `&S` — no mutation of rule state or game state.
/// This means rules can be evaluated speculatively, in parallel, or replayed
/// without side effects. Any rule state that must persist across ticks (e.g.
/// cooldowns) should live inside `S` and be modified via effects, so it appears
/// in the Delta audit trail.
///
/// # Type parameters
///
/// - `E` — the effect type this rule produces
/// - `S` — the state type this rule observes
/// - `R` — the rule ID type (typically a user-defined `Copy + Eq + Hash` enum)
pub trait Rule<E, S, R> {
    /// A stable, unique identifier for this rule.
    ///
    /// Used to stamp each [`Delta`] produced by this rule, enabling
    /// [`Changeset::trace`](crate::delta::Changeset::trace) to group all deltas
    /// from a specific rule. Should be unique across the rule set — two rules
    /// with the same ID will produce indistinguishable deltas.
    fn id(&self) -> R;

    /// A human-readable name for this rule.
    ///
    /// Used in logs and debug output. Does not need to be unique or stable.
    fn name(&self) -> &str;

    /// The priority of this rule within the engine's evaluation order.
    ///
    /// Higher values run first. Use [`Priority::HIGH`], [`Priority::DEFAULT`],
    /// or [`Priority::LOW`] for the common cases, or a custom `Priority(n)`
    /// for finer-grained control.
    fn priority(&self) -> Priority;

    /// Return `true` if this rule should fire given `state`.
    ///
    /// This is the cheap gate called before [`generate`](Rule::generate). Keep
    /// it fast — a field comparison, a threshold check. Do not allocate here.
    ///
    /// If `matches` returns `false`, `generate` is never called and this rule
    /// produces no deltas for this evaluation pass.
    fn matches(&self, state: &S) -> bool;

    /// Generate the effects this rule wants to apply to `state`.
    ///
    /// Only called when [`matches`](Rule::matches) returns `true`. The effects
    /// are applied in order, each producing one [`Delta`] stamped with this
    /// rule's [`id`](Rule::id).
    ///
    /// Return an empty `Vec` to indicate the rule matched but has nothing to
    /// do — this is distinct from not matching and can be observed separately.
    fn generate(&self, state: &S) -> Vec<E>;
}

/// Holds a collection of rules and evaluates them against a state in priority
/// order, producing a [`Changeset`] of stamped, ordered [`Delta`]s.
///
/// # Evaluation model
///
/// Rules are evaluated sequentially in descending priority order. Each rule
/// sees the state *after* all higher-priority rules have already applied their
/// effects. This means rule ordering is meaningful: a high-priority heal rule
/// will affect what state a low-priority damage rule observes.
///
/// The produced `Changeset` is always consistent — each delta's `after` hash
/// matches the next delta's `before` hash — because effects are applied
/// incrementally and hashes are recorded at each step.
///
/// # Conflict semantics
///
/// When two rules both want to modify the same field, the highest-priority rule
/// applies its effect first. Lower-priority rules then see the already-modified
/// state. There is no explicit conflict resolution step — priority order is the
/// conflict resolution mechanism.
///
/// # Dynamic dispatch
///
/// `RuleEngine` stores rules as `Box<dyn Rule<E, S, R>>`. This provides
/// flexibility — you can mix rule types, inject rules at runtime, and build
/// engines without knowing all rule types at compile time. The cost is one
/// pointer indirection per rule call.
///
/// If you need zero-cost dispatch, skip `RuleEngine` and write your own
/// evaluation loop against a closed enum of rule types. The [`Effect`],
/// [`Delta`], and [`Changeset`] types are independent of `RuleEngine`.
pub struct RuleEngine<E, S, R> {
    /// Rules stored sorted highest-priority-first.
    /// This invariant is maintained by both `new` and `add_rule`.
    rules: Vec<Box<dyn Rule<E, S, R>>>,
}

impl<E, S, R> RuleEngine<E, S, R> {
    /// Create an engine from an initial set of rules.
    ///
    /// Sorts rules by priority descending (highest priority first) at
    /// construction time so that [`evaluate`](RuleEngine::evaluate) can iterate
    /// in O(n) order without re-sorting.
    pub fn new(mut rules: Vec<Box<dyn Rule<E, S, R>>>) -> Self {
        // Sort descending: highest priority at index 0.
        rules.sort_by_key(|b| std::cmp::Reverse(b.priority()));
        Self { rules }
    }

    /// Add a rule to the engine, maintaining highest-priority-first sort order.
    ///
    /// Uses binary search to find the correct insertion position in O(log n),
    /// then inserts the boxed rule. After this call the internal slice remains
    /// sorted, so the next `evaluate` will see the new rule in its correct
    /// priority position.
    pub fn add_rule(&mut self, rule: Box<dyn Rule<E, S, R>>) {
        // partition_point returns the first index where the predicate is false.
        // We want highest-first, so the predicate is "existing priority ≥ new priority".
        let priority = rule.priority();
        let pos = self.rules.partition_point(|r| r.priority() >= priority);
        self.rules.insert(pos, rule);
    }

    /// Evaluate all rules against `state` and return a [`Changeset`] of every
    /// effect that fired.
    ///
    /// # Algorithm
    ///
    /// 1. Clone `state` into a mutable working copy.
    /// 2. For each rule (highest-priority-first):
    ///    a. Call `rule.matches(&current)`.
    ///    b. If `true`, call `rule.generate(&current)`.
    ///    c. For each effect: record a stamped `Delta`, then advance `current`.
    /// 3. Return the accumulated `Changeset`.
    ///
    /// Each rule sees the state *as modified by all previous rules*, ensuring
    /// deterministic, priority-order interactions.
    ///
    /// The returned `Changeset` satisfies `is_consistent()` by construction.
    pub fn evaluate(&self, state: &S) -> Changeset<E, R>
    where
        E: Effect<S>,
        S: Hash + Clone,
        R: Copy,
    {
        let mut current = state.clone();
        let mut changeset = Changeset::new();

        for rule in &self.rules {
            if rule.matches(&current) {
                let effects = rule.generate(&current);
                let rule_id = rule.id();
                for effect in effects {
                    let before = hash_of(&current);
                    let next = effect.apply(&current);
                    let after = hash_of(&next);
                    changeset.deltas.push(Delta {
                        effect,
                        before,
                        after,
                        source: Some(rule_id),
                    });
                    current = next;
                }
            }
        }

        changeset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delta::hash_of;
    use crate::effect::Effect;

    // ── Test domain ────────────────────────────────────────────────────────

    #[derive(Debug, Clone, PartialEq, Hash)]
    struct HealthState {
        current: u32,
        max: u32,
    }

    impl HealthState {
        fn new(current: u32, max: u32) -> Self {
            Self { current, max }
        }
        fn full(max: u32) -> Self {
            Self { current: max, max }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    enum HealthEffect {
        Heal(u32),
        Damage(u32),
    }

    impl Effect<HealthState> for HealthEffect {
        fn apply(&self, state: &HealthState) -> HealthState {
            match self {
                HealthEffect::Heal(n) => HealthState {
                    current: (state.current + n).min(state.max),
                    max: state.max,
                },
                HealthEffect::Damage(n) => HealthState {
                    current: state.current.saturating_sub(*n),
                    max: state.max,
                },
            }
        }
        fn describe(&self) -> String {
            match self {
                HealthEffect::Heal(n) => format!("Heal({n})"),
                HealthEffect::Damage(n) => format!("Damage({n})"),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum RuleId {
        Regen,
        Poison,
        BossAura,
    }

    // Regen: heals when below threshold and not at max
    struct RegenRule {
        threshold: u32,
        amount: u32,
    }

    impl Rule<HealthEffect, HealthState, RuleId> for RegenRule {
        fn id(&self) -> RuleId { RuleId::Regen }
        fn name(&self) -> &str { "RegenRule" }
        fn priority(&self) -> Priority { Priority::DEFAULT }
        fn matches(&self, state: &HealthState) -> bool {
            state.current < self.threshold && state.current < state.max
        }
        fn generate(&self, _state: &HealthState) -> Vec<HealthEffect> {
            vec![HealthEffect::Heal(self.amount)]
        }
    }

    // Poison: always deals damage
    struct PoisonRule {
        damage: u32,
    }

    impl Rule<HealthEffect, HealthState, RuleId> for PoisonRule {
        fn id(&self) -> RuleId { RuleId::Poison }
        fn name(&self) -> &str { "PoisonRule" }
        fn priority(&self) -> Priority { Priority::LOW }
        fn matches(&self, _state: &HealthState) -> bool { true }
        fn generate(&self, _state: &HealthState) -> Vec<HealthEffect> {
            vec![HealthEffect::Damage(self.damage)]
        }
    }

    // NeverMatchesRule: matches() always false, generate() should never be called
    struct NeverMatchesRule;
    impl Rule<HealthEffect, HealthState, RuleId> for NeverMatchesRule {
        fn id(&self) -> RuleId { RuleId::BossAura }
        fn name(&self) -> &str { "NeverMatchesRule" }
        fn priority(&self) -> Priority { Priority::HIGH }
        fn matches(&self, _: &HealthState) -> bool { false }
        fn generate(&self, _: &HealthState) -> Vec<HealthEffect> {
            // This must never be called
            panic!("generate called on a rule that did not match")
        }
    }

    // NoOpRule: matches but produces no effects
    struct NoOpRule;
    impl Rule<HealthEffect, HealthState, RuleId> for NoOpRule {
        fn id(&self) -> RuleId { RuleId::BossAura }
        fn name(&self) -> &str { "NoOpRule" }
        fn priority(&self) -> Priority { Priority::DEFAULT }
        fn matches(&self, _: &HealthState) -> bool { true }
        fn generate(&self, _: &HealthState) -> Vec<HealthEffect> { vec![] }
    }

    // MultiEffectRule: produces two effects per firing
    struct MultiEffectRule;
    impl Rule<HealthEffect, HealthState, RuleId> for MultiEffectRule {
        fn id(&self) -> RuleId { RuleId::BossAura }
        fn name(&self) -> &str { "MultiEffectRule" }
        fn priority(&self) -> Priority { Priority::HIGH }
        fn matches(&self, _: &HealthState) -> bool { true }
        fn generate(&self, _: &HealthState) -> Vec<HealthEffect> {
            vec![HealthEffect::Damage(10), HealthEffect::Damage(5)]
        }
    }

    // ── Priority ordering tests ────────────────────────────────────────────

    #[test]
    fn priority_high_greater_than_default() {
        assert!(Priority::HIGH > Priority::DEFAULT);
    }

    #[test]
    fn priority_default_greater_than_low() {
        assert!(Priority::DEFAULT > Priority::LOW);
    }

    #[test]
    fn priority_high_greater_than_low() {
        assert!(Priority::HIGH > Priority::LOW);
    }

    #[test]
    fn priority_custom_values_ordered_correctly() {
        assert!(Priority(50) > Priority(25));
        assert!(Priority(-1) < Priority(0));
        assert!(Priority(101) > Priority::HIGH);
    }

    // ── Rule matches gate ──────────────────────────────────────────────────

    #[test]
    fn rule_with_matches_false_never_calls_generate() {
        let engine = RuleEngine::new(vec![
            Box::new(NeverMatchesRule),
        ]);
        let h = HealthState::new(50, 100);
        // If generate were called, it would panic — this test passes if no panic
        let cs = engine.evaluate(&h);
        assert!(cs.deltas.is_empty());
    }

    #[test]
    fn regen_does_not_match_when_at_or_above_threshold() {
        let rule = RegenRule { threshold: 50, amount: 10 };
        assert!(!rule.matches(&HealthState::new(50, 100)));
        assert!(!rule.matches(&HealthState::new(60, 100)));
    }

    #[test]
    fn regen_does_not_match_at_full_health() {
        let rule = RegenRule { threshold: 50, amount: 10 };
        // current == max: current < max fails even if below threshold
        let h = HealthState::new(40, 40);
        assert!(!rule.matches(&h));
    }

    // ── Priority respected in evaluation ──────────────────────────────────

    #[test]
    fn higher_priority_rule_effects_applied_first() {
        // Regen DEFAULT added first, Poison LOW added after — engine should sort
        let engine = RuleEngine::new(vec![
            Box::new(PoisonRule { damage: 5 }),         // LOW
            Box::new(RegenRule { threshold: 100, amount: 10 }), // DEFAULT
        ]);
        let h = HealthState::new(50, 100);
        let cs = engine.evaluate(&h);
        // First delta must come from the higher-priority Regen (DEFAULT > LOW)
        assert_eq!(cs.deltas[0].source, Some(RuleId::Regen));
        assert_eq!(cs.deltas[1].source, Some(RuleId::Poison));
    }

    #[test]
    fn lower_priority_rule_sees_state_updated_by_higher_priority_rule() {
        // Regen (DEFAULT) heals 20: 40 → 60. Poison (LOW) then damages 5.
        // Poison's `before` hash should match state after regen.
        let engine = RuleEngine::new(vec![
            Box::new(RegenRule { threshold: 50, amount: 20 }), // DEFAULT
            Box::new(PoisonRule { damage: 5 }),                // LOW
        ]);
        let h = HealthState::new(40, 100);
        let cs = engine.evaluate(&h);

        let after_regen = HealthState::new(60, 100);
        assert_eq!(cs.deltas[1].before, hash_of(&after_regen));
    }

    // ── RuleEngine::new sorts correctly ───────────────────────────────────

    #[test]
    fn new_sorts_rules_by_priority_descending() {
        let engine: RuleEngine<HealthEffect, HealthState, RuleId> = RuleEngine::new(vec![
            Box::new(PoisonRule { damage: 5 }),                // LOW
            Box::new(RegenRule { threshold: 100, amount: 10 }), // DEFAULT
        ]);
        // Verify order by evaluating — the first delta should be from Regen
        let h = HealthState::new(50, 100);
        let cs = engine.evaluate(&h);
        assert_eq!(cs.deltas[0].source, Some(RuleId::Regen));
    }

    // ── RuleEngine::add_rule maintains sort order ──────────────────────────

    #[test]
    fn add_rule_maintains_sort_order() {
        let mut engine: RuleEngine<HealthEffect, HealthState, RuleId> = RuleEngine::new(vec![
            Box::new(PoisonRule { damage: 5 }), // LOW
        ]);
        // Add a HIGH priority rule after construction
        engine.add_rule(Box::new(MultiEffectRule)); // HIGH

        let h = HealthState::new(50, 100);
        let cs = engine.evaluate(&h);
        // MultiEffectRule (HIGH) should fire first
        assert_eq!(cs.deltas[0].source, Some(RuleId::BossAura));
    }

    #[test]
    fn add_rule_default_priority_inserted_between_high_and_low() {
        let mut engine: RuleEngine<HealthEffect, HealthState, RuleId> = RuleEngine::new(vec![
            Box::new(PoisonRule { damage: 5 }), // LOW
        ]);
        engine.add_rule(Box::new(RegenRule { threshold: 100, amount: 5 })); // DEFAULT

        let h = HealthState::new(50, 100);
        let cs = engine.evaluate(&h);
        // Regen (DEFAULT) should come before Poison (LOW)
        assert_eq!(cs.deltas[0].source, Some(RuleId::Regen));
        assert_eq!(cs.deltas[1].source, Some(RuleId::Poison));
    }

    // ── evaluate: produces consistent Changeset ────────────────────────────

    #[test]
    fn evaluate_produces_consistent_changeset() {
        let engine = RuleEngine::new(vec![
            Box::new(RegenRule { threshold: 80, amount: 20 }),
            Box::new(PoisonRule { damage: 10 }),
        ]);
        let h = HealthState::new(50, 100);
        let cs = engine.evaluate(&h);
        assert!(cs.is_consistent());
    }

    #[test]
    fn evaluate_consistent_with_multiple_rules_and_effects() {
        let engine = RuleEngine::new(vec![
            Box::new(MultiEffectRule),                          // HIGH: 2 effects
            Box::new(RegenRule { threshold: 100, amount: 5 }), // DEFAULT: 1 effect
            Box::new(PoisonRule { damage: 3 }),                // LOW: 1 effect
        ]);
        let h = HealthState::new(50, 100);
        let cs = engine.evaluate(&h);
        assert!(cs.is_consistent());
        assert_eq!(cs.deltas.len(), 4);
    }

    // ── evaluate: stamps correct source RuleId ─────────────────────────────

    #[test]
    fn evaluate_stamps_correct_rule_id_on_each_delta() {
        let engine = RuleEngine::new(vec![
            Box::new(RegenRule { threshold: 100, amount: 10 }),
            Box::new(PoisonRule { damage: 5 }),
        ]);
        let h = HealthState::new(50, 100);
        let cs = engine.evaluate(&h);
        assert_eq!(cs.deltas[0].source, Some(RuleId::Regen));
        assert_eq!(cs.deltas[1].source, Some(RuleId::Poison));
    }

    // ── evaluate: skips non-matching rules ────────────────────────────────

    #[test]
    fn evaluate_skips_non_matching_rules() {
        // Regen won't match full health
        let engine = RuleEngine::new(vec![
            Box::new(RegenRule { threshold: 50, amount: 10 }),
        ]);
        let h = HealthState::full(100);
        let cs = engine.evaluate(&h);
        assert!(cs.deltas.is_empty());
    }

    #[test]
    fn evaluate_no_op_rule_adds_no_deltas() {
        let engine = RuleEngine::new(vec![Box::new(NoOpRule)]);
        let h = HealthState::new(50, 100);
        let cs = engine.evaluate(&h);
        assert!(cs.deltas.is_empty());
    }

    // ── Multiple effects: all deltas present in correct order ─────────────

    #[test]
    fn multiple_effects_from_single_rule_all_present_in_order() {
        let engine = RuleEngine::new(vec![Box::new(MultiEffectRule)]);
        let h = HealthState::new(50, 100);
        let cs = engine.evaluate(&h);
        assert_eq!(cs.deltas.len(), 2);
        // Both stamped with BossAura
        assert_eq!(cs.deltas[0].source, Some(RuleId::BossAura));
        assert_eq!(cs.deltas[1].source, Some(RuleId::BossAura));
        // First effect: Damage(10), second: Damage(5)
        assert_eq!(cs.deltas[0].effect, HealthEffect::Damage(10));
        assert_eq!(cs.deltas[1].effect, HealthEffect::Damage(5));
        assert!(cs.is_consistent());
    }

    // ── Stateful test: simulate two ticks ─────────────────────────────────

    #[test]
    fn two_tick_simulation_second_tick_starts_from_terminus_of_first() {
        let engine = RuleEngine::new(vec![
            Box::new(RegenRule { threshold: 80, amount: 15 }), // DEFAULT
            Box::new(PoisonRule { damage: 5 }),                // LOW
        ]);

        // Tick 1: start at 50 HP
        let initial = HealthState::new(50, 100);
        let cs1 = engine.evaluate(&initial);
        assert!(cs1.is_consistent());

        // Derive the state after tick 1 by replaying effects
        let mut state_after_tick1 = initial.clone();
        for delta in &cs1.deltas {
            state_after_tick1 = delta.effect.apply(&state_after_tick1);
        }

        // Tick 2: starts from terminus of tick 1
        let cs2 = engine.evaluate(&state_after_tick1);
        assert!(cs2.is_consistent());

        // Verify the second changeset's origin matches the hash of the state
        // we derived from the first changeset's terminus
        assert_eq!(
            cs2.origin(),
            Some(&hash_of(&state_after_tick1))
        );

        // Also verify continuity: terminus of tick 1 should equal origin of tick 2
        // (both derived from the same logical state)
        assert_eq!(cs1.terminus(), Some(&hash_of(&state_after_tick1)));
        assert_eq!(cs1.terminus(), cs2.origin());
    }
}
