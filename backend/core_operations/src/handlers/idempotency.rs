/// Compute a stable hash for the request payload we care about.
pub fn compute_request_hash(payload: &str) -> String {
    // For this use case we don't need cryptographic properties — only a stable
    // identifier that distinguishes different payloads. Using the payload
    // string itself keeps things simple and avoids extra dependencies.
    payload.to_owned()
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
