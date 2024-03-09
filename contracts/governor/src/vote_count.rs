use soroban_sdk::{panic_with_error, Env};

use crate::{constants::BPS_SCALAR, errors::GovernorError, types::VoteCount};

/// Implement VoteCount functions based on the counting type where and support
/// * support of 0 is against
/// * support of 1 is for
/// * support of 2 is abstain
/// * counting type is a number that flags what votes to count in quorum where {MSB}...{against}{for}{abstain}
impl VoteCount {
    /// Create a new VoteCount where all vote types have 0 votes
    pub fn new() -> Self {
        Self {
            against: 0,
            _for: 0,
            abstain: 0,
        }
    }

    /// Add a vote to the VoteCount
    ///
    /// ### Arguments
    /// * `e` - The environment
    /// * `support` - The vote to cast:
    ///                 - 0 to vote against
    ///                 - 1 to vote for
    ///                 - 2 to vote abstain
    /// * `amount` - The amount of votes to add
    ///
    /// ### Panics
    /// * If the support is not 0, 1, or 2
    pub fn add_vote(&mut self, e: &Env, support: u32, amount: i128) {
        match support {
            0 => self.against += amount,
            1 => self._for += amount,
            2 => self.abstain += amount,
            _ => panic_with_error!(e, GovernorError::InvalidProposalSupportError),
        }
    }

    /// Check if the vote has reached quorum
    ///
    /// ### Arguments
    /// * `quorum` - The quorum to reach (in bps)
    /// * `counting_type` - The type of votes to count in the quorum where {MSB}...{against}{for}{abstain}
    /// * `total_votes` - The total number of votes
    ///
    /// ### Returns
    /// * True if the vote has reached quorum
    /// * False if the vote has not reached quorum
    pub fn is_over_quorum(&self, quorum: u32, counting_type: u32, total_votes: i128) -> bool {
        let quorum_votes = self.count_quorum(counting_type);
        let quorum_requirement_floor = (total_votes * quorum as i128) / BPS_SCALAR;
        quorum_votes > quorum_requirement_floor
    }

    /// Check if the vote has passed the threshold
    ///
    /// ### Arguments
    /// * `vote_threshold` - The vote_threshold "for" must exceed "against" to pass (in bps)
    ///
    /// ### Returns
    /// * True if the vote has passed the threshold
    /// * False if the vote has not passed the threshold
    pub fn is_over_threshold(&self, vote_threshold: u32) -> bool {
        let against_and_for_votes = self.against + self._for;
        if against_and_for_votes == 0 {
            return false;
        }
        let for_votes = (self._for * BPS_SCALAR) / against_and_for_votes;
        for_votes > vote_threshold as i128
    }

    /// Count the number of votes included in the quorum
    ///
    /// ### Arguments
    /// * `counting_type` - The type of votes to count in the quorum where {MSB}...{against}{for}{abstain}
    fn count_quorum(&self, counting_type: u32) -> i128 {
        let mut quorum_votes = 0;
        if counting_type & 0b100 != 0 {
            quorum_votes += self.against;
        }
        if counting_type & 0b010 != 0 {
            quorum_votes += self._for;
        }
        if counting_type & 0b001 != 0 {
            quorum_votes += self.abstain;
        }
        quorum_votes
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_over_quorum() {
        let e = Env::default();
        let mut vote_count = VoteCount::new();
        vote_count.add_vote(&e, 0, 100); // Add 100 votes against (0b100)
        vote_count.add_vote(&e, 1, 101); // Add 200 votes for (0b010)
        vote_count.add_vote(&e, 2, 1); // Add 50 votes abstain (0b001)

        // quorum = 100 (10% of 1000)
        assert!(vote_count.is_over_quorum(1000, 0b111, 1000));
        assert!(vote_count.is_over_quorum(1000, 0b110, 1000));
        assert!(vote_count.is_over_quorum(1000, 0b101, 1000));
        assert!(!vote_count.is_over_quorum(1000, 0b100, 1000));
        assert!(vote_count.is_over_quorum(1000, 0b011, 1000));
        assert!(vote_count.is_over_quorum(1000, 0b010, 1000));
        assert!(!vote_count.is_over_quorum(1000, 0b001, 1000));
        assert!(!vote_count.is_over_quorum(1000, 0b000, 1000));
    }

    #[test]
    fn test_is_over_threshold() {
        let e = Env::default();
        let mut vote_count = VoteCount::new();
        vote_count.add_vote(&e, 0, 100); // Add 100 votes against
        vote_count.add_vote(&e, 1, 100); // Add 100 votes for
        vote_count.add_vote(&e, 2, 1000); // Add 50 votes abstain

        // rounds for / against down
        assert!(!vote_count.is_over_threshold(5000));
        assert!(vote_count.is_over_threshold(4999));

        vote_count.add_vote(&e, 1, 1);
        assert!(vote_count.is_over_threshold(5000));
    }
}

