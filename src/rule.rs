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
