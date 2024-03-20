use soroban_sdk::{Address, Env, String, Symbol};

use crate::types::{ProposalAction, VoteCount};

pub struct GovernorEvents {}

impl GovernorEvents {
    /// Emitted when a proposal is created
    ///
    /// Note: The size limit for an event is 8kB. Title and calldata must be within the limit
    /// to create the proposal.
    ///
    /// - topics - `["proposal_created", proposal_id: u32, proposer: Address]`
    /// - data - `[title: String, desc: String, action: ProposalAction, vote_start: u32, vote_end: u32]`
    pub fn proposal_created(
        e: &Env,
        proposal_id: u32,
        proposer: Address,
        title: String,
        desc: String,
        action: ProposalAction,
        vote_start: u32,
        vote_end: u32,
    ) {
        let topics = (Symbol::new(&e, "proposal_created"), proposal_id, proposer);
        e.events()
            .publish(topics, (title, desc, action, vote_start, vote_end));
    }

    /// Emitted when a proposal is canceled
    ///
    /// - topics - `["proposal_canceled", proposal_id: u32]`
    /// - data - ()
    pub fn proposal_canceled(e: &Env, proposal_id: u32) {
        let topics = (Symbol::new(&e, "proposal_canceled"), proposal_id);
        e.events().publish(topics, ());
    }

    /// Emitted when a proposal is closed
    ///
    /// - topics - `["proposal_closed", proposal_id: u32, status: u32]`
    /// - data - `final_votes: VoteCount`
    pub fn proposal_voting_closed(e: &Env, proposal_id: u32, status: u32, final_votes: VoteCount) {
        let topics = (
            Symbol::new(&e, "proposal_voting_closed"),
            proposal_id,
            status,
        );
        e.events().publish(topics, final_votes);
    }

    /// Emitted when a proposal is executed
    ///
    /// - topics - `["proposal_executed", proposal_id: u32]`
    /// - data - Void
    pub fn proposal_executed(e: &Env, proposal_id: u32) {
        let topics = (Symbol::new(&e, "proposal_executed"), proposal_id);
        e.events().publish(topics, ());
    }

    /// Emitted when a proposal is expired
    ///
    /// - topics - `["proposal_expired", proposal_id: u32]`
    /// - data - Void
    pub fn proposal_expired(e: &Env, proposal_id: u32) {
        let topics = (Symbol::new(&e, "proposal_expired"), proposal_id);
        e.events().publish(topics, ());
    }

    /// Emitted when a vote is cast
    ///
    /// - topics - `["vote_cast", proposal_id: u32, voter: Address]`
    /// - data - `[support: u32, amount: i128]`
    pub fn vote_cast(e: &Env, proposal_id: u32, voter: Address, support: u32, amount: i128) {
        let topics = (Symbol::new(&e, "vote_cast"), proposal_id, voter);
        e.events().publish(topics, (support, amount));
    }
}
