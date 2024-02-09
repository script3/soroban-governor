use soroban_sdk::{Address, Env, String, Symbol};

use crate::types::Calldata;

pub struct GovernorEvents {}

impl GovernorEvents {
    /// Emitted when a proposal is created
    ///
    /// Note: The size limit for an event is 2kB. Title and calldata must be within the limit
    /// to create the proposal.
    ///
    /// - topics - `["proposal_created", proposal_id: u32, proposer: Address]`
    /// - data - `[title: String, calldata: Calldata]`
    pub fn proposal_created(
        e: &Env,
        proposal_id: u32,
        proposer: Address,
        title: String,
        calldata: Calldata,
    ) {
        let topics = (Symbol::new(&e, "proposal_created"), proposal_id, proposer);
        e.events().publish(topics, (title, calldata));
    }

    /// Emitted when a proposal is defeated
    ///
    /// - topics - `["proposal_closed", proposal_id: u32]`
    /// - data - `[new_status: ProposalStatus]`
    pub fn proposal_defeated(e: &Env, proposal_id: u32) {
        let topics = (Symbol::new(&e, "proposal_defeated"), proposal_id);
        e.events().publish(topics, ());
    }

    /// Emitted when a proposal is queued for submission
    ///
    /// - topics - `["proposal_queued", proposal_id: u32]`
    /// - data - `[unlock_timestamp: u64]`
    pub fn proposal_queued(e: &Env, proposal_id: u32, unlock_timestamp: u64) {
        let topics = (Symbol::new(&e, "proposal_queued"), proposal_id);
        e.events().publish(topics, unlock_timestamp);
    }

    /// Emitted when a proposal is canceled
    ///
    /// - topics - `["proposal_canceled", proposal_id: u32]`
    /// - data - `[]`
    pub fn proposal_canceled(e: &Env, proposal_id: u32) {
        let topics = (Symbol::new(&e, "proposal_canceled"), proposal_id);
        e.events().publish(topics, ());
    }

    /// Emitted when a proposal is executed
    ///
    /// - topics - `["proposal_executed", proposal_id: u32]`
    /// - data - `[]`
    pub fn proposal_executed(e: &Env, proposal_id: u32) {
        let topics = (Symbol::new(&e, "proposal_executed"), proposal_id);
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
