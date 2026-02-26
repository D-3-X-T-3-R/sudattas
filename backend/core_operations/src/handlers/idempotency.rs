/// Compute a stable hash for the request payload we care about.
pub fn compute_request_hash(payload: &str) -> String {
    // For this use case we don't need cryptographic properties — only a stable
    // identifier that distinguishes different payloads. Using the payload
    // string itself keeps things simple and avoids extra dependencies.
    payload.to_owned()
}
