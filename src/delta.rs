use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// A cheap, non-cryptographic fingerprint of a state value.
///
/// `StateHash` answers one question: "did the state change?" It is computed
/// via [`hash_of`] using Rust's `DefaultHasher`, which is fast but comes with
/// important caveats — see [`hash_of`] for the full story.
///
/// Hashes are compared with `==` to detect changes between state snapshots.
/// They appear in [`Delta`] as `before` and `after` fields, forming a chain
/// that [`Changeset::is_consistent`] can verify.
///
/// # Non-cryptographic nature
///
/// `StateHash` uses `DefaultHasher`, which is **not** cryptographic. It is not
/// suitable for security-sensitive applications, persisting across process
/// boundaries, or network protocols. Its only guarantee is that it is fast and
/// deterministic within a single process execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateHash(u64);

/// Compute a `StateHash` for any [`Hash`]-able state.
///
/// # Implementation
///
/// Uses `std::collections::hash_map::DefaultHasher`. This is intentionally
/// *not* cryptographic: it is fast, available with zero dependencies, and
/// sufficient for change-detection within a single process run.
///
/// # Limitations and when to upgrade
///
/// - **Not stable across Rust versions or compilations.** `DefaultHasher`'s
///   algorithm is unspecified and may change between Rust releases. Do not
///   persist `StateHash` values to disk or send them over the network.
/// - **Not collision-resistant.** Two different states may (rarely) produce
///   the same hash. For game state, false negatives (missing a real change)
///   are worse than false positives, and collisions are rare enough in
///   practice that this is acceptable.
/// - **If you need stability across processes** (e.g. for replays loaded from
///   disk), swap `DefaultHasher` for a stable hasher like `FxHasher` from the
///   `rustc-hash` crate, or a cryptographic hasher if integrity guarantees
///   matter.
pub fn hash_of<S: Hash>(state: &S) -> StateHash {
    let mut hasher = DefaultHasher::new();
    state.hash(&mut hasher);
    StateHash(hasher.finish())
}

/// Records a single state transition caused by applying one effect.
///
/// A `Delta` pairs the symbolic description of a change (the effect) with
/// evidence of what state existed before and after, and optionally the rule
/// that caused it.
///
/// # Fields
///
/// - `effect` — what was applied (the intent, not the outcome)
/// - `before` — the [`StateHash`] of the state *before* the effect
/// - `after` — the [`StateHash`] of the state *after* the effect
/// - `source` — which rule generated this effect, if any
///
/// # Why `source` is `Option<R>`
///
/// Not every state change originates from a rule. External inputs (player
/// actions, network events, scripted cutscenes, test fixtures) produce deltas
/// too. `Option<R>` lets the same `Delta` type represent both rule-driven and
/// externally-driven changes without forcing callers to invent a sentinel rule
/// ID. When `source` is `None`, the change is treated as coming from "outside
/// the rules engine."
///
/// # Hash-chain integrity
///
/// Given a state `s` and a `Delta`, you can verify the delta is consistent:
/// ```text
/// hash(s) == delta.before           → delta was produced from this state
/// hash(effect.apply(s)) == delta.after  → the recorded outcome is correct
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Delta<E, R> {
    /// The effect that caused this transition.
    ///
    /// Records the *intent* — `Damage(100)` remains `Damage(100)` even if the
    /// target only had 10 HP. The before/after hashes capture the actual result.
    pub effect: E,

    /// Hash of the state immediately before `effect` was applied.
    pub before: StateHash,

    /// Hash of the state immediately after `effect` was applied.
    pub after: StateHash,

    /// The rule that generated this effect, if any.
    ///
    /// `None` indicates an external or manually-constructed change — one that
    /// did not come from a rule evaluated by a [`crate::rule::RuleEngine`].
    pub source: Option<R>,
}

/// An ordered sequence of [`Delta`]s representing a series of state changes.
///
/// A `Changeset` is the primary output of [`crate::rule::RuleEngine::evaluate`].
/// It records every effect that fired during a single evaluation pass, in the
/// order they were applied.
///
/// # Hash-chain consistency
///
/// When a changeset is built correctly, the deltas form a *hash chain*:
/// `delta[n].after == delta[n+1].before`. This invariant can be checked with
/// [`Changeset::is_consistent`]. A broken chain indicates a bug in evaluation
/// logic — perhaps effects were recorded out of order, or a delta was
/// constructed against the wrong state snapshot.
///
/// # Conflict detection
///
/// Two deltas with the same `before` hash but different `after` hashes represent
/// a fork — both claim to follow from the same state but diverge. This is useful
/// for detecting concurrent or conflicting rule applications.
#[derive(Debug)]
pub struct Changeset<E, R> {
    /// The deltas in this changeset, ordered by application time.
    pub deltas: Vec<Delta<E, R>>,
}

impl<E, R> Changeset<E, R> {
    /// Create an empty changeset.
    pub fn new() -> Self {
        Self { deltas: Vec::new() }
    }

    /// Check that the deltas form a valid hash chain.
    ///
    /// In a consistent changeset every consecutive pair of deltas satisfies:
    ///
    /// ```text
    /// delta[n].after == delta[n+1].before
    /// ```
    ///
    /// This property guarantees that each effect was applied to the state
    /// produced by the previous effect — there are no gaps or overlaps in the
    /// state history.
    ///
    /// An empty changeset and a single-delta changeset are trivially
    /// consistent (there are no adjacent pairs to check).
    ///
    /// # When to call this
    ///
    /// The rule engine always produces consistent changesets. You might call
    /// this after manually assembling a changeset or when deserializing one
    /// from an untrusted source.
    #[must_use]
    pub fn is_consistent(&self) -> bool {
        self.deltas
            .windows(2)
            .all(|w| w[0].after == w[1].before)
    }

    /// Return a reference to the hash of the state *before* the first delta.
    ///
    /// This is the state the changeset was applied to — the entry point of the
    /// hash chain.
    ///
    /// Returns `None` if the changeset is empty.
    #[must_use]
    pub fn origin(&self) -> Option<&StateHash> {
        self.deltas.first().map(|d| &d.before)
    }

    /// Return a reference to the hash of the state *after* the last delta.
    ///
    /// This is the state the changeset produced — the exit point of the hash
    /// chain.
    ///
    /// Returns `None` if the changeset is empty.
    #[must_use]
    pub fn terminus(&self) -> Option<&StateHash> {
        self.deltas.last().map(|d| &d.after)
    }

    /// Iterate over deltas that were generated by a specific rule.
    ///
    /// Filters by `source == Some(rule)`, returning deltas in their original
    /// order within the changeset.
    ///
    /// # Rule → Delta traceability
    ///
    /// This is where storing `source` on every delta pays off. You can ask
    /// "show me everything the Poison rule did this turn" and get back exactly
    /// those deltas, in order. Combined with `before` and `after` hashes, you
    /// can reconstruct the precise state transitions attributable to one rule.
    ///
    /// Deltas with `source == None` are never returned by `trace`, regardless
    /// of the `rule` argument.
    pub fn trace(&self, rule: R) -> impl Iterator<Item = &Delta<E, R>>
    where
        R: Copy + Eq,
    {
        self.deltas
            .iter()
            .filter(move |d| d.source == Some(rule))
    }
}

impl<E, R> Default for Changeset<E, R> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    }

    // Helper: build a Delta by manually computing hashes
    fn make_delta(
        effect: HealthEffect,
        state: &HealthState,
        source: Option<RuleId>,
    ) -> (Delta<HealthEffect, RuleId>, HealthState) {
        let before = hash_of(state);
        let next = effect.apply(state);
        let after = hash_of(&next);
        let delta = Delta { effect, before, after, source };
        (delta, next)
    }

    // ── hash_of tests ──────────────────────────────────────────────────────

    #[test]
    fn hash_of_same_input_produces_same_hash() {
        let a = HealthState::new(50, 100);
        let b = HealthState::new(50, 100);
        assert_eq!(hash_of(&a), hash_of(&b));
    }

    #[test]
    fn hash_of_different_current_produces_different_hashes() {
        let a = HealthState::new(50, 100);
        let b = HealthState::new(51, 100);
        assert_ne!(hash_of(&a), hash_of(&b));
    }

    #[test]
    fn hash_of_different_max_produces_different_hashes() {
        let a = HealthState::new(50, 100);
        let b = HealthState::new(50, 99);
        assert_ne!(hash_of(&a), hash_of(&b));
    }

    #[test]
    fn hash_of_is_deterministic() {
        let h = HealthState::new(42, 100);
        assert_eq!(hash_of(&h), hash_of(&h));
    }

    // ── Delta field tests ──────────────────────────────────────────────────

    #[test]
    fn delta_fields_store_correctly() {
        let state = HealthState::new(50, 100);
        let (delta, after_state) = make_delta(HealthEffect::Heal(10), &state, Some(RuleId::Regen));

        assert_eq!(delta.effect, HealthEffect::Heal(10));
        assert_eq!(delta.before, hash_of(&state));
        assert_eq!(delta.after, hash_of(&after_state));
        assert_eq!(delta.source, Some(RuleId::Regen));
    }

    #[test]
    fn delta_source_none_for_external_change() {
        let state = HealthState::new(50, 100);
        let (delta, _) = make_delta(HealthEffect::Damage(5), &state, None);
        assert_eq!(delta.source, None);
    }

    #[test]
    fn delta_before_and_after_differ_for_nontrivial_effect() {
        let state = HealthState::new(50, 100);
        let (delta, _) = make_delta(HealthEffect::Heal(10), &state, None);
        assert_ne!(delta.before, delta.after);
    }

    // ── Changeset consistency tests ────────────────────────────────────────

    #[test]
    fn empty_changeset_is_consistent() {
        let cs: Changeset<HealthEffect, RuleId> = Changeset::new();
        assert!(cs.is_consistent());
    }

    #[test]
    fn single_delta_changeset_is_consistent() {
        let state = HealthState::new(50, 100);
        let (delta, _) = make_delta(HealthEffect::Heal(10), &state, Some(RuleId::Regen));
        let cs = Changeset { deltas: vec![delta] };
        assert!(cs.is_consistent());
    }

    #[test]
    fn valid_chain_of_two_is_consistent() {
        let state = HealthState::new(50, 100);
        let (d1, state2) = make_delta(HealthEffect::Heal(10), &state, Some(RuleId::Regen));
        let (d2, _) = make_delta(HealthEffect::Damage(5), &state2, Some(RuleId::Poison));
        let cs = Changeset { deltas: vec![d1, d2] };
        assert!(cs.is_consistent());
    }

    #[test]
    fn broken_chain_is_not_consistent() {
        let state_a = HealthState::new(50, 100);
        let state_b = HealthState::new(99, 100); // different starting state — gap

        let (d1, _) = make_delta(HealthEffect::Heal(10), &state_a, Some(RuleId::Regen));
        // d2.before = hash of state_b, not connected to d1.after
        let (d2, _) = make_delta(HealthEffect::Damage(5), &state_b, Some(RuleId::Poison));

        let cs = Changeset { deltas: vec![d1, d2] };
        assert!(!cs.is_consistent());
    }

    // ── Origin and terminus tests ──────────────────────────────────────────

    #[test]
    fn origin_is_before_of_first_delta() {
        let state = HealthState::new(50, 100);
        let (delta, _) = make_delta(HealthEffect::Heal(10), &state, Some(RuleId::Regen));
        let expected_origin = hash_of(&state);
        let cs = Changeset { deltas: vec![delta] };
        assert_eq!(cs.origin(), Some(&expected_origin));
    }

    #[test]
    fn terminus_is_after_of_last_delta() {
        let state = HealthState::new(50, 100);
        let effect = HealthEffect::Heal(10);
        let after_state = effect.apply(&state);
        let (delta, _) = make_delta(effect, &state, Some(RuleId::Regen));
        let expected_terminus = hash_of(&after_state);
        let cs = Changeset { deltas: vec![delta] };
        assert_eq!(cs.terminus(), Some(&expected_terminus));
    }

    #[test]
    fn origin_is_none_for_empty_changeset() {
        let cs: Changeset<HealthEffect, RuleId> = Changeset::new();
        assert_eq!(cs.origin(), None);
    }

    #[test]
    fn terminus_is_none_for_empty_changeset() {
        let cs: Changeset<HealthEffect, RuleId> = Changeset::new();
        assert_eq!(cs.terminus(), None);
    }

    #[test]
    fn origin_and_terminus_agree_for_single_delta() {
        let state = HealthState::new(0, 100);
        // Damage(0) is a no-op effect (current stays 0, hash won't change)
        // Let's use Heal(0) to see same before/after
        let (delta, _) = make_delta(HealthEffect::Heal(0), &state, None);
        // before and after are same hash (no-op), so origin == terminus
        let cs = Changeset { deltas: vec![delta] };
        assert_eq!(cs.origin(), cs.terminus());
    }

    // ── Trace tests ────────────────────────────────────────────────────────

    #[test]
    fn trace_returns_only_deltas_from_target_rule() {
        let state = HealthState::new(50, 100);
        let (d1, state2) = make_delta(HealthEffect::Heal(10), &state, Some(RuleId::Regen));
        let (d2, _) = make_delta(HealthEffect::Damage(5), &state2, Some(RuleId::Poison));
        let cs = Changeset { deltas: vec![d1, d2] };

        let regen: Vec<_> = cs.trace(RuleId::Regen).collect();
        assert_eq!(regen.len(), 1);
        assert_eq!(regen[0].source, Some(RuleId::Regen));
    }

    #[test]
    fn trace_returns_all_matching_deltas() {
        let state = HealthState::new(50, 100);
        let (d1, state2) = make_delta(HealthEffect::Heal(10), &state, Some(RuleId::Regen));
        let (d2, state3) = make_delta(HealthEffect::Heal(5), &state2, Some(RuleId::Regen));
        let (d3, _) = make_delta(HealthEffect::Damage(3), &state3, Some(RuleId::Poison));
        let cs = Changeset { deltas: vec![d1, d2, d3] };

        let regen: Vec<_> = cs.trace(RuleId::Regen).collect();
        assert_eq!(regen.len(), 2);
        for d in &regen {
            assert_eq!(d.source, Some(RuleId::Regen));
        }
    }

    #[test]
    fn trace_skips_non_matching_rules() {
        let state = HealthState::new(50, 100);
        let (delta, _) = make_delta(HealthEffect::Heal(10), &state, Some(RuleId::Regen));
        let cs = Changeset { deltas: vec![delta] };

        let poison: Vec<_> = cs.trace(RuleId::Poison).collect();
        assert!(poison.is_empty());
    }

    #[test]
    fn trace_skips_source_none_deltas() {
        let state = HealthState::new(50, 100);
        let (delta, _) = make_delta(HealthEffect::Heal(10), &state, None);
        let cs = Changeset { deltas: vec![delta] };

        // Neither rule ID should match a None source
        let regen: Vec<_> = cs.trace(RuleId::Regen).collect();
        let poison: Vec<_> = cs.trace(RuleId::Poison).collect();
        assert!(regen.is_empty());
        assert!(poison.is_empty());
    }
}
