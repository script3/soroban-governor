use crate::{error::TokenVotesError, storage};
use soroban_sdk::{panic_with_error, Address, Env};

pub fn receive_balance(e: &Env, address: &Address, amount: i128) {
    let balance = storage::get_balance(e, address);
    storage::set_balance(e, address, &(balance + amount));
}

pub fn spend_balance(e: &Env, address: &Address, amount: i128) {
    let balance = storage::get_balance(e, address);
    if balance < amount {
        panic_with_error!(e, TokenVotesError::BalanceError);
    }
    storage::set_balance(e, address, &(balance - amount));
}
