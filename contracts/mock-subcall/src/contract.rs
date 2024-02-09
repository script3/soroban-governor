use sep_41_token::TokenClient;
use soroban_sdk::{contract, contracterror, contractimpl, panic_with_error, Address, Env};

use crate::storage;

/// The error codes for the contract.
#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ContractError {
    Error = 256,
}

#[contract]
pub struct SubcallContract;

#[contractimpl]
impl SubcallContract {
    pub fn initialize(e: Env, token: Address, governor: Address) {
        if storage::get_is_init(&e) {
            panic_with_error!(&e, ContractError::Error);
        }

        storage::set_token(&e, &token);
        storage::set_governor(&e, &governor);
        storage::set_is_init(&e);
        storage::extend_instance(&e);
    }

    pub fn subcall(e: Env, amount: i128) {
        let governor = storage::get_governor(&e);
        governor.require_auth();

        let token = storage::get_token(&e);
        let token_client = TokenClient::new(&e, &token);
        token_client.transfer(&governor, &e.current_contract_address(), &amount);
    }

    pub fn call(e: Env, amount: i128) {
        let governor = storage::get_governor(&e);
        governor.require_auth();

        let token = storage::get_token(&e);
        let token_client = TokenClient::new(&e, &token);
        token_client.transfer(&e.current_contract_address(), &governor, &amount);
    }
}
