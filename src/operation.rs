// ============================================================
// Operation Trait
// ============================================================

pub trait Operation<S>: Clone {
    fn apply(&self, state: &mut S);
    fn undo(&self, state: &mut S);
    fn hash_bytes(&self) -> Vec<u8>;
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct Inc;

    impl Operation<i32> for Inc {
        fn apply(&self, state: &mut i32) {
            *state += 1;
        }
        fn undo(&self, state: &mut i32) {
            *state -= 1;
        }
        fn hash_bytes(&self) -> Vec<u8> {
            vec![0]
        }
    }

    #[test]
    fn operation_apply_undo_invert() {
        let op = Inc;
        let mut state = 0i32;
        op.apply(&mut state);
        assert_eq!(state, 1);
        op.undo(&mut state);
        assert_eq!(state, 0);
    }
}
