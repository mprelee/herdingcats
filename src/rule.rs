// ============================================================
// Rule Trait
// ============================================================

use crate::operation::Operation;
use crate::transaction::Transaction;

pub trait Rule<S, O, E, P>
where
    S: Clone,
    O: Operation<S>,
    P: Copy + Ord,
{
    fn id(&self) -> &'static str;
    fn priority(&self) -> P;

    fn before(
        &self,
        _state: &S,
        _event: &mut E,
        _tx: &mut Transaction<O>,
    ) {}

    fn after(
        &self,
        _state: &S,
        _event: &E,
        _tx: &mut Transaction<O>,
    ) {}
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::Operation;

    #[derive(Clone)]
    struct NoOp;

    impl Operation<()> for NoOp {
        fn apply(&self, _state: &mut ()) {}
        fn undo(&self, _state: &mut ()) {}
        fn hash_bytes(&self) -> Vec<u8> {
            vec![]
        }
    }

    struct TestRule;

    impl Rule<(), NoOp, (), u8> for TestRule {
        fn id(&self) -> &'static str {
            "test"
        }
        fn priority(&self) -> u8 {
            0
        }
    }

    #[test]
    fn rule_id_and_priority() {
        let rule = TestRule;
        assert_eq!(rule.id(), "test");
        assert_eq!(rule.priority(), 0u8);
    }
}
