use soroban_sdk::{panic_with_error, unwrap::UnwrapOptimized, Address, Env, Vec};

use crate::{constants::MAX_CHECKPOINT_AGE_LEDGERS, error::TokenVotesError, storage};

pub trait Checkpoint {
    /// Convert a sequence and amount to a Checkpoint
    ///
    /// The amount value will be truncated to a u96.
    ///
    /// ### Arguments
    /// * sequence - The sequence to convert
    /// * amount - The amount to convert
    fn from_checkpoint_data(e: &Env, sequence: u32, amount: i128) -> Self;

    /// Convert a Checkpoint to a sequence and amount
    ///
    /// ### Returns
    /// * (sequence, amount) - The sequence and amount of the Checkpoint
    fn to_checkpoint_data(self) -> (u32, i128);
}

/// Stores the Checkpoint as a u128.
///
/// The Checkpoint encodeds the sequence (u32) and the amount (u96) into
/// a u128 such that the sequence is the most significant 32 bits and the
/// amount is the least significant 96 bits:
///
/// 0x{sequence}{amount}
///
/// The amount will be taken as an i128 and returned as an i128
/// to conform with the SEP-0041 token standard. However, the Checkpoint
/// will only be able to support amount values within the range of a u96.
impl Checkpoint for u128 {
    fn from_checkpoint_data(e: &Env, sequence: u32, amount: i128) -> Self {
        #[allow(overflowing_literals)]
        let temp = amount & 0xFFFFFFFF_00000000_00000000_00000000;
        if temp != 0 {
            panic_with_error!(e, TokenVotesError::InvalidCheckpointError);
        }
        (sequence as u128) << 96 | (amount as u128)
    }

    fn to_checkpoint_data(self) -> (u32, i128) {
        let sequence = (self >> 96) as u32;
        let amount = (self & 0x00000000_FFFFFFFF_FFFFFFFF_FFFFFFFF) as i128;
        (sequence, amount)
    }
}

/// Get the amount of the checkpoint that has a sequence number greater than or equal
/// to the given sequence.
///
/// Returns 0 if no checkpoint exists that meets the criteria.
///
/// ### Arguments
/// * checkpoints - The checkpoints to search
/// * sequence - The sequence to search for
pub fn upper_lookup(e: &Env, checkpoints: &Vec<u128>, sequence: u32) -> i128 {
    if checkpoints.is_empty() {
        return 0;
    }

    // search with max checkpoint value for the sequence to ensure that we catch any checkpoint
    // with a matching sequence as less than or equal to
    match checkpoints.binary_search(u128::from_checkpoint_data(
        &e,
        sequence,
        0xFFFFFFFF_FFFFFFFF_FFFFFFFF,
    )) {
        Ok(index) => checkpoints.get_unchecked(index).to_checkpoint_data().1,
        Err(index) => {
            if index == 0 {
                0
            } else {
                checkpoints.get_unchecked(index - 1).to_checkpoint_data().1
            }
        }
    }
}

/// Add "to_add" to the checkpoints vector for the user.
///
/// This function assumes that the caller is setting a new value for the persistent
/// entry on this ledger.
///
/// ### Arguments
/// * vote_ledgers - The vote ledgers
/// * user - The address of the user
/// * to_add - The voting units checkpoint to add
pub fn add_user_checkpoint(e: &Env, vote_ledgers: &Vec<u32>, user: &Address, to_add: u128) {
    let mut user_checkpoints = storage::get_voting_units_checkpoints(e, user);
    let needs_write = add_checkpoint(e, vote_ledgers, &mut user_checkpoints, &to_add);
    if needs_write {
        storage::set_voting_units_checkpoints(e, user, &user_checkpoints);
    }
}

/// Add "to_add" to the checkpoints vector for the total supple.
///
/// This function assumes that the caller is setting a new value for the persistent
/// entry on this ledger.
///
/// ### Arguments
/// * vote_ledgers - The vote ledgers
/// * to_add - The voting units checkpoint to add
pub fn add_supply_checkpoint(e: &Env, vote_ledgers: &Vec<u32>, to_add: u128) {
    let mut supply_checkpoints = storage::get_total_supply_checkpoints(e);
    let needs_write = add_checkpoint(e, vote_ledgers, &mut supply_checkpoints, &to_add);
    if needs_write {
        storage::set_total_supply_checkpoints(e, &supply_checkpoints);
    }
}

/// Add a vote ledger to the list of vote ledgers. This function will also prune any
/// vote ledgers that are at least `MAX_CHECKPOINT_AGE_LEDGERS` old.
///
/// ### Arguments
/// * sequence - The sequence of the vote ledger to add
pub fn add_vote_ledger(e: &Env, sequence: u32) {
    let mut vote_ledgers = storage::get_vote_ledgers(&e);
    let len = vote_ledgers.len();
    let ledger_cutoff = e
        .ledger()
        .sequence()
        .checked_sub(MAX_CHECKPOINT_AGE_LEDGERS);
    if len > 0 && ledger_cutoff.is_some() {
        // if the `ledger_cutoff` is found or if the index in which it could
        // be inserted is returned, we remove everything before it
        let result = vote_ledgers.binary_search(ledger_cutoff.unwrap_optimized());
        let index = match result {
            Ok(index) => index,
            Err(index) => index,
        };
        // check if there is anything to remove before doing the work
        if index > 0 {
            vote_ledgers = vote_ledgers.slice(index..len);
        }
    }

    // insert the new vote ledger in order
    match vote_ledgers.binary_search(&sequence) {
        // if the sequence is already in the list, we don't need to insert it
        Ok(_) => (),
        Err(index) => {
            vote_ledgers.insert(index, sequence);
        }
    }
    storage::set_vote_ledgers(&e, &vote_ledgers);
}

/// Appends "to_add" to the checkpoints vector in place. This function also
/// manages any pruning of old checkpoints that may be necessary.
///
/// This function assumes that the caller is setting a new value for the persistent
/// entry on this ledger.
///
/// Returns a bool if the checkpoint list was modified and should be written back to chain
///
/// ### Arguments
/// * checkpoints - The checkpoints to add to
/// * to_add - The voting units to add
fn add_checkpoint(
    e: &Env,
    vote_ledgers: &Vec<u32>,
    checkpoints: &mut Vec<u128>,
    to_add: &u128,
) -> bool {
    let mut needs_write = false;
    let mut len = checkpoints.len();

    let (to_add_seq, _) = to_add.to_checkpoint_data();

    let vote_ledgers_len = vote_ledgers.len();
    // If vote_ledgers_len == 0, then we do not need to track any checkpoints so nothing needs to be added.
    // We will still attempt to prune checkpoints if necessary
    if vote_ledgers_len != 0 {
        // Check if the checkpoint `to_add` is needed to ensure a safe vote history.
        // This occurs when there is a proposal vote start time in between the sequence of `to_add` (inclusive)
        // and the current ledger sequence (exclusive).
        let (vote_ledger_index, vote_ledger) = match vote_ledgers.binary_search(to_add_seq) {
            // exact match found
            Ok(index) => (index, vote_ledgers.get_unchecked(index)),
            // non-exact match found - index is where the value would be inserted
            Err(index) => {
                if index == vote_ledgers_len {
                    // `to_add` is greater than all vote_ledgers
                    // return zero to prevent `to_add` from being inserted
                    (0, 0)
                } else {
                    (index, vote_ledgers.get_unchecked(index))
                }
            }
        };
        if vote_ledger >= to_add_seq && vote_ledger < e.ledger().sequence() {
            // `to_add` is needed
            if len == 0 {
                checkpoints.push_back(to_add.clone());
            } else {
                let last = checkpoints.last_unchecked();
                let (last_seq, _) = last.to_checkpoint_data();
                if last_seq == to_add_seq {
                    // last entry is no longer relevant
                    checkpoints.pop_back();
                    len -= 1;
                } else if vote_ledger_index > 0 {
                    // check if last checkpoint is between the previous vote_ledger and next vote_ledger
                    // if so, the checkpoint being added will make the last checkpoint irrelevant
                    let prev_vote_ledger = vote_ledgers.get_unchecked(vote_ledger_index - 1);
                    if last_seq > prev_vote_ledger {
                        checkpoints.pop_back();
                        len -= 1;
                    }
                }
                // always add `to_add` to the end of the vector
                checkpoints.push_back(to_add.clone());
                len += 1;
            }
            needs_write = true;
        }
        // prune checkpoints older than the oldest vote_ledger entry
        // except the most recent checkpoint older than or equal to the oldest vote_ledger entry
        let max_sequence = vote_ledgers.first_unchecked();
        let search = u128::from_checkpoint_data(e, max_sequence, 0);
        let lower_bound_inclusive = match checkpoints.binary_search(search) {
            Ok(index) => index,
            Err(index) => {
                if index == 0 {
                    0
                } else {
                    index - 1
                }
            }
        };
        if lower_bound_inclusive != 0 {
            *checkpoints = checkpoints.slice(lower_bound_inclusive..len);
            needs_write = true;
        }
    }
    needs_write
}

#[cfg(test)]
mod tests {
    use std::println;

    use crate::{constants::ONE_DAY_LEDGERS, contract::TokenVotes};

    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger, LedgerInfo},
        vec, Address, Env, Vec,
    };

    const DEFAULT_LEDGER_INFO: LedgerInfo = LedgerInfo {
        timestamp: 1441065600,
        protocol_version: 20,
        sequence_number: 20 * 17280,
        network_id: [0_u8; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 1000,
        min_persistent_entry_ttl: 1000,
        max_entry_ttl: 100000000,
    };

    #[test]
    fn test_checkpoint_data_conversion() {
        let e = Env::default();

        let sequence: u32 = 1234567;
        let amount: i128 = 98765_0000000;
        let checkpoint = u128::from_checkpoint_data(&e, sequence, amount);

        let checkpoint_later = u128::from_checkpoint_data(&e, sequence + 1, 0);
        assert!(checkpoint < checkpoint_later);

        let (seq, amt) = checkpoint.to_checkpoint_data();
        assert_eq!(seq, sequence);
        assert_eq!(amt, amount);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #102)")]
    fn test_checkpoint_data_amount_too_large() {
        let e = Env::default();

        let sequence: u32 = 1234567;
        let amount: i128 = 2_i128.pow(96);
        u128::from_checkpoint_data(&e, sequence, amount);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #102)")]
    fn test_checkpoint_data_amount_negative() {
        let e = Env::default();

        let sequence: u32 = 1234567;
        let amount: i128 = -1;
        u128::from_checkpoint_data(&e, sequence, amount);
    }

    #[test]
    fn test_upper_lookup() {
        let e = Env::default();

        let mut checkpoints = Vec::<u128>::new(&e);
        checkpoints.push_back(u128::from_checkpoint_data(&e, 123, 8293480));
        checkpoints.push_back(u128::from_checkpoint_data(&e, 124, 1234567));
        checkpoints.push_back(u128::from_checkpoint_data(&e, 130, 9876543));
        assert_eq!(upper_lookup(&e, &checkpoints, 122), 0);
        assert_eq!(upper_lookup(&e, &checkpoints, 123), 8293480);
        assert_eq!(upper_lookup(&e, &checkpoints, 124), 1234567);
        assert_eq!(upper_lookup(&e, &checkpoints, 129), 1234567);
        assert_eq!(upper_lookup(&e, &checkpoints, 199), 9876543);
    }

    #[test]
    fn test_upper_lookup_empty() {
        let e = Env::default();

        let checkpoints = Vec::<u128>::new(&e);
        assert_eq!(upper_lookup(&e, &checkpoints, 0), 0);
    }

    #[test]
    fn test_add_user_checkpoint_needs_write_empty() {
        let e = Env::default();
        e.ledger().set(DEFAULT_LEDGER_INFO);

        let voting_ledgers = vec![&e, 172800 - 100];

        let votes = e.register_contract(None, TokenVotes {});
        let samwise = Address::generate(&e);

        e.as_contract(&votes, || {
            let to_add = u128::from_checkpoint_data(&e, 172800 - 200, 100);
            add_user_checkpoint(&e, &voting_ledgers, &samwise, to_add);

            let user_checkpoints = storage::get_voting_units_checkpoints(&e, &samwise);
            assert_eq!(user_checkpoints.len(), 1);
            assert_eq!(user_checkpoints.get_unchecked(0), to_add);
        });
    }

    #[test]
    fn test_add_user_checkpoint_no_write_empty() {
        let e = Env::default();
        e.ledger().set(DEFAULT_LEDGER_INFO);

        let voting_ledgers = vec![&e, DEFAULT_LEDGER_INFO.sequence_number - 100];

        let votes = e.register_contract(None, TokenVotes {});
        let samwise = Address::generate(&e);

        e.as_contract(&votes, || {
            let to_add =
                u128::from_checkpoint_data(&e, DEFAULT_LEDGER_INFO.sequence_number - 10, 100);
            add_user_checkpoint(&e, &voting_ledgers, &samwise, to_add);

            let user_checkpoints = storage::get_voting_units_checkpoints(&e, &samwise);
            assert_eq!(user_checkpoints.len(), 0);
        });
    }

    #[test]
    fn test_add_supply_checkpoint_needs_write() {
        let e = Env::default();
        e.ledger().set(DEFAULT_LEDGER_INFO);

        let voting_ledgers = vec![
            &e,
            DEFAULT_LEDGER_INFO.sequence_number - 300,
            DEFAULT_LEDGER_INFO.sequence_number - 100,
        ];
        let checkpoints = vec![
            &e,
            u128::from_checkpoint_data(&e, DEFAULT_LEDGER_INFO.sequence_number - 300, 0),
        ];

        let votes = e.register_contract(None, TokenVotes {});

        e.as_contract(&votes, || {
            storage::set_total_supply_checkpoints(&e, &checkpoints);
            let to_add =
                u128::from_checkpoint_data(&e, DEFAULT_LEDGER_INFO.sequence_number - 200, 100);
            add_supply_checkpoint(&e, &voting_ledgers, to_add);

            let supply_checkpoints = storage::get_total_supply_checkpoints(&e);
            assert_eq!(supply_checkpoints.len(), 2);
            assert_eq!(supply_checkpoints.last_unchecked(), to_add);
        });
    }

    #[test]
    fn test_add_supply_checkpoint_no_write_empty() {
        let e = Env::default();
        e.ledger().set(DEFAULT_LEDGER_INFO);

        let voting_ledgers = vec![&e];
        let checkpoints = vec![
            &e,
            u128::from_checkpoint_data(&e, DEFAULT_LEDGER_INFO.sequence_number - 300, 0),
        ];

        let votes = e.register_contract(None, TokenVotes {});

        e.as_contract(&votes, || {
            storage::set_total_supply_checkpoints(&e, &checkpoints);
            let to_add = u128::from_checkpoint_data(&e, 172800 - 200, 100);
            add_supply_checkpoint(&e, &voting_ledgers, to_add);

            let supply_checkpoints = storage::get_total_supply_checkpoints(&e);
            assert_eq!(supply_checkpoints.len(), 1);
        });
    }

    #[test]
    fn test_add_checkpoint_keeps_entry_for_oldest_voting_ledger() {
        let e = Env::default();
        e.ledger().set(DEFAULT_LEDGER_INFO);

        let ledger = DEFAULT_LEDGER_INFO.sequence_number;
        let mut voting_ledgers = Vec::<u32>::new(&e);
        voting_ledgers.push_back(ledger);
        voting_ledgers.push_back(ledger + 3 * ONE_DAY_LEDGERS);
        voting_ledgers.push_back(ledger + 5 * ONE_DAY_LEDGERS);
        let mut checkpoints = Vec::<u128>::new(&e);
        let first = u128::from_checkpoint_data(&e, ledger, 123);
        checkpoints.push_back(first);
        checkpoints.push_back(u128::from_checkpoint_data(
            &e,
            ledger + 3 * ONE_DAY_LEDGERS - 1,
            456,
        ));

        let mut new_ledger_info = DEFAULT_LEDGER_INFO.clone();
        new_ledger_info.sequence_number += 8 * ONE_DAY_LEDGERS;
        e.ledger().set(new_ledger_info);

        let to_add = u128::from_checkpoint_data(&e, ledger + 4 * ONE_DAY_LEDGERS + 100, 42);
        let needs_write = add_checkpoint(&e, &voting_ledgers, &mut checkpoints, &to_add);

        assert!(needs_write);
        assert_eq!(checkpoints.len(), 3);
        let vote_last = checkpoints.last_unchecked();
        assert_eq!(vote_last.to_checkpoint_data(), to_add.to_checkpoint_data());
        let vote_first = checkpoints.first_unchecked();
        assert_eq!(vote_first.to_checkpoint_data(), first.to_checkpoint_data());
    }

    #[test]
    fn test_add_checkpoint_prunes_old_entries() {
        let e = Env::default();
        e.ledger().set(DEFAULT_LEDGER_INFO);

        let ledger = DEFAULT_LEDGER_INFO.sequence_number;
        let mut voting_ledgers = Vec::<u32>::new(&e);
        voting_ledgers.push_back(ledger + 3 * ONE_DAY_LEDGERS);
        voting_ledgers.push_back(ledger + 5 * ONE_DAY_LEDGERS);
        let mut checkpoints = Vec::<u128>::new(&e);
        checkpoints.push_back(u128::from_checkpoint_data(&e, ledger, 123));
        checkpoints.push_back(u128::from_checkpoint_data(
            &e,
            ledger + 3 * ONE_DAY_LEDGERS - 1,
            456,
        ));

        let mut new_ledger_info = DEFAULT_LEDGER_INFO.clone();
        new_ledger_info.sequence_number += 8 * ONE_DAY_LEDGERS;
        e.ledger().set(new_ledger_info);

        let to_add = u128::from_checkpoint_data(&e, ledger + 4 * ONE_DAY_LEDGERS + 100, 42);
        let needs_write = add_checkpoint(&e, &voting_ledgers, &mut checkpoints, &to_add);

        assert!(needs_write);
        assert_eq!(checkpoints.len(), 2);
        let vote_last = checkpoints.last_unchecked();
        assert_eq!(vote_last.to_checkpoint_data(), to_add.to_checkpoint_data());
        let vote_first = checkpoints.first_unchecked();
        assert_eq!(
            vote_first.to_checkpoint_data(),
            (ledger + 3 * ONE_DAY_LEDGERS - 1, 456)
        );
    }

    #[test]
    fn test_add_checkpoint_replaces_entries() {
        let e = Env::default();
        e.ledger().set(DEFAULT_LEDGER_INFO);

        let ledger = DEFAULT_LEDGER_INFO.sequence_number;
        let mut voting_ledgers = Vec::<u32>::new(&e);
        voting_ledgers.push_back(ledger + 3 * ONE_DAY_LEDGERS);
        voting_ledgers.push_back(ledger + 5 * ONE_DAY_LEDGERS);
        let mut checkpoints = Vec::<u128>::new(&e);
        checkpoints.push_back(u128::from_checkpoint_data(
            &e,
            ledger + 3 * ONE_DAY_LEDGERS - 1,
            456,
        ));
        checkpoints.push_back(u128::from_checkpoint_data(
            &e,
            ledger + 3 * ONE_DAY_LEDGERS + 100,
            123,
        ));

        let mut new_ledger_info = DEFAULT_LEDGER_INFO.clone();
        new_ledger_info.sequence_number += 8 * ONE_DAY_LEDGERS;
        e.ledger().set(new_ledger_info);

        let to_add = u128::from_checkpoint_data(&e, ledger + 4 * ONE_DAY_LEDGERS + 100, 42);
        let needs_write = add_checkpoint(&e, &voting_ledgers, &mut checkpoints, &to_add);

        assert!(needs_write);
        assert_eq!(checkpoints.len(), 2);
        let vote_last = checkpoints.last_unchecked();
        assert_eq!(vote_last.to_checkpoint_data(), to_add.to_checkpoint_data());
        let vote_first = checkpoints.first_unchecked();
        assert_eq!(
            vote_first.to_checkpoint_data(),
            (ledger + 3 * ONE_DAY_LEDGERS - 1, 456)
        );
    }

    #[test]
    fn test_add_checkpoint_replaces_same_sequence() {
        let e = Env::default();
        e.ledger().set(DEFAULT_LEDGER_INFO);

        let ledger = DEFAULT_LEDGER_INFO.sequence_number;
        let mut voting_ledgers = Vec::<u32>::new(&e);
        voting_ledgers.push_back(ledger + 3 * ONE_DAY_LEDGERS);
        voting_ledgers.push_back(ledger + 5 * ONE_DAY_LEDGERS);
        let mut checkpoints = Vec::<u128>::new(&e);
        checkpoints.push_back(u128::from_checkpoint_data(
            &e,
            ledger + 3 * ONE_DAY_LEDGERS - 1,
            456,
        ));
        checkpoints.push_back(u128::from_checkpoint_data(
            &e,
            ledger + 3 * ONE_DAY_LEDGERS + 100,
            123,
        ));

        let mut new_ledger_info = DEFAULT_LEDGER_INFO.clone();
        new_ledger_info.sequence_number += 8 * ONE_DAY_LEDGERS;
        e.ledger().set(new_ledger_info);

        let to_add = u128::from_checkpoint_data(&e, ledger + 3 * ONE_DAY_LEDGERS + 100, 42);
        let needs_write = add_checkpoint(&e, &voting_ledgers, &mut checkpoints, &to_add);

        assert!(needs_write);
        assert_eq!(checkpoints.len(), 2);
        let vote_last = checkpoints.last_unchecked();
        assert_eq!(vote_last.to_checkpoint_data(), to_add.to_checkpoint_data());
        let vote_first = checkpoints.first_unchecked();
        assert_eq!(
            vote_first.to_checkpoint_data(),
            (ledger + 3 * ONE_DAY_LEDGERS - 1, 456)
        );
    }

    #[test]
    fn test_add_checkpoint_vote_ledger_same_as_cur_ledger_skips() {
        let e = Env::default();
        e.ledger().set(DEFAULT_LEDGER_INFO);

        let ledger = DEFAULT_LEDGER_INFO.sequence_number;
        let mut voting_ledgers = Vec::<u32>::new(&e);
        voting_ledgers.push_back(ledger);
        let mut checkpoints = Vec::<u128>::new(&e);
        checkpoints.push_back(u128::from_checkpoint_data(&e, ledger - 123, 456));

        // checkpoint not needed, vote ledger captured by new entry
        let to_add = u128::from_checkpoint_data(&e, ledger, 42);
        let needs_write = add_checkpoint(&e, &voting_ledgers, &mut checkpoints, &to_add);

        assert!(!needs_write);
        assert_eq!(checkpoints.len(), 1);
        let vote_first = checkpoints.first_unchecked();
        assert_eq!(vote_first.to_checkpoint_data(), (ledger - 123, 456));
    }

    #[test]
    fn test_add_vote_ledger() {
        let e = Env::default();
        e.ledger().set(DEFAULT_LEDGER_INFO);

        let ledger = DEFAULT_LEDGER_INFO.sequence_number;

        let mut vote_ledgers = Vec::<u32>::new(&e);
        vote_ledgers.push_back(ledger - MAX_CHECKPOINT_AGE_LEDGERS);
        vote_ledgers.push_back(ledger - ONE_DAY_LEDGERS / 2);
        vote_ledgers.push_back(ledger - 10);
        vote_ledgers.push_back(ledger + ONE_DAY_LEDGERS);

        let votes = e.register_contract(None, TokenVotes {});
        e.as_contract(&votes, || {
            storage::set_vote_ledgers(&e, &vote_ledgers);

            let to_add_sequence = ledger + ONE_DAY_LEDGERS + 100;
            add_vote_ledger(&e, to_add_sequence);

            // perform expected change to vote_ledgers array
            vote_ledgers.push_back(to_add_sequence);

            let new_vote_ledgers = storage::get_vote_ledgers(&e);
            println!("new_vote_ledgers: {:?}", new_vote_ledgers);
            println!("vote_ledgers: {:?}", vote_ledgers);
            assert_eq!(new_vote_ledgers.len(), vote_ledgers.len());
            for i in 0..vote_ledgers.len() {
                assert_eq!(
                    new_vote_ledgers.get_unchecked(i),
                    vote_ledgers.get_unchecked(i)
                );
            }
        });
    }

    #[test]
    fn test_add_vote_ledger_prunes() {
        let e = Env::default();
        e.ledger().set(DEFAULT_LEDGER_INFO);

        let ledger = DEFAULT_LEDGER_INFO.sequence_number;

        let mut vote_ledgers = Vec::<u32>::new(&e);
        vote_ledgers.push_back(ledger - MAX_CHECKPOINT_AGE_LEDGERS - 100);
        vote_ledgers.push_back(ledger - MAX_CHECKPOINT_AGE_LEDGERS - 1);
        vote_ledgers.push_back(ledger - ONE_DAY_LEDGERS / 2);
        vote_ledgers.push_back(ledger - 10);
        vote_ledgers.push_back(ledger + ONE_DAY_LEDGERS);

        let votes = e.register_contract(None, TokenVotes {});
        e.as_contract(&votes, || {
            storage::set_vote_ledgers(&e, &vote_ledgers);

            let to_add_sequence = ledger + ONE_DAY_LEDGERS + 100;
            add_vote_ledger(&e, to_add_sequence);

            // perform expected change to vote_ledgers array
            vote_ledgers.pop_front_unchecked();
            vote_ledgers.pop_front_unchecked();
            vote_ledgers.push_back(to_add_sequence);

            let new_vote_ledgers = storage::get_vote_ledgers(&e);
            println!("new_vote_ledgers: {:?}", new_vote_ledgers);
            println!("vote_ledgers: {:?}", vote_ledgers);
            assert_eq!(new_vote_ledgers.len(), vote_ledgers.len());
            for i in 0..vote_ledgers.len() {
                assert_eq!(
                    new_vote_ledgers.get_unchecked(i),
                    vote_ledgers.get_unchecked(i)
                );
            }
        });
    }

    #[test]
    fn test_add_vote_ledger_same_ledger() {
        let e = Env::default();
        e.ledger().set(DEFAULT_LEDGER_INFO);

        let ledger = DEFAULT_LEDGER_INFO.sequence_number;

        let mut vote_ledgers = Vec::<u32>::new(&e);
        vote_ledgers.push_back(ledger - ONE_DAY_LEDGERS / 2);
        vote_ledgers.push_back(ledger - 10);
        vote_ledgers.push_back(ledger + ONE_DAY_LEDGERS + 100);

        let votes = e.register_contract(None, TokenVotes {});
        e.as_contract(&votes, || {
            storage::set_vote_ledgers(&e, &vote_ledgers);

            let to_add_sequence = ledger + ONE_DAY_LEDGERS + 100;
            add_vote_ledger(&e, to_add_sequence);

            // perform expected change to vote_ledgers array

            let new_vote_ledgers = storage::get_vote_ledgers(&e);
            assert_eq!(new_vote_ledgers.len(), vote_ledgers.len());
            for i in 0..vote_ledgers.len() {
                assert_eq!(
                    new_vote_ledgers.get_unchecked(i),
                    vote_ledgers.get_unchecked(i)
                );
            }
        });
    }

    #[test]
    fn test_add_vote_ledger_underflow_math() {
        let e = Env::default();
        let mut modified_ledger_info = DEFAULT_LEDGER_INFO.clone();
        modified_ledger_info.sequence_number = ONE_DAY_LEDGERS;
        e.ledger().set(modified_ledger_info);

        let ledger = DEFAULT_LEDGER_INFO.sequence_number;

        let mut vote_ledgers = Vec::<u32>::new(&e);
        vote_ledgers.push_back(ledger - ONE_DAY_LEDGERS / 2);
        vote_ledgers.push_back(ledger - 10);
        vote_ledgers.push_back(ledger + ONE_DAY_LEDGERS);

        let votes = e.register_contract(None, TokenVotes {});
        e.as_contract(&votes, || {
            storage::set_vote_ledgers(&e, &vote_ledgers);

            let to_add_sequence = ledger + ONE_DAY_LEDGERS + 100;
            add_vote_ledger(&e, to_add_sequence);

            // perform expected change to vote_ledgers array
            vote_ledgers.push_back(to_add_sequence);

            let new_vote_ledgers = storage::get_vote_ledgers(&e);
            assert_eq!(new_vote_ledgers.len(), vote_ledgers.len());
            for i in 0..vote_ledgers.len() {
                assert_eq!(
                    new_vote_ledgers.get_unchecked(i),
                    vote_ledgers.get_unchecked(i)
                );
            }
        });
    }

    #[test]
    fn test_add_vote_ledger_out_of_order() {
        let e = Env::default();
        e.ledger().set(DEFAULT_LEDGER_INFO);

        let ledger = DEFAULT_LEDGER_INFO.sequence_number;

        let mut vote_ledgers = Vec::<u32>::new(&e);
        vote_ledgers.push_back(ledger - MAX_CHECKPOINT_AGE_LEDGERS - 1);
        vote_ledgers.push_back(ledger - ONE_DAY_LEDGERS / 2);
        vote_ledgers.push_back(ledger - 10);
        vote_ledgers.push_back(ledger + ONE_DAY_LEDGERS);

        let votes = e.register_contract(None, TokenVotes {});
        e.as_contract(&votes, || {
            storage::set_vote_ledgers(&e, &vote_ledgers);

            let to_add_sequence = ledger;
            add_vote_ledger(&e, to_add_sequence);

            // perform expected change to vote_ledgers array
            vote_ledgers.pop_front_unchecked();
            vote_ledgers.insert(2, to_add_sequence);

            let new_vote_ledgers = storage::get_vote_ledgers(&e);
            assert_eq!(new_vote_ledgers.len(), vote_ledgers.len());
            for i in 0..vote_ledgers.len() {
                assert_eq!(
                    new_vote_ledgers.get_unchecked(i),
                    vote_ledgers.get_unchecked(i)
                );
            }
        });
    }

    #[test]
    fn test_add_vote_ledger_from_empty() {
        let e = Env::default();
        e.ledger().set(DEFAULT_LEDGER_INFO);

        let ledger = DEFAULT_LEDGER_INFO.sequence_number;

        let votes = e.register_contract(None, TokenVotes {});
        e.as_contract(&votes, || {
            let vote_1 = ledger + ONE_DAY_LEDGERS;
            add_vote_ledger(&e, vote_1);
            let vote_2 = ledger + 100;
            add_vote_ledger(&e, vote_2);
            let vote_3 = ledger + 2 * ONE_DAY_LEDGERS;
            add_vote_ledger(&e, vote_3);
            let vote_4 = ledger + 101;
            add_vote_ledger(&e, vote_4);
            let vote_5 = ledger + ONE_DAY_LEDGERS;
            add_vote_ledger(&e, vote_5);

            let new_vote_ledgers = storage::get_vote_ledgers(&e);
            let vote_ledgers = vec![&e, vote_2, vote_4, vote_1, vote_3];
            assert_eq!(new_vote_ledgers.len(), vote_ledgers.len());
            for i in 0..vote_ledgers.len() {
                assert_eq!(
                    new_vote_ledgers.get_unchecked(i),
                    vote_ledgers.get_unchecked(i)
                );
            }
        });
    }
}
