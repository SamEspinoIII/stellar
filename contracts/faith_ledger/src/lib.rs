#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol};

#[contract]
pub struct FaithLedger;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    DonorBal(Address),
    BucketBal(u32),
    TokenAddress,
}

#[contractimpl]
impl FaithLedger {
    // Initializes the contract with the accepted asset token address (e.g., USDC or PHPC)
    pub fn initialize(env: Env, token: Address) {
        if env.storage().instance().has(&DataKey::TokenAddress) {
            panic!("Contract already initialized");
        }
        env.storage().instance().set(&DataKey::TokenAddress, &token);
    }

    // Accepts a donation, transfers tokens to contract storage, and updates specific ministry tracking
    pub fn donate(env: Env, donor: Address, amount: i128, bucket_id: u32) {
        donor.require_auth();
        
        if amount <= 0 {
            panic!("Donation amount must be greater than zero");
        }
        if bucket_id > 3 {
            panic!("Invalid ministry bucket ID. Use 1 (General), 2 (Building), or 3 (Outreach)");
        }

        let token_addr: Address = env.storage().instance().get(&DataKey::TokenAddress).unwrap();
        let client = token::Client::new(&env, &token_addr);
        
        // Transfer funds from donor to this smart contract
        client.transfer(&donor, &env.current_contract_address(), &amount);

        // Update total amount given by this specific donor
        let donor_key = DataKey::DonorBal(donor.clone());
        let current_donor_bal: i128 = env.storage().persistent().get(&donor_key).unwrap_or(0);
        env.storage().persistent().set(&donor_key, &(current_donor_bal + amount));

        // Update target ministry allocation bucket balance
        let bucket_key = DataKey::BucketBal(bucket_id);
        let current_bucket_bal: i128 = env.storage().persistent().get(&bucket_key).unwrap_or(0);
        env.storage().persistent().set(&bucket_key, &(current_bucket_bal + amount));

        // Emit an on-chain event for real-time app notifications
        env.events().publish(
            (Symbol::new(&env, "donation"), donor, bucket_id),
            amount,
        );
    }

    // Read function to get the current allocation inside a specific ministry bucket
    pub fn get_bucket_balance(env: Env, bucket_id: u32) -> i128 {
        env.storage().persistent().get(&DataKey::BucketBal(bucket_id)).unwrap_or(0)
    }

    // Read function to get total lifetime contribution profile of a single donor
    pub fn get_donor_balance(env: Env, donor: Address) -> i128 {
        env.storage().persistent().get(&DataKey::DonorBal(donor)).unwrap_or(0)
    }
}