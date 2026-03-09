// ============================================================
// FNV-1a 64-bit Hash
// ============================================================

// FNV-1a (Fowler–Noll–Vo) is a fast, non-cryptographic hash function chosen
// for its determinism, simplicity, and absence of external dependencies.
// It is used here to build the engine's `replay_hash`: a running fingerprint
// that accumulates the hash of every committed, deterministic operation in
// order, allowing two engine instances to verify they have processed the same
// sequence of moves.
//
// These are the standard FNV-1a 64-bit constants defined in the FNV spec:
// offset basis and prime are fixed values that initialize and mix the hash.
// Changing either would produce a different — incompatible — fingerprint.

// The 64-bit FNV-1a offset basis: the initial hash value before any bytes
// are mixed in. An empty input returns this value unchanged.
pub(crate) const FNV_OFFSET: u64 = 0xcbf29ce484222325;

// The 64-bit FNV-1a prime: each byte is XOR'd into the hash, then the result
// is multiplied by this prime to mix bits across the full 64-bit word.
pub(crate) const FNV_PRIME: u64 = 0x100000001b3;

// Compute the FNV-1a 64-bit hash of `bytes`.
//
// The algorithm iterates each byte, XOR-ing it into the running hash, then
// multiplying by `FNV_PRIME` (wrapping on overflow). This mix ensures that
// every bit of every input byte influences the full 64-bit output.
//
// Edge case: an empty slice returns `FNV_OFFSET` — the unmodified initial
// hash state. The engine uses this as the `replay_hash` value for a fresh
// (no-moves-committed) engine, so an empty game and a game with no
// deterministic ops both share the same fingerprint baseline.
pub(crate) fn fnv1a_hash(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_determinism() {
        assert_eq!(fnv1a_hash(b"hello"), fnv1a_hash(b"hello"));
    }

    #[test]
    fn hash_sensitivity() {
        assert_ne!(fnv1a_hash(b"hello"), fnv1a_hash(b"world"));
    }

    #[test]
    fn hash_empty_input() {
        assert_eq!(fnv1a_hash(&[]), FNV_OFFSET);
    }
}
