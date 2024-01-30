use soroban_sdk::{
    contract, contractimpl, panic_with_error, unwrap::UnwrapOptimized, Address, Env, String, Vec,
};

use crate::dependencies::VotesClient;
use crate::errors::GovernorError;
use crate::governor::Governor;
use crate::storage::{
    self, Calldata, GovernorSettings, Proposal, ProposalStatus, SubCalldata, MAX_VOTE_PERIOD,
};
#[contract]
pub struct GovernorContract;

#[contractimpl]
impl Governor for GovernorContract {
    fn initialize(e: Env, votes: Address, settings: GovernorSettings) {
        if storage::get_is_init(&e) {
            panic_with_error!(&e, GovernorError::AlreadyInitializedError);
        }
        if settings.vote_delay + settings.vote_period + settings.timelock > MAX_VOTE_PERIOD {
            panic_with_error!(&e, GovernorError::InvalidSettingsError)
        }

        storage::set_voter_token_address(&e, &votes);
        storage::set_settings(&e, &settings);
        storage::set_is_init(&e);
    }

    fn settings(e: Env) -> GovernorSettings {
        storage::get_settings(&e)
    }

    fn propose(
        e: Env,
        creator: Address,
        calldata: Calldata,
        sub_calldata: Vec<SubCalldata>,
        title: String,
        description: String,
    ) -> u32 {
        let settings = storage::get_settings(&e);
        let creater_votes =
            VotesClient::new(&e, &storage::get_voter_token_address(&e)).get_votes(&creator);
        if creater_votes < settings.proposal_threshold {
            panic_with_error!(&e, GovernorError::BalanceError)
        }

        let proposal_id = storage::get_proposal_id(&e);
        let vote_start = e.ledger().timestamp() + settings.vote_delay;
        let vote_end = vote_start + settings.vote_period;
        storage::set_proposal(
            &e,
            &proposal_id,
            &Proposal {
                id: proposal_id,
                title,
                calldata,
                sub_calldata,
                description,
                proposer: creator,
                vote_start,
                vote_end,
            },
        );
        storage::set_proposal_status(&e, &proposal_id, &storage::ProposalStatus::Pending);
        storage::set_proposal_id(&e, &(proposal_id + 1));
        proposal_id
    }

    fn close(e: Env, proposal_id: u32) {
        todo!()
    }

    fn execute(e: Env, proposal_id: u32) {
        todo!()
    }

    fn cancel(e: Env, creator: Address, proposal_id: u32) {
        todo!()
    }

    fn vote(e: Env, voter: Address, proposal_id: u32, support: u32) {
        todo!()
    }

    fn get_vote(e: Env, voter: Address, proposal_id: u32) -> Option<u32> {
        todo!()
    }
}
