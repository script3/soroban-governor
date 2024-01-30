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
        let proposal = storage::get_proposal(&e, &proposal_id);
        if proposal.is_none() {
            panic_with_error!(&e, GovernorError::NonExistentProposalError);
        }

        let proposal = proposal.unwrap_optimized();
        if e.ledger().timestamp() < proposal.vote_end {
            panic_with_error!(&e, GovernorError::VotePeriodNotFinishedError)
        }

        let settings = storage::get_settings(&e);
        let total_vote_supply =
            VotesClient::new(&e, &storage::get_voter_token_address(&e)).total_supply();
        let mut quorum_votes: i128 = 0;
        let proposal_results = storage::get_proposal_vote_count(&e, &proposal_id);

        if settings.counting_type & 0x1 == 1 {
            quorum_votes += proposal_results.votes_abstained;
        }
        if settings.counting_type >> 1 & 0x1 == 1 {
            quorum_votes += proposal_results.votes_against;
        }
        if settings.counting_type >> 2 & 0x1 == 1 {
            quorum_votes += proposal_results.votes_for;
        }
        let votes_for_percent = proposal_results.votes_for * 100
            / (proposal_results.votes_against + proposal_results.votes_for);

        if (quorum_votes * 100 / total_vote_supply) as u32 >= settings.quorum
            && votes_for_percent as u32 >= settings.vote_threshold
        {
            storage::set_proposal_status(&e, &proposal_id, &ProposalStatus::Queued);
        } else {
            storage::set_proposal_status(&e, &proposal_id, &ProposalStatus::Defeated);
        }
    }

    fn execute(e: Env, proposal_id: u32) {
        todo!()
    }

    fn cancel(e: Env, creator: Address, proposal_id: u32) {
        todo!()
    }

    fn vote(e: Env, voter: Address, proposal_id: u32, support: u32) {
        voter.require_auth();
        let proposal = storage::get_proposal(&e, &proposal_id);
        if proposal.is_none() {
            panic_with_error!(&e, GovernorError::NonExistentProposalError);
        }

        let proposal = proposal.unwrap_optimized();
        let proposal_status = storage::get_proposal_status(&e, &proposal_id);
        if proposal_status != ProposalStatus::Active {
            if proposal_status == ProposalStatus::Pending
                && proposal.vote_start <= e.ledger().timestamp()
            {
                storage::set_proposal_status(&e, &proposal_id, &ProposalStatus::Active)
            } else {
                panic_with_error!(&e, GovernorError::ProposalNotActiveError);
            }
        }

        let voter_power = VotesClient::new(&e, &storage::get_voter_token_address(&e))
            .get_past_votes(&voter, &proposal.vote_start);
        let mut vote_count = storage::get_proposal_vote_count(&e, &proposal_id);
        let voter_status = storage::get_voter_status(&e, &voter, &proposal_id);

        // Check if voter has already voted and remove previous vote from count
        if let Some(voter_status) = voter_status {
            match voter_status {
                0 => vote_count.votes_abstained -= voter_power,
                1 => vote_count.votes_against -= voter_power,
                2 => vote_count.votes_for -= voter_power,
                _ => (),
            }
        }

        match support {
            0 => vote_count.votes_abstained += voter_power,
            1 => vote_count.votes_against += voter_power,
            2 => vote_count.votes_for += voter_power,
            _ => panic_with_error!(&e, GovernorError::InvalidProposalSupportError),
        }

        storage::set_voter_status(&e, &voter, &proposal_id, &support);
        storage::set_proposal_vote_count(&e, &proposal_id, &vote_count);
    }

    fn get_vote(e: Env, voter: Address, proposal_id: u32) -> Option<u32> {
        todo!()
    }
}
