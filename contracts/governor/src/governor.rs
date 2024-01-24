use soroban_sdk::{contractclient, Address, Env, String, Vec};

use crate::storage::{CallData, GovernorSettings, SubCallData};

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
    /// * `calldata` - The calldata to execute when the proposal is executed
    /// * `sub_calldata` - The sub calldata to pre-authorize when the proposal is executed
    /// * `title` - The title of the proposal
    /// * `description` - The description of the proposal
    ///
    /// ### Panics
    /// If the proposal is not created successfully
    fn propose(
        e: Env,
        creator: Address,
        calldata: CallData,
        sub_calldata: Vec<SubCallData>,
        title: String,
        description: String,
    ) -> u32;

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
    /// * `creator` - The address of the account that created the proposal
    /// * `proposal_id` - The id of the proposal to cancel
    ///
    /// ### Panics
    /// * If the proposal_id is invalid
    /// * If the proposal is has already started voting
    /// * If the proposal is not created by the creator
    fn cancel(e: Env, creator: Address, proposal_id: u32);

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
}
