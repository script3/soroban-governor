use soroban_sdk::{panic_with_error, Address, Env, Vec};

use crate::{
    checkpoints::{add_user_checkpoint, Checkpoint},
    error::TokenVotesError,
    events::TokenVotesEvents,
    storage,
};

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

            TokenVotesEvents::votes_changed(e, from.clone(), prev_voting_units, voting_units);
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

            TokenVotesEvents::votes_changed(e, to.clone(), prev_voting_units, voting_units);
        }
    }
}
