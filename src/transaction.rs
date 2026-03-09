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

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_new_defaults() {
        let tx: Transaction<()> = Transaction::new();
        assert!(tx.ops.is_empty());
        assert!(tx.irreversible);
        assert!(tx.deterministic);
        assert!(!tx.cancelled);
    }

    #[test]
    fn transaction_cancelled_can_be_set() {
        let mut tx: Transaction<()> = Transaction::new();
        tx.cancelled = true;
        assert!(tx.cancelled);
    }
}
