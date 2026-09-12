//! One-click unsubscribe links embedded in campaign emails need to work for a fully anonymous
//! click — no login, no admin session — while still not letting anyone unsubscribe an arbitrary
//! other subscriber by guessing their numeric id. Signs the subscriber id with the same shared
//! `INTERNAL_API_SECRET` other internal/service-boundary links in this codebase already use
//! (see `graphql/src/main.rs`), so no new secret or schema column is needed.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

fn secret() -> Option<String> {
    std::env::var("INTERNAL_API_SECRET")
        .ok()
        .filter(|s| !s.trim().is_empty())
}

/// Compute the unsubscribe token for a subscriber id. Returns `None` if `INTERNAL_API_SECRET`
/// isn't configured (misconfiguration — callers should treat this as "can't build the link").
pub fn generate_unsubscribe_token(subscriber_id: i64) -> Option<String> {
    let secret = secret()?;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).ok()?;
    mac.update(subscriber_id.to_string().as_bytes());
    Some(hex::encode(mac.finalize().into_bytes()))
}

/// Constant-time verification of a token from an unsubscribe link.
pub fn verify_unsubscribe_token(subscriber_id: i64, token: &str) -> bool {
    let Some(expected) = generate_unsubscribe_token(subscriber_id) else {
        return false;
    };
    let expected_bytes = expected.as_bytes();
    let provided_bytes = token.trim().as_bytes();
    expected_bytes.len() == provided_bytes.len() && expected_bytes.ct_eq(provided_bytes).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    // `#[test]` functions in this module run in parallel by default and all touch the same
    // process-wide `INTERNAL_API_SECRET` env var; serialize them, and restore whatever value
    // (if any) was already there rather than unconditionally removing it — matching the
    // `StartupEnvGuard` pattern in `startup_config.rs`, for the same reason: a real value may
    // already be loaded into the process env from `.env` outside of this test.
    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvGuard {
        _lock: MutexGuard<'static, ()>,
        original: Option<String>,
    }
    impl EnvGuard {
        fn set(val: &str) -> Self {
            let lock = env_lock().lock().unwrap_or_else(|p| p.into_inner());
            let original = std::env::var("INTERNAL_API_SECRET").ok();
            std::env::set_var("INTERNAL_API_SECRET", val);
            EnvGuard {
                _lock: lock,
                original,
            }
        }
        fn unset() -> Self {
            let lock = env_lock().lock().unwrap_or_else(|p| p.into_inner());
            let original = std::env::var("INTERNAL_API_SECRET").ok();
            std::env::remove_var("INTERNAL_API_SECRET");
            EnvGuard {
                _lock: lock,
                original,
            }
        }
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(v) => std::env::set_var("INTERNAL_API_SECRET", v),
                None => std::env::remove_var("INTERNAL_API_SECRET"),
            }
        }
    }

    #[test]
    fn token_round_trips_for_correct_subscriber() {
        let _g = EnvGuard::set("test-secret-value");
        let token = generate_unsubscribe_token(42).expect("secret is set");
        assert!(verify_unsubscribe_token(42, &token));
    }

    #[test]
    fn token_rejected_for_wrong_subscriber_id() {
        let _g = EnvGuard::set("test-secret-value");
        let token = generate_unsubscribe_token(42).expect("secret is set");
        assert!(!verify_unsubscribe_token(43, &token));
    }

    #[test]
    fn token_rejected_when_tampered() {
        let _g = EnvGuard::set("test-secret-value");
        let mut token = generate_unsubscribe_token(42).expect("secret is set");
        token.push('0');
        assert!(!verify_unsubscribe_token(42, &token));
    }

    #[test]
    fn generate_returns_none_without_secret() {
        let _g = EnvGuard::unset();
        assert!(generate_unsubscribe_token(1).is_none());
    }
}
