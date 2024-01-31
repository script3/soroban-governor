use crate::{error::TokenVotesError, storage};
use soroban_sdk::{panic_with_error, Address, Env};

pub fn create_allowance(
    e: &Env,
    from: &Address,
    spender: &Address,
    amount: i128,
    expiration_ledger: u32,
) {
    if amount > 0 && expiration_ledger < e.ledger().sequence() {
        panic_with_error!(e, TokenVotesError::AllowanceError);
    }

    storage::set_allowance(e, from, spender, amount, expiration_ledger);
}

pub fn spend_allowance(e: &Env, from: &Address, spender: &Address, amount: i128) {
    let allowance = storage::get_allowance(e, from, spender);
    if allowance.amount < amount || e.ledger().sequence() > allowance.expiration_ledger {
        panic_with_error!(e, TokenVotesError::AllowanceError);
    }
    storage::set_allowance(
        e,
        from,
        spender,
        allowance.amount - amount,
        allowance.expiration_ledger,
    );
}
