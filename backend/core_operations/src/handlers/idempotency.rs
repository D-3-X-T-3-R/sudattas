use sha2::{Digest, Sha256};

/// Compute a stable hash for the request payload we care about.
pub fn compute_request_hash(payload: &str) -> String {
    // A real hash (not the raw payload) is required here because request_hash is stored in a
    // VARCHAR(255) column: for carts large enough to serialize past 255 chars, storing the raw
    // JSON either fails the insert or silently truncates — and truncated values from two
    // different large payloads sharing the same first-255-char prefix would incorrectly compare
    // equal, defeating the idempotency check. SHA-256 hex (64 chars) always fits and is stable.
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_request_hash_is_stable_and_distinguishes_payloads() {
        let a = compute_request_hash("{\"user_id\":1,\"cart\":[]}");
        let b = compute_request_hash("{\"user_id\":1,\"cart\":[]}");
        assert_eq!(a, b, "same payload must produce same hash");

        let c = compute_request_hash("{\"user_id\":2,\"cart\":[]}");
        assert_ne!(a, c, "different payload must produce different hash");
    }
}
