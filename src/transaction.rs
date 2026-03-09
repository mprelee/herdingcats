// ============================================================
// Transaction
// ============================================================

#[derive(Clone)]
pub struct Transaction<O> {
    pub ops: Vec<O>,
    pub irreversible: bool,
    pub deterministic: bool,
    pub cancelled: bool,
}

impl<O> Transaction<O> {
    pub fn new() -> Self {
        Self {
            ops: vec![],
            irreversible: true,
            deterministic: true,
            cancelled: false,
        }
    }
}

// ============================================================
// Rule Lifetime
// ============================================================

#[derive(Clone, Copy, Debug)]
pub enum RuleLifetime {
    Permanent,
    Turns(u32),
    Triggers(u32),
}
