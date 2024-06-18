use crate::{
    checkpoints::{add_supply_checkpoint, Checkpoint},
    error::TokenVotesError,
    storage,
    voting_units::move_voting_units,
};
use soroban_sdk::{panic_with_error, Address, Env};

#[cfg(feature = "bonding")]
use crate::emissions;

/// Add tokens to an address's balance and update emissions and voting power
///
/// ### Arguments
/// * `to` - The address to add the balance to
/// * `amount` - The amount to add. This function does nothing if the amount is not greater than 0.
///
/// ### Panics
/// If the total suppy exceeds the maximum value of the checkpoint
pub fn mint_balance(e: &Env, to: &Address, amount: i128) {
    if amount > 0 {
        let balance = storage::get_balance(e, to);
        let total_supply_checkpoint = storage::get_total_supply(e);
        let (_, mut supply) = total_supply_checkpoint.to_checkpoint_data();

        #[cfg(feature = "bonding")]
        emissions::update_emissions(e, supply, to, balance);

        supply = supply
            .checked_add(amount)
            .unwrap_or_else(|| panic_with_error!(e, TokenVotesError::OverflowError));
        storage::set_total_supply(
            e,
            &u128::from_checkpoint_data(e, e.ledger().sequence(), supply),
        );

        let vote_ledgers = storage::get_vote_ledgers(e);
        add_supply_checkpoint(e, &vote_ledgers, total_supply_checkpoint);
        move_voting_units(
            e,
            &vote_ledgers,
            None,
            Some(&storage::get_delegate(e, to)),
            amount,
        );

        storage::set_balance(e, to, &(balance + amount));
    }
}

/// Remove tokens from an address's balance and update emissions and voting power
///
/// ### Arguments
/// * `from` - The address to remove the balance from
/// * `amount` - The amount to remove. This function does nothing if the amount is not greater than 0.
///
/// ### Panics
/// This function panics if the balance is less than the amount to remove.
pub fn burn_balance(e: &Env, from: &Address, amount: i128) {
    if amount > 0 {
        let balance = storage::get_balance(e, from);
        if balance < amount {
            panic_with_error!(e, TokenVotesError::BalanceError);
        }

        let total_supply_checkpoint = storage::get_total_supply(e);
        let (_, mut supply) = total_supply_checkpoint.to_checkpoint_data();

        #[cfg(feature = "bonding")]
        emissions::update_emissions(e, supply, from, balance);

        supply -= amount;
        if supply < 0 {
            panic_with_error!(e, TokenVotesError::InsufficientVotesError);
        }
        storage::set_total_supply(
            e,
            &u128::from_checkpoint_data(e, e.ledger().sequence(), supply),
        );

        let vote_ledgers = storage::get_vote_ledgers(e);
        add_supply_checkpoint(e, &vote_ledgers, total_supply_checkpoint);
        move_voting_units(
            e,
            &vote_ledgers,
            Some(&storage::get_delegate(e, from)),
            None,
            amount,
        );

        storage::set_balance(e, from, &(balance - amount));
    }
}

#[cfg(feature = "sep-0041")]
pub fn transfer_balance(e: &Env, from: &Address, to: &Address, amount: i128) {
    if amount > 0 {
        let from_balance = storage::get_balance(e, from);
        if from_balance < amount {
            panic_with_error!(e, TokenVotesError::BalanceError);
        }
        storage::set_balance(e, from, &(from_balance - amount));

        let to_balance = storage::get_balance(e, to);
        storage::set_balance(e, to, &(to_balance + amount));

        #[cfg(feature = "bonding")]
        {
            let total_supply_checkpoint = storage::get_total_supply(e);
            let (_, supply) = total_supply_checkpoint.to_checkpoint_data();
            emissions::update_emissions(e, supply, from, from_balance);
            if from != to {
                emissions::update_emissions(e, supply, to, to_balance);
            }
        }

        let vote_ledgers = storage::get_vote_ledgers(e);
        move_voting_units(
            e,
            &vote_ledgers,
            Some(&storage::get_delegate(e, from)),
            Some(&storage::get_delegate(e, to)),
            amount,
        );
    }
}
