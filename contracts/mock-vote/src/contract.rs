use crate::storage;
use soroban_sdk::{contract, contractimpl, Address, Env};
#[contract]
pub struct MockTokenVotes;

pub trait MockVotesTrait {
    /// Get the current voting power of an account
    ///
    /// ### Arguments
    /// * `account` - The address of the account
    fn get_votes(e: Env, user: Address) -> i128;

    /// Set the current voting power of an account
    ///
    /// ### Arguments
    /// * `account` - The address of the account
    /// * `amount` - The voting power of the account
    fn set_votes(e: Env, user: Address, amount: i128);

    /// Get the voting power of an account at a specific ledger sequence number
    ///
    /// ### Arguments
    /// * `account` - The address of the account
    /// * `sequence` - The ledger sequence number to get the voting power at
    fn get_past_votes(e: Env, user: Address, sequence: u64) -> i128;

    /// Get the voting power of an account at a specific ledger sequence number
    /// ### Arguments
    /// * `account` - The address of the account
    /// * `sequence` - The ledger sequence number to get the voting power at
    /// * `amount` - The voting power of the account
    fn set_past_votes(e: Env, user: Address, sequence: u64, amount: i128);

    fn total_supply(e: Env) -> i128;
}

#[contractimpl]
impl MockVotesTrait for MockTokenVotes {
    fn get_votes(e: Env, user: Address) -> i128 {
        storage::get_votes(&e, &user)
    }

    fn set_votes(e: Env, user: Address, amount: i128) {
        storage::set_votes(&e, &user, &amount);
        let total_supply = storage::get_total_supply(&e) + amount;
        storage::set_total_supply(&e, &total_supply);
    }

    fn get_past_votes(e: Env, user: Address, timestamp: u64) -> i128 {
        storage::get_past_votes(&e, &user, &timestamp)
    }

    fn set_past_votes(e: Env, user: Address, timestamp: u64, amount: i128) {
        storage::set_past_votes(&e, &user, &timestamp, &amount);
    }

    fn total_supply(e: Env) -> i128 {
        storage::get_total_supply(&e)
    }
}
