use soroban_sdk::{panic_with_error, Address, Env, Vec};

use crate::{
    checkpoints::{add_supply_checkpoint, add_user_checkpoint, Checkpoint},
    error::TokenVotesError,
    events::VoterTokenEvents,
    storage,
};

/// Mint voting units to an address
pub fn mint_voting_units(e: &Env, to: &Address, amount: i128) {
    if amount > 0 {
        let total_supply_checkpoint = storage::get_total_supply(e);
        let (_, mut supply) = total_supply_checkpoint.to_checkpoint_data();

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
    }
}

/// Burn voting units from an address
pub fn burn_voting_units(e: &Env, from: &Address, amount: i128) {
    if amount > 0 {
        let total_supply_checkpoint = storage::get_total_supply(e);
        let (_, mut supply) = total_supply_checkpoint.to_checkpoint_data();

        supply -= amount;
        if supply < 0 {
            panic_with_error!(e, TokenVotesError::InsufficientVotesError);
        }
        storage::set_total_supply(e, &total_supply_checkpoint);

        let vote_ledgers = storage::get_vote_ledgers(e);
        add_supply_checkpoint(e, &vote_ledgers, total_supply_checkpoint);
        move_voting_units(
            e,
            &vote_ledgers,
            Some(&storage::get_delegate(e, from)),
            None,
            amount,
        );
    }
}

/// Transfer voting units as the result of a transfer or delegation taken between "from" and "to"
pub fn transfer_voting_units(e: &Env, from: &Address, to: &Address, amount: i128) {
    let vote_ledgers = storage::get_vote_ledgers(e);
    move_voting_units(
        e,
        &vote_ledgers,
        Some(&storage::get_delegate(e, from)),
        Some(&storage::get_delegate(e, to)),
        amount,
    );
}

/// Move voting units from one address to another
pub fn move_voting_units(
    e: &Env,
    vote_ledgers: &Vec<u32>,
    from: Option<&Address>,
    to: Option<&Address>,
    amount: i128,
) {
    if from != to && amount > 0 {
        if let Some(from) = from {
            // Decrease voting units of `from` and push their old units
            // to the checkpoint
            let user_checkpoint = storage::get_voting_units(e, from);
            let (_, mut voting_units) = user_checkpoint.to_checkpoint_data();
            let prev_voting_units = voting_units.clone();

            voting_units -= amount;
            if voting_units < 0 {
                panic_with_error!(e, TokenVotesError::InsufficientVotesError);
            }

            storage::set_voting_units(
                e,
                from,
                &u128::from_checkpoint_data(e, e.ledger().sequence(), voting_units),
            );
            add_user_checkpoint(e, vote_ledgers, from, user_checkpoint);

            VoterTokenEvents::votes_changed(e, from.clone(), prev_voting_units, voting_units);
        }
        if let Some(to) = to {
            // Increase voting units of `to` and push their old units
            // to the checkpoint
            let user_checkpoint = storage::get_voting_units(e, to);
            let (_, mut voting_units) = user_checkpoint.to_checkpoint_data();
            let prev_voting_units = voting_units.clone();

            voting_units += amount;

            storage::set_voting_units(
                e,
                to,
                &u128::from_checkpoint_data(e, e.ledger().sequence(), voting_units),
            );
            add_user_checkpoint(e, vote_ledgers, to, user_checkpoint);

            VoterTokenEvents::votes_changed(e, to.clone(), prev_voting_units, voting_units);
        }
    }
}
