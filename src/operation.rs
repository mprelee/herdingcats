// ============================================================
// Operation Trait
// ============================================================

pub trait Operation<S>: Clone {
    fn apply(&self, state: &mut S);
    fn undo(&self, state: &mut S);
    fn hash_bytes(&self) -> Vec<u8>;
}
