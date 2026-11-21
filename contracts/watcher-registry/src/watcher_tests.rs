//! Unit tests for `WatcherRegistry` admin and watcher operations (#260).

#![cfg(test)]

use soroban_sdk::{testutils::{Address as _, Ledger as _}, Address, Env, Vec};

use crate::{ContractError, WatcherRegistry, WatcherRegistryClient};

fn setup() -> (Env, WatcherRegistryClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(WatcherRegistry, ());
    let client = WatcherRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

#[test]
fn initialize_sets_the_admin() {
    let (_env, client, admin) = setup();
    assert_eq!(client.get_admins().len(), 1);
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn double_initialize_is_rejected() {
    let (env, client, _admin) = setup();
    let other = Address::generate(&env);
    assert_eq!(client.try_initialize(&other), Err(Ok(ContractError::AlreadyInitialized)));
}

#[test]
fn non_admin_cannot_add_an_admin() {
    let (env, client, _admin) = setup();
    let stranger = Address::generate(&env);
    let new_admin = Address::generate(&env);
    assert_eq!(client.try_add_admin(&stranger, &new_admin), Err(Ok(ContractError::Unauthorized)));
}

#[test]
fn duplicate_add_admin_is_idempotent() {
    let (env, client, admin) = setup();
    let second = Address::generate(&env);
    client.add_admin(&admin, &second);
    client.add_admin(&admin, &second);
    assert_eq!(client.get_admins().len(), 2);
}

#[test]
fn removing_the_last_admin_is_refused() {
    let (_env, client, admin) = setup();
    assert_eq!(client.try_remove_admin(&admin, &admin), Err(Ok(ContractError::LastAdmin)));
}

#[test]
fn transfer_admin_replaces_the_admin_set() {
    let (env, client, admin) = setup();
    let new_admin = Address::generate(&env);
    client.transfer_admin(&admin, &new_admin);
    let admins = client.get_admins();
    assert_eq!(admins.len(), 1);
    assert_eq!(admins.get(0).unwrap(), new_admin);
}

#[test]
fn register_watcher_is_idempotent_and_admin_only() {
    let (env, client, admin) = setup();
    let watcher = Address::generate(&env);
    client.register_watcher(&admin, &watcher, &None);
    client.register_watcher(&admin, &watcher, &None);
    assert_eq!(client.get_watcher_count(), 1);

    let stranger = Address::generate(&env);
    assert_eq!(client.try_register_watcher(&stranger, &watcher, &None), Err(Ok(ContractError::Unauthorized)));
}

#[test]
fn remove_unknown_watcher_returns_not_found() {
    let (env, client, admin) = setup();
    let watcher = Address::generate(&env);
    assert_eq!(client.try_remove_watcher(&admin, &watcher), Err(Ok(ContractError::WatcherNotFound)));
}

#[test]
fn clear_all_watchers_empties_the_list() {
    let (env, client, admin) = setup();
    client.register_watcher(&admin, &Address::generate(&env), &None);
    client.register_watcher(&admin, &Address::generate(&env), &None);
    client.clear_all_watchers(&admin);
    assert_eq!(client.get_watcher_count(), 0);
}

#[test]
fn is_authorized_is_true_only_for_registered_watchers() {
    let (env, client, admin) = setup();
    let watcher = Address::generate(&env);
    let other = Address::generate(&env);
    client.register_watcher(&admin, &watcher, &None);
    assert!(client.is_authorized(&watcher));
    assert!(!client.is_authorized(&other));
}

#[test]
fn renew_watcher_with_none_makes_it_permanent_again() {
    let (env, client, admin) = setup();
    let watcher = Address::generate(&env);

    // Register with an expiry at ledger 50.
    client.register_watcher(&admin, &watcher, &Some(50));
    assert!(client.is_authorized(&watcher));

    // Renew with None to clear the expiry, making it permanent again.
    client.renew_watcher(&admin, &watcher, &None);

    // Advance the ledger past the original expiry and assert still authorized.
    env.ledger().set_sequence_number(100);
    assert!(client.is_authorized(&watcher));

    // Renew with a future expiry and assert still authorized within the window.
    client.renew_watcher(&admin, &watcher, &Some(200));
    env.ledger().set_sequence_number(150);
    assert!(client.is_authorized(&watcher));
}

#[test]
fn paused_registry_blocks_mutations_but_allows_reads() {
    let (env, client, admin) = setup();
    let watcher = Address::generate(&env);
    client.register_watcher(&admin, &watcher, &None);

    client.pause(&admin);
    assert!(client.is_paused());

    // Mutations are blocked while paused.
    let new_watcher = Address::generate(&env);
    assert_eq!(
        client.try_register_watcher(&admin, &new_watcher, &None),
        Err(Ok(ContractError::ContractPaused))
    );
    assert_eq!(
        client.try_remove_watcher(&admin, &watcher),
        Err(Ok(ContractError::ContractPaused))
    );
    let new_admin = Address::generate(&env);
    assert_eq!(
        client.try_add_admin(&admin, &new_admin),
        Err(Ok(ContractError::ContractPaused))
    );
    let mut batch = Vec::new(&env);
    batch.push_back(Address::generate(&env));
    assert_eq!(
        client.try_batch_register_watchers(&admin, &batch),
        Err(Ok(ContractError::ContractPaused))
    );

    // Reads still succeed while paused.
    assert!(client.is_watcher_authorized(&watcher));
    assert_eq!(client.get_watchers().len(), 1);
    assert!(client.is_paused());
}
