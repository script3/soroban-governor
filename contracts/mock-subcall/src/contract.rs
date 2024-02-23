use sep_41_token::TokenClient;
use soroban_sdk::{contract, contracterror, contractimpl, panic_with_error, Address, Env};

use crate::storage;

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

    pub fn no_auth_sc(e: Env, amount: i128) {
        let governor = storage::get_governor(&e);

        let token = storage::get_token(&e);
        let token_client = TokenClient::new(&e, &token);
        token_client.transfer(&governor, &e.current_contract_address(), &amount);
    }

    pub fn call_subcall(e: Env, subcall_address: Address, amount: i128, auth: bool) {
        let governor = storage::get_governor(&e);
        governor.require_auth();

        let subcall_client = SubcallContractClient::new(&e, &subcall_address);
        if auth {
            subcall_client.subcall(&amount);
        } else {
            subcall_client.no_auth_sc(&amount);
        }
    }
}
