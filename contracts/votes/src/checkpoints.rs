use soroban_sdk::Vec;

use crate::storage::VotingUnits;

/// Return the value in the most recent checkpoint that is less than or equal to the given timestamp.
///
/// ### Arguments
/// * checkpoints - The checkpoints to search
/// * timestamp - The maximum timestamp to search for
pub fn upper_lookup(checkpoints: &Vec<VotingUnits>, timestamp: u64) -> Option<VotingUnits> {
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
        Some(checkpoints.get_unchecked(high - 1))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use soroban_sdk::{Env, Vec};

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
        assert_eq!(result.unwrap().timestamp, 2000);

        let result = upper_lookup(&test_vec, 2000);
        assert_eq!(result.unwrap().timestamp, 2000);

        let result = upper_lookup(&test_vec, 7001);
        assert_eq!(result.unwrap().timestamp, 7000);

        let result = upper_lookup(&test_vec, 5999);
        assert_eq!(result.unwrap().timestamp, 5000);

        let result = upper_lookup(&test_vec, 4500);
        assert_eq!(result.unwrap().timestamp, 4000);

        let result = upper_lookup(&test_vec, 995);
        assert_eq!(result.unwrap().timestamp, 995);

        let result = upper_lookup(&test_vec, 994);
        assert!(result.is_none());
    }
}
