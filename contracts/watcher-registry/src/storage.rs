//! Storage helpers for the watcher registry contract.

use soroban_sdk::{Address, Env};

use crate::types::{DataKey, WatcherRecord};

/// Persist a watcher record keyed by its address.
pub fn set_watcher(env: &Env, watcher: &Address, record: &WatcherRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::Watcher(watcher.clone()), record);
}

/// Load a watcher record if present.
pub fn get_watcher(env: &Env, watcher: &Address) -> Option<WatcherRecord> {
    env.storage()
        .persistent()
        .get(&DataKey::Watcher(watcher.clone()))
}

/// Returns true when the watcher exists and has not expired at the current
/// ledger timestamp.
pub fn is_watcher_authorized(env: &Env, watcher: &Address) -> bool {
    match get_watcher(env, watcher) {
        Some(record) => match record.expires_at {
            Some(exp) => env.ledger().timestamp() <= exp,
            None => true,
        },
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};

    fn sample_record(env: &Env, watcher: &Address, expires_at: Option<u64>) -> WatcherRecord {
        WatcherRecord {
            watcher: watcher.clone(),
            registered_at: env.ledger().timestamp(),
            expires_at,
        }
    }

    #[test]
    fn watcher_authorized_at_expiry_boundary() {
        let env = Env::default();
        let watcher = Address::generate(&env);

        env.ledger().set_timestamp(0);
        set_watcher(&env, &watcher, &sample_record(&env, &watcher, Some(100)));

        // At exactly the expiry timestamp the watcher is still authorized.
        env.ledger().set_timestamp(100);
        assert!(is_watcher_authorized(&env, &watcher));

        // One second past expiry the watcher is no longer authorized.
        env.ledger().set_timestamp(101);
        assert!(!is_watcher_authorized(&env, &watcher));
    }
}
