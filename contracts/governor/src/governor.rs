use soroban_sdk::{contractclient, Address, Env, String};

use crate::types::{GovernorSettings, Proposal, ProposalAction, VoteCount};

#[contractclient(name = "GovernorClient")]
pub trait Governor {
    /// Setup the governor contract
    ///
    /// ### Arguments
    /// * `votes` - The address of the contract used to track votes
    /// * `settings` - The settings for the governor
    fn initialize(e: Env, votes: Address, settings: GovernorSettings);

    /// Get the current settings of the governor
    fn settings(e: Env) -> GovernorSettings;

    /// Create a new proposal
    ///
    /// Returns the id of the new proposal
    ///
    /// ### Arguments
    /// * `creator` - The address of the account creating the proposal
    /// * `title` - The title of the proposal
    /// * `description` - The description of the proposal
    /// * `action` - The action the proposal will take if passed
    ///
    /// ### Panics
    /// If the proposal is not created successfully
    fn propose(
        e: Env,
        creator: Address,
        title: String,
        description: String,
        action: ProposalAction,
    ) -> u32;

    /// Get a proposal by its id
    ///
    /// Returns None if the proposal does not exist
    ///
    /// ### Arguments
    /// * `proposal_id` - The id of the proposal to get
    fn get_proposal(e: Env, proposal_id: u32) -> Option<Proposal>;

    /// Close the voting period for a proposal. Closing a proposal requires the quorum to be reached or the voting
    /// period to have ended. The proposal will be queued for execution if the quorum is reached and the vote passes.
    /// Otherwise, the proposal will be marked as failed.
    ///
    /// ### Arguments
    /// * `proposal_id` - The id of the proposal to close
    ///
    /// ### Panics
    /// * If the proposal_id is invalid
    /// * If the proposal is not ready to be closed
    fn close(e: Env, proposal_id: u32);

    /// Execute a proposal. Execution required the proposal has been queued for execution and the timelock has passed.
    ///
    /// ### Arguments
    /// * `proposal_id` - The id of the proposal to execute
    ///
    /// ### Panics
    /// * If the proposal_id is invalid
    /// * If the proposal is not ready to be executed
    fn execute(e: Env, proposal_id: u32);

    /// Cancel a proposal. Canceling a proposal requires the proposal to not have opened for voting yet.
    ///
    /// ### Arguments
    /// * `from` - The address of the account canceling the proposal
    /// * `proposal_id` - The id of the proposal to cancel
    ///
    /// ### Panics
    /// * If the `proposal_id` is invalid
    /// * If the proposal has already started voting
    /// * If from did not authorize the cancel or does not have the ability to cancel the proposal
    fn cancel(e: Env, from: Address, proposal_id: u32);

    /// Vote on a proposal with the voter's voting power at the time of the proposals voting checkpoint.
    ///
    /// ### Arguments
    /// * `voter` - The address of the account voting
    /// * `proposal_id` - The id of the proposal to vote on
    /// * `support` - The vote to cast:
    ///                 - 0 to vote abstain
    ///                 - 1 to vote against
    ///                 - 2 to vote for
    fn vote(e: Env, voter: Address, proposal_id: u32, support: u32);

    /// Get the voting status of a voter for a proposal.
    ///
    /// Returns None if the voter has not voted on the proposal, or a u32 that
    /// represents the vote cast (0 = abstain, 1 = against, 2 = for).
    ///
    /// ### Arguments
    /// * `voter` - The address of the account voting
    /// * `proposal_id` - The id of the proposal to vote on
    ///
    /// ### Panics
    /// * If the proposal_id is invalid
    fn get_vote(e: Env, voter: Address, proposal_id: u32) -> Option<u32>;

    /// Get the vote count for a proposal.
    ///
    /// Returns the vote count for the proposal, including the number of votes for, against, and abstained.
    /// If the proposal does not exist or has not been voted against, the vote count will be all zeros.
    ///
    /// ### Arguments
    /// * `proposal_id` - The id of the proposal to get the vote count for
    fn get_proposal_votes(e: Env, proposal_id: u32) -> VoteCount;
}
