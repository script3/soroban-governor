use soroban_sdk::{Address, Env, String, Symbol};

use crate::types::ProposalAction;

pub struct GovernorEvents {}

impl GovernorEvents {
    /// Emitted when a proposal is created
    ///
    /// Note: The size limit for an event is 2kB. Title and calldata must be within the limit
    /// to create the proposal.
    ///
    /// - topics - `["proposal_created", proposal_id: u32, proposer: Address]`
    /// - data - `[title: String, desc: String, action: ProposalAction]`
    pub fn proposal_created(
        e: &Env,
        proposal_id: u32,
        proposer: Address,
        title: String,
        desc: String,
        action: ProposalAction,
    ) {
        let topics = (Symbol::new(&e, "proposal_created"), proposal_id, proposer);
        e.events().publish(topics, (title, desc, action));
    }

    /// Emitted when a proposal has changed status
    ///
    /// - topics - `["proposal_closed", proposal_id: u32, status: u32]`
    /// - data - `[]`
    pub fn proposal_updated(e: &Env, proposal_id: u32, status: u32) {
        let topics = (Symbol::new(&e, "proposal_updated"), proposal_id, status);
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
