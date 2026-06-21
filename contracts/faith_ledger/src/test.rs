#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, token, Address, Env};

fn setup_test_env<'a>() -> (Env, FaithLedgerClient<'a>, token::Client<'a>, token::AdminClient<'a>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, FaithLedger);
    let client = FaithLedgerClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::Client::new(&env, &token_id);
    let token_admin_client = token::AdminClient::new(&env, &token_id);

    let donor = Address::generate(&env);
    token_admin_client.mint(&donor, &10000);

    client.initialize(&token_id);

    (env, client, token_client, token_admin_client, donor, contract_id)
}

#[test]
fn test_donation_happy_path() {
    let (env, client, token_client, _, donor, contract_id) = setup_test_env();

    // Execute MVP donation path to Bucket 1 (General Ministry Fund)
    client.donate(&donor, &500, &1);

    // Verify balances updated correctly
    assert_eq!(token_client.balance(&donor), 9500);
    assert_eq!(token_client.balance(&contract_id), 500);
}

#[test]
#[should_panic(expected = "Donation amount must be greater than zero")]
fn test_donation_fails_with_zero_amount() {
    let (_env, client, _, _, donor, _) = setup_test_env();
    
    // Attempting to donate zero tokens should trigger a smart contract panic
    client.donate(&donor, &0, &1);
}

#[test]
#[should_panic(expected = "Invalid ministry bucket ID")]
fn test_donation_fails_invalid_bucket() {
    let (_env, client, _, _, donor, _) = setup_test_env();
    
    // Out-of-bounds bucket mapping ID selection
    client.donate(&donor, &200, &99);
}

#[test]
fn test_state_verification_donor_tracking() {
    let (_env, client, _, _, donor, _) = setup_test_env();

    // Sequential donations to test structural state mutation
    client.donate(&donor, &300, &1);
    client.donate(&donor, &200, &2);

    // Verify the cumulative tracking profile totals exactly 500
    assert_eq!(client.get_donor_balance(&donor), 500);
}

#[test]
fn test_state_verification_bucket_aggregation() {
    let (env, client, _, token_admin_client, donor_1, _) = setup_test_env();
    
    let donor_2 = Address::generate(&env);
    token_admin_client.mint(&donor_2, &5000);

    // Two distinct users giving to the same designated target pool
    client.donate(&donor_1, &400, &3);
    client.donate(&donor_2, &600, &3);

    // Verify aggregate storage matches the inputs perfectly
    assert_eq!(client.get_bucket_balance(&3), 1000);
}