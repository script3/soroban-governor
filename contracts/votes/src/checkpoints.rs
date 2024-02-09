use soroban_sdk::{Env, Vec};

use crate::{constants::MAX_VOTE_CHECKPOINT_PERIOD, storage::VotingUnits};

/// Appends "to_add" to the checkpoints vector in place. This function also
/// manages any pruning of old checkpoints that may be necessary.
///
/// ### Arguments
/// * checkpoints - The checkpoints to add to
/// * to_add - The voting units to add
pub fn add_checkpoint(e: &Env, checkpoints: &mut Vec<VotingUnits>, to_add: &VotingUnits) {
    let mut len = checkpoints.len();
    if len == 0 {
        checkpoints.push_back(to_add.clone());
    } else {
        let last = checkpoints.get_unchecked(len - 1);
        if last.timestamp == to_add.timestamp {
            // last entry is no longer relevant
            checkpoints.pop_back();
            len -= 1;
        }
        // always add "to_add" to the end of the vector
        checkpoints.push_back(to_add.clone());
        len += 1;

        // prune checkpoints older than MAX_VOTE_CHECKPOINT_PERIOD
        // except the most recent checkpoint older than or equal to MAX_VOTE_CHECKPOINT_PERIOD
        if let Some(lower_bound_inclusive) = upper_lookup(
            checkpoints,
            e.ledger().timestamp() - MAX_VOTE_CHECKPOINT_PERIOD as u64,
        ) {
            *checkpoints = checkpoints.slice(lower_bound_inclusive..len);
        }
    }
}

/// Return the index for the most recent checkpoint that is less than or equal to the given timestamp.
///
/// ### Arguments
/// * checkpoints - The checkpoints to search
/// * timestamp - The maximum timestamp to search for
pub fn upper_lookup(checkpoints: &Vec<VotingUnits>, timestamp: u64) -> Option<u32> {
    let mut high = checkpoints.len();
    let mut low = 0;
    // Binary search for the highest checkpoint with a timestamp less than or equal to the given timestamp
    while low < high {
        let mid = (low + high) / 2;
        let entry = checkpoints.get_unchecked(mid);
        if entry.timestamp > timestamp {
            high = mid;
        } else {
            low = mid + 1;
        }
    }

    if high == 0 {
        None
    } else {
        Some(high - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Ledger, LedgerInfo},
        Env, Vec,
    };

    const ONE_DAY: u64 = 24 * 60 * 60;

    #[test]
    fn test_add_checkpoint_empty_vec() {
        let e = Env::default();
        let mut test_vec = Vec::<VotingUnits>::new(&e);
        let to_add = VotingUnits {
            amount: 123,
            timestamp: 1000,
        };

        add_checkpoint(&e, &mut test_vec, &to_add);

        assert_eq!(test_vec.len(), 1);
        let vote_last = test_vec.last_unchecked();
        assert_eq!(vote_last.amount, 123);
        assert_eq!(vote_last.timestamp, 1000);
    }

    #[test]
    fn test_add_checkpoint_no_pruning() {
        let e = Env::default();

        let start_time = 1441065600;
        let mut test_vec = Vec::<VotingUnits>::new(&e);
        test_vec.push_back(VotingUnits {
            amount: 0,
            timestamp: start_time,
        });
        test_vec.push_back(VotingUnits {
            amount: 123,
            timestamp: start_time + ONE_DAY,
        });
        test_vec.push_back(VotingUnits {
            amount: 50,
            timestamp: start_time + 2 * ONE_DAY,
        });
        test_vec.push_back(VotingUnits {
            amount: 5000,
            timestamp: start_time + 5 * ONE_DAY,
        });
        test_vec.push_back(VotingUnits {
            amount: 999,
            timestamp: start_time + 7 * ONE_DAY,
        });

        e.ledger().set(LedgerInfo {
            timestamp: start_time + 8 * ONE_DAY,
            protocol_version: 20,
            sequence_number: 100,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 10000,
            min_persistent_entry_ttl: 10000,
            max_entry_ttl: 10000,
        });
        let to_add = VotingUnits {
            amount: 42,
            timestamp: start_time + 8 * ONE_DAY - 100,
        };

        add_checkpoint(&e, &mut test_vec, &to_add);

        assert_eq!(test_vec.len(), 6);
        let vote_last = test_vec.last_unchecked();
        assert_eq!(vote_last.amount, 42);
        assert_eq!(vote_last.timestamp, start_time + 8 * ONE_DAY - 100);
        let vote_first = test_vec.first_unchecked();
        assert_eq!(vote_first.amount, 0);
        assert_eq!(vote_first.timestamp, start_time);
    }

    #[test]
    fn test_add_checkpoint_prune_partial_keeps_1_older() {
        let e = Env::default();

        let start_time = 1441065600;
        let mut test_vec = Vec::<VotingUnits>::new(&e);
        test_vec.push_back(VotingUnits {
            amount: 0,
            timestamp: start_time,
        });
        // @dev: this should be kept as its the most recent checkpoint older than
        //       MAX_VOTE_CHECKPOINT_PERIOD
        test_vec.push_back(VotingUnits {
            amount: 123,
            timestamp: start_time + 2 * ONE_DAY - 1,
        });
        test_vec.push_back(VotingUnits {
            amount: 50,
            timestamp: start_time + 2 * ONE_DAY + 1,
        });
        test_vec.push_back(VotingUnits {
            amount: 5000,
            timestamp: start_time + 5 * ONE_DAY,
        });
        test_vec.push_back(VotingUnits {
            amount: 999,
            timestamp: start_time + 7 * ONE_DAY,
        });

        e.ledger().set(LedgerInfo {
            timestamp: start_time + 10 * ONE_DAY,
            protocol_version: 20,
            sequence_number: 100,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 10000,
            min_persistent_entry_ttl: 10000,
            max_entry_ttl: 10000,
        });
        let to_add = VotingUnits {
            amount: 42,
            timestamp: start_time + 10 * ONE_DAY - 100,
        };

        add_checkpoint(&e, &mut test_vec, &to_add);

        assert_eq!(test_vec.len(), 5);
        let vote_last = test_vec.last_unchecked();
        assert_eq!(vote_last.amount, 42);
        assert_eq!(vote_last.timestamp, start_time + 10 * ONE_DAY - 100);
        let vote_first = test_vec.first_unchecked();
        assert_eq!(vote_first.amount, 123);
        assert_eq!(vote_first.timestamp, start_time + 2 * ONE_DAY - 1);
    }

    #[test]
    fn test_add_checkpoint_prune_all() {
        let e = Env::default();

        let start_time = 1441065600;
        let mut test_vec = Vec::<VotingUnits>::new(&e);
        test_vec.push_back(VotingUnits {
            amount: 0,
            timestamp: start_time,
        });
        test_vec.push_back(VotingUnits {
            amount: 123,
            timestamp: start_time + ONE_DAY - 1,
        });
        test_vec.push_back(VotingUnits {
            amount: 50,
            timestamp: start_time + ONE_DAY,
        });

        e.ledger().set(LedgerInfo {
            timestamp: start_time + 10 * ONE_DAY,
            protocol_version: 20,
            sequence_number: 100,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 10000,
            min_persistent_entry_ttl: 10000,
            max_entry_ttl: 10000,
        });
        let to_add = VotingUnits {
            amount: 42,
            timestamp: start_time + ONE_DAY + 10,
        };

        add_checkpoint(&e, &mut test_vec, &to_add);

        assert_eq!(test_vec.len(), 1);
        let vote_last = test_vec.last_unchecked();
        assert_eq!(vote_last.amount, 42);
        assert_eq!(vote_last.timestamp, start_time + ONE_DAY + 10);
    }

    #[test]
    fn test_upper_lookup() {
        let e = Env::default();
        let mut test_vec = Vec::<VotingUnits>::new(&e);
        test_vec.push_back(VotingUnits {
            amount: 0,
            timestamp: 995,
        });
        test_vec.push_back(VotingUnits {
            amount: 123,
            timestamp: 1000,
        });
        test_vec.push_back(VotingUnits {
            amount: 456,
            timestamp: 2000,
        });
        test_vec.push_back(VotingUnits {
            amount: 789,
            timestamp: 3000,
        });
        test_vec.push_back(VotingUnits {
            amount: 101112,
            timestamp: 4000,
        });
        test_vec.push_back(VotingUnits {
            amount: 131415,
            timestamp: 5000,
        });
        test_vec.push_back(VotingUnits {
            amount: 161718,
            timestamp: 6000,
        });
        test_vec.push_back(VotingUnits {
            amount: 143021,
            timestamp: 6100,
        });
        test_vec.push_back(VotingUnits {
            amount: 192021,
            timestamp: 7000,
        });

        let result = upper_lookup(&test_vec, 2500);
        assert_eq!(test_vec.get_unchecked(result.unwrap()).timestamp, 2000);

        let result = upper_lookup(&test_vec, 2000);
        assert_eq!(test_vec.get_unchecked(result.unwrap()).timestamp, 2000);

        let result = upper_lookup(&test_vec, 7001);
        assert_eq!(test_vec.get_unchecked(result.unwrap()).timestamp, 7000);

        let result = upper_lookup(&test_vec, 5999);
        assert_eq!(test_vec.get_unchecked(result.unwrap()).timestamp, 5000);

        let result = upper_lookup(&test_vec, 4500);
        assert_eq!(test_vec.get_unchecked(result.unwrap()).timestamp, 4000);

        let result = upper_lookup(&test_vec, 995);
        assert_eq!(test_vec.get_unchecked(result.unwrap()).timestamp, 995);

        let result = upper_lookup(&test_vec, 994);
        assert!(result.is_none());
    }

    // #[test]
    // fn test_lower_lookup() {
    //     let e = Env::default();
    //     let mut test_vec = Vec::<VotingUnits>::new(&e);
    //     test_vec.push_back(VotingUnits {
    //         amount: 0,
    //         timestamp: 995,
    //     });
    //     test_vec.push_back(VotingUnits {
    //         amount: 123,
    //         timestamp: 1000,
    //     });
    //     test_vec.push_back(VotingUnits {
    //         amount: 456,
    //         timestamp: 2000,
    //     });
    //     test_vec.push_back(VotingUnits {
    //         amount: 789,
    //         timestamp: 3000,
    //     });
    //     test_vec.push_back(VotingUnits {
    //         amount: 101112,
    //         timestamp: 4000,
    //     });
    //     test_vec.push_back(VotingUnits {
    //         amount: 131415,
    //         timestamp: 5000,
    //     });
    //     test_vec.push_back(VotingUnits {
    //         amount: 161718,
    //         timestamp: 6000,
    //     });
    //     test_vec.push_back(VotingUnits {
    //         amount: 143021,
    //         timestamp: 6100,
    //     });
    //     test_vec.push_back(VotingUnits {
    //         amount: 192021,
    //         timestamp: 7000,
    //     });

    //     let result = lower_lookup(&test_vec, 2500);
    //     assert_eq!(test_vec.get_unchecked(result.unwrap()).timestamp, 3000);

    //     let result = lower_lookup(&test_vec, 2000);
    //     assert_eq!(test_vec.get_unchecked(result.unwrap()).timestamp, 2000);

    //     let result = lower_lookup(&test_vec, 7001);
    //     assert!(result.is_none());

    //     let result = lower_lookup(&test_vec, 5999);
    //     assert_eq!(test_vec.get_unchecked(result.unwrap()).timestamp, 6000);

    //     let result = lower_lookup(&test_vec, 4001);
    //     assert_eq!(test_vec.get_unchecked(result.unwrap()).timestamp, 5000);

    //     let result = lower_lookup(&test_vec, 995);
    //     assert_eq!(test_vec.get_unchecked(result.unwrap()).timestamp, 995);

    //     let result = lower_lookup(&test_vec, 994);
    //     assert_eq!(test_vec.get_unchecked(result.unwrap()).timestamp, 995);
    // }
}
