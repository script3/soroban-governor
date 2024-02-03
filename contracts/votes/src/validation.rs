use soroban_sdk::{panic_with_error, Env};

use crate::error::TokenVotesError;

pub fn require_nonnegative_amount(e: &Env, amount: i128) {
    if amount < 0 {
        panic_with_error!(e, TokenVotesError::NegativeAmountError);
    }
}
