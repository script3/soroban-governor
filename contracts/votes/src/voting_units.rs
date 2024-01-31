use soroban_sdk::{panic_with_error, Address, Env};

use crate::{error::TokenVotesError, events::VoterTokenEvents, storage};

/// Mint voting units to an address
pub fn mint_voting_units(e: &Env, to: &Address, amount: i128) {
    if amount > 0 {
        let mut total_supply = storage::get_total_supply(e);
        let prev_supply = total_supply.clone();

        total_supply.amount = total_supply
            .amount
            .checked_add(amount)
            .unwrap_or_else(|| panic_with_error!(e, TokenVotesError::OverflowError));
        total_supply.timestamp = e.ledger().timestamp();
        storage::set_total_supply(e, &total_supply);

        let mut supply_checkpoints = storage::get_total_supply_checkpoints(e);
        supply_checkpoints.push_back(prev_supply.clone());
        storage::set_total_supply_checkpoints(e, &supply_checkpoints);

        move_voting_units(e, None, Some(&storage::get_delegate(e, to)), amount);
    }
}

/// Burn voting units from an address
pub fn burn_voting_units(e: &Env, from: &Address, amount: i128) {
    if amount > 0 {
        let mut total_supply = storage::get_total_supply(e);
        let prev_supply = total_supply.clone();

        total_supply.amount -= amount;
        total_supply.timestamp = e.ledger().timestamp();
        if total_supply.amount < 0 {
            panic_with_error!(e, TokenVotesError::InsufficientVotesError);
        }
        storage::set_total_supply(e, &total_supply);

        let mut supply_checkpoints = storage::get_total_supply_checkpoints(e);
        supply_checkpoints.push_back(prev_supply.clone());
        storage::set_total_supply_checkpoints(e, &supply_checkpoints);

        move_voting_units(e, Some(&storage::get_delegate(e, from)), None, amount);
    }
}

/// Transfer voting units as the result of a transfer or delegation taken between "from" and "to"
pub fn transfer_voting_units(e: &Env, from: &Address, to: &Address, amount: i128) {
    move_voting_units(
        e,
        Some(&storage::get_delegate(e, from)),
        Some(&storage::get_delegate(e, to)),
        amount,
    );
}

/// Move voting units from one address to another
pub fn move_voting_units(e: &Env, from: Option<&Address>, to: Option<&Address>, amount: i128) {
    if from != to && amount > 0 {
        if let Some(from) = from {
            // Decrease voting units of `from` and push their old units
            // to the checkpoint
            let mut voting_units = storage::get_voting_units(e, from);
            let prev_units = voting_units.clone();

            voting_units.amount -= amount;
            voting_units.timestamp = e.ledger().timestamp();
            if voting_units.amount < 0 {
                panic_with_error!(e, TokenVotesError::InsufficientVotesError);
            }
            storage::set_voting_units(e, from, &voting_units);

            let mut voting_checkpoints = storage::get_voting_units_checkpoints(e, from);
            voting_checkpoints.push_back(prev_units.clone());
            storage::set_voting_units_checkpoints(e, from, &voting_checkpoints);

            VoterTokenEvents::votes_changed(
                e,
                from.clone(),
                prev_units.amount,
                voting_units.amount,
            );
        }
        if let Some(to) = to {
            // Increase voting units of `to` and push their old units
            // to the checkpoint
            let mut voting_units = storage::get_voting_units(e, to);
            let prev_units = voting_units.clone();

            voting_units.amount += amount;
            voting_units.timestamp = e.ledger().timestamp();
            storage::set_voting_units(e, to, &voting_units);

            let mut voting_checkpoints = storage::get_voting_units_checkpoints(e, to);
            voting_checkpoints.push_back(prev_units.clone());
            storage::set_voting_units_checkpoints(e, to, &voting_checkpoints);

            VoterTokenEvents::votes_changed(e, to.clone(), prev_units.amount, voting_units.amount);
        }
    }
}
