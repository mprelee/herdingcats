/// Describes a transformation that can be applied to a state value.
///
/// # Design philosophy
///
/// An `Effect` is *intent*, not *outcome*. If you apply `Damage(100)` to a
/// character with 10 HP, the effect is still called `Damage(100)` — clamping,
/// saturation, and other boundary conditions are the responsibility of `apply`.
/// This separation keeps the audit trail honest: the log shows what was
/// *intended*, and `apply` encodes what the game rules say *actually happens*.
///
/// Effects are pure: `apply` takes a reference to the current state and returns
/// a new state. Nothing is mutated in place. This makes effects trivially safe
/// to replay, inspect, and test.
///
/// # Type parameter
///
/// `S` is the state type this effect transforms. A single game might have
/// multiple state types (health, position, inventory) and separate effect
/// enums for each.
pub trait Effect<S> {
    /// Produce a new state by applying this effect to `state`.
    ///
    /// Implementations must:
    /// - Never mutate `state` in place.
    /// - Enforce game constraints (clamping, saturation, etc.) here, not at
    ///   the call site.
    /// - Be deterministic: the same `(self, state)` pair must always produce
    ///   the same result.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Damage(100) on 10 HP saturates to 0, not -90.
    /// let result = HealthEffect::Damage(100).apply(&Health::new(10, 100));
    /// assert_eq!(result.current, 0);
    /// ```
    fn apply(&self, state: &S) -> S;

    /// Return a human-readable description of this effect's intent.
    ///
    /// The description should reflect the *intent* of the effect, not the
    /// outcome. `"Damage(100)"` is a better description than
    /// `"reduced HP by 10 (clamped)"`. Descriptions are primarily useful
    /// for logging, debugging, and UI display.
    fn describe(&self) -> String;

    /// Return the logical inverse of this effect, if one exists.
    ///
    /// # Why `Option<Self>` instead of `Box<dyn Effect<S>>`?
    ///
    /// Effects are typically compile-time enums. Returning `Option<Self>`
    /// keeps the inverse as the same concrete type, which means callers get
    /// full pattern-matching capabilities and zero dynamic dispatch overhead.
    ///
    /// If an effect has no meaningful inverse (e.g. clamping means the inverse
    /// would need to know the pre-clamp value, which is lost), return `None`.
    /// The default implementation does exactly this — opt in to invertibility
    /// only where it's well-defined.
    ///
    /// Note that `Self: Sized` is required because we're returning `Self` by
    /// value; this automatically excludes `Effect` from being used as a trait
    /// object when the inverse is needed (which is usually fine, since inverse
    /// is an optional capability).
    fn inverse(&self) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Test domain ────────────────────────────────────────────────────────

    #[derive(Debug, Clone, PartialEq, Hash)]
    struct Health {
        current: u32,
        max: u32,
    }

    impl Health {
        fn new(current: u32, max: u32) -> Self {
            Self { current, max }
        }
        fn full(max: u32) -> Self {
            Self {
                current: max,
                max,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    enum HealthEffect {
        Heal(u32),
        Damage(u32),
        SetMax(u32),
    }

    impl Effect<Health> for HealthEffect {
        fn apply(&self, state: &Health) -> Health {
            match self {
                HealthEffect::Heal(amount) => Health {
                    current: (state.current + amount).min(state.max),
                    max: state.max,
                },
                HealthEffect::Damage(amount) => Health {
                    current: state.current.saturating_sub(*amount),
                    max: state.max,
                },
                HealthEffect::SetMax(new_max) => Health {
                    current: state.current.min(*new_max),
                    max: *new_max,
                },
            }
        }

        fn describe(&self) -> String {
            match self {
                HealthEffect::Heal(n) => format!("Heal({n})"),
                HealthEffect::Damage(n) => format!("Damage({n})"),
                HealthEffect::SetMax(n) => format!("SetMax({n})"),
            }
        }
    }

    // ── Heal tests ─────────────────────────────────────────────────────────

    #[test]
    fn heal_increases_current() {
        let h = Health::new(50, 100);
        let result = HealthEffect::Heal(20).apply(&h);
        assert_eq!(result.current, 70);
        assert_eq!(result.max, 100);
    }

    #[test]
    fn heal_clamps_at_max() {
        let h = Health::new(90, 100);
        let result = HealthEffect::Heal(20).apply(&h);
        // 90 + 20 = 110, but max is 100 — must clamp
        assert_eq!(result.current, 100);
        assert_ne!(result.current, 110);
    }

    #[test]
    fn heal_on_full_health_is_no_op() {
        let h = Health::full(100);
        let result = HealthEffect::Heal(50).apply(&h);
        assert_eq!(result, h);
    }

    // ── Damage tests ───────────────────────────────────────────────────────

    #[test]
    fn damage_decreases_current() {
        let h = Health::new(80, 100);
        let result = HealthEffect::Damage(30).apply(&h);
        assert_eq!(result.current, 50);
    }

    #[test]
    fn damage_saturates_at_zero() {
        let h = Health::new(5, 100);
        let result = HealthEffect::Damage(100).apply(&h);
        // 5 - 100 would underflow u32; must saturate to 0
        assert_eq!(result.current, 0);
        assert_ne!(result.current, u32::MAX); // ensure no wrap-around
    }

    #[test]
    fn damage_does_not_change_max() {
        let h = Health::new(50, 100);
        let result = HealthEffect::Damage(30).apply(&h);
        assert_eq!(result.max, 100);
    }

    // ── SetMax tests ───────────────────────────────────────────────────────

    #[test]
    fn set_max_clamps_current_when_over_new_max() {
        let h = Health::new(80, 100);
        let result = HealthEffect::SetMax(50).apply(&h);
        assert_eq!(result.max, 50);
        assert_eq!(result.current, 50); // clamped from 80
    }

    #[test]
    fn set_max_preserves_current_when_under_new_max() {
        let h = Health::new(30, 100);
        let result = HealthEffect::SetMax(50).apply(&h);
        assert_eq!(result.max, 50);
        assert_eq!(result.current, 30); // unchanged, 30 < 50
    }

    // ── Describe tests ─────────────────────────────────────────────────────

    #[test]
    fn describe_returns_symbolic_string() {
        assert_eq!(HealthEffect::Heal(10).describe(), "Heal(10)");
        assert_eq!(HealthEffect::Damage(25).describe(), "Damage(25)");
        assert_eq!(HealthEffect::SetMax(75).describe(), "SetMax(75)");
    }

    // ── Inverse tests ──────────────────────────────────────────────────────

    #[test]
    fn inverse_returns_none_by_default() {
        // HealthEffect doesn't override inverse(), so all variants return None
        assert_eq!(HealthEffect::Heal(10).inverse(), None);
        assert_eq!(HealthEffect::Damage(5).inverse(), None);
        assert_eq!(HealthEffect::SetMax(100).inverse(), None);
    }
}
