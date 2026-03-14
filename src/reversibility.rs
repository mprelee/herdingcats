/// Declares whether a dispatch transition may be undone.
///
/// Pass as the second argument to `Engine::dispatch`. The compiler enforces
/// that callers cannot omit this declaration — there is no default.
///
/// # Usage
///
/// ```
/// use herdingcats::Reversibility;
///
/// // A normal game move the player can undo:
/// let _ = Reversibility::Reversible;
///
/// // A dice roll or randomised event that permanently advances history:
/// let _ = Reversibility::Irreversible;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reversibility {
    /// This transition can be undone via `Engine::undo`.
    Reversible,
    /// This transition is irreversible; committing it clears all undo/redo history.
    Irreversible,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reversibility_is_copy_and_eq() {
        let r = Reversibility::Reversible;
        let r2 = r; // copy — no move
        assert_eq!(r, r2);
        assert_eq!(Reversibility::Reversible, Reversibility::Reversible);
        assert_ne!(Reversibility::Reversible, Reversibility::Irreversible);
    }

    #[test]
    fn reversibility_is_debug() {
        let s = format!("{:?}", Reversibility::Reversible);
        assert!(!s.is_empty());
        let s2 = format!("{:?}", Reversibility::Irreversible);
        assert!(!s2.is_empty());
    }

    #[test]
    fn reversibility_variants_are_exhaustively_matchable() {
        let r = Reversibility::Reversible;
        let result = match r {
            Reversibility::Reversible => "reversible",
            Reversibility::Irreversible => "irreversible",
        };
        assert_eq!(result, "reversible");

        let ir = Reversibility::Irreversible;
        let result2 = match ir {
            Reversibility::Reversible => "reversible",
            Reversibility::Irreversible => "irreversible",
        };
        assert_eq!(result2, "irreversible");
    }
}
