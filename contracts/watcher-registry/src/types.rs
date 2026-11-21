use soroban_sdk::{contracterror, contracttype};

// ── Errors ────────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    Unauthorized = 2,
    NotInitialized = 3,
    /// Returned when trying to remove the last admin, which would lock the contract.
    LastAdmin = 4,
    /// Returned when the specified watcher is not currently registered.
    WatcherNotFound = 5,
    /// Returned when a mutating entrypoint is called while the contract is paused.
    ContractPaused = 6,
}

// ── Storage keys ─────────────────────────────────────────────────────────────

/// Storage key variants used to address instance entries.
#[contracttype]
pub enum DataKey {
    /// Stores the `Vec<Address>` of current admins.
    Admins,
    /// Stores the `Vec<Address>` of authorized watcher nodes.
    Watchers,
    /// Stores the running count of authorized watchers.
    WatcherCount,
    /// Stores the paused flag for the contract.
    Paused,
}
