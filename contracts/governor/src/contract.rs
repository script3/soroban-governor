use soroban_sdk::{
    contract, contractimpl, panic_with_error, unwrap::UnwrapOptimized, Address, Env, String,
};

use crate::{
    constants::{MAX_PROPOSAL_LIFETIME, MAX_VOTE_PERIOD},
    dependencies::VotesClient,
    errors::GovernorError,
    events::GovernorEvents,
    governor::Governor,
    storage,
    types::{
        GovernorSettings, Proposal, ProposalAction, ProposalConfig, ProposalData, ProposalStatus,
        VoteCount,
    },
};

#[contract]
pub struct GovernorContract;

#[contractimpl]
impl Governor for GovernorContract {
    fn initialize(e: Env, votes: Address, settings: GovernorSettings) {
        if storage::get_is_init(&e) {
            panic_with_error!(&e, GovernorError::AlreadyInitializedError);
        }
        if settings.vote_period > MAX_VOTE_PERIOD {
            panic_with_error!(&e, GovernorError::InvalidSettingsError)
        }
        if settings.vote_delay + settings.vote_period + settings.timelock > MAX_PROPOSAL_LIFETIME {
            panic_with_error!(&e, GovernorError::InvalidSettingsError)
        }

        storage::set_voter_token_address(&e, &votes);
        storage::set_settings(&e, &settings);
        storage::set_is_init(&e);
        storage::extend_instance(&e);
    }

    fn settings(e: Env) -> GovernorSettings {
        storage::get_settings(&e)
    }

    fn propose(
        e: Env,
        creator: Address,
        title: String,
        description: String,
        action: ProposalAction,
    ) -> u32 {
        creator.require_auth();
        storage::extend_instance(&e);

        if storage::has_active_proposal(&e, &creator) {
            panic_with_error!(&e, GovernorError::ProposalAlreadyActiveError);
        }

        let settings = storage::get_settings(&e);
        let votes_client = VotesClient::new(&e, &storage::get_voter_token_address(&e));
        let creater_votes = votes_client.get_votes(&creator);
        if creater_votes < settings.proposal_threshold {
            panic_with_error!(&e, GovernorError::InsufficientVotingUnitsError)
        }

        let proposal_id = storage::get_next_proposal_id(&e);
        let vote_start = e.ledger().sequence() + settings.vote_delay;
        let vote_end = vote_start + settings.vote_period;
        let proposal_config = ProposalConfig {
            title: title.clone(),
            description: description.clone(),
            action: action.clone(),
        };
        let proposal_data = ProposalData {
            creator: creator.clone(),
            vote_start,
            vote_end,
            status: ProposalStatus::Pending,
            executable: proposal_config.is_executable(),
        };
        storage::set_next_proposal_id(&e, &(proposal_id + 1));
        storage::set_proposal_config(&e, &proposal_id, &proposal_config);
        storage::set_proposal_data(&e, &proposal_id, &proposal_data);
        storage::set_active_proposal(&e, &creator);

        votes_client.set_vote_sequence(&vote_start);

        GovernorEvents::proposal_created(&e, proposal_id, creator, title, description, action);
        proposal_id
    }

    fn get_proposal(e: Env, proposal_id: u32) -> Option<Proposal> {
        let config = storage::get_proposal_config(&e, &proposal_id);
        let data = storage::get_proposal_data(&e, &proposal_id);
        if config.is_none() || data.is_none() {
            None
        } else {
            Some(Proposal {
                id: proposal_id,
                config: config.unwrap_optimized(),
                data: data.unwrap_optimized(),
            })
        }
    }

    fn close(e: Env, proposal_id: u32) {
        storage::extend_instance(&e);
        let mut proposal_data = storage::get_proposal_data(&e, &proposal_id)
            .unwrap_or_else(|| panic_with_error!(&e, GovernorError::NonExistentProposalError));

        if e.ledger().sequence() < proposal_data.vote_end {
            panic_with_error!(&e, GovernorError::VotePeriodNotFinishedError)
        }

        let settings = storage::get_settings(&e);
        let votes_client = VotesClient::new(&e, &storage::get_voter_token_address(&e));
        let total_vote_supply = votes_client.get_past_total_supply(&proposal_data.vote_start);

        let vote_count = storage::get_proposal_vote_count(&e, &proposal_id);
        let passed_quorum =
            vote_count.is_over_quorum(settings.quorum, settings.counting_type, total_vote_supply);
        let passed_vote_threshold = vote_count.is_over_threshold(settings.vote_threshold);

        if passed_vote_threshold && passed_quorum {
            proposal_data.status = ProposalStatus::Successful;
        } else {
            proposal_data.status = ProposalStatus::Defeated;
        }
        storage::set_proposal_data(&e, &proposal_id, &proposal_data);
        storage::del_active_proposal(&e, &proposal_data.creator);
        GovernorEvents::proposal_updated(&e, proposal_id, proposal_data.status as u32);
    }

    fn execute(e: Env, proposal_id: u32) {
        storage::extend_instance(&e);
        let mut proposal_data = storage::get_proposal_data(&e, &proposal_id)
            .unwrap_or_else(|| panic_with_error!(&e, GovernorError::NonExistentProposalError));

        if proposal_data.status != ProposalStatus::Successful {
            panic_with_error!(&e, GovernorError::ProposalNotSuccessfulError);
        }

        let settings = storage::get_settings(&e);
        if e.ledger().sequence() < proposal_data.vote_end + settings.timelock {
            panic_with_error!(&e, GovernorError::TimelockNotMetError);
        }

        if e.ledger().sequence()
            > proposal_data.vote_end + settings.timelock + settings.grace_period
        {
            proposal_data.status = ProposalStatus::Expired;
        } else {
            let proposal_config = storage::get_proposal_config(&e, &proposal_id).unwrap_optimized();
            proposal_config.execute(&e);
            proposal_data.status = ProposalStatus::Executed;
        }
        GovernorEvents::proposal_updated(&e, proposal_id, proposal_data.status as u32);
        storage::set_proposal_data(&e, &proposal_id, &proposal_data);
    }

    fn cancel(e: Env, from: Address, proposal_id: u32) {
        storage::extend_instance(&e);
        from.require_auth();

        let mut proposal_data = storage::get_proposal_data(&e, &proposal_id)
            .unwrap_or_else(|| panic_with_error!(&e, GovernorError::NonExistentProposalError));
        if from != proposal_data.creator {
            let settings = storage::get_settings(&e);
            if from != settings.council {
                panic_with_error!(&e, GovernorError::UnauthorizedError);
            }
        }

        if proposal_data.status != ProposalStatus::Pending {
            panic_with_error!(&e, GovernorError::CancelActiveProposalError);
        }
        proposal_data.status = ProposalStatus::Canceled;
        storage::set_proposal_data(&e, &proposal_id, &proposal_data);
        GovernorEvents::proposal_updated(&e, proposal_id, proposal_data.status as u32);
    }

    fn vote(e: Env, voter: Address, proposal_id: u32, support: u32) {
        voter.require_auth();
        storage::extend_instance(&e);
        let mut proposal_data = storage::get_proposal_data(&e, &proposal_id)
            .unwrap_or_else(|| panic_with_error!(&e, GovernorError::NonExistentProposalError));
        if proposal_data.status != ProposalStatus::Active {
            if proposal_data.status == ProposalStatus::Pending
                && proposal_data.vote_start <= e.ledger().sequence()
            {
                proposal_data.status = ProposalStatus::Active;
                storage::set_proposal_data(&e, &proposal_id, &proposal_data);
            } else {
                panic_with_error!(&e, GovernorError::ProposalNotActiveError);
            }
        }

        if storage::get_voter_status(&e, &voter, &proposal_id).is_some() {
            panic_with_error!(&e, GovernorError::AlreadyVotedError);
        }

        let voter_power = VotesClient::new(&e, &storage::get_voter_token_address(&e))
            .get_past_votes(&voter, &proposal_data.vote_start);
        if voter_power <= 0 {
            panic_with_error!(&e, GovernorError::InsufficientVotingUnitsError);
        }

        let mut vote_count = storage::get_proposal_vote_count(&e, &proposal_id);
        vote_count.add_vote(&e, support, voter_power);

        storage::set_voter_status(&e, &voter, &proposal_id, &support);
        storage::set_proposal_vote_count(&e, &proposal_id, &vote_count);

        GovernorEvents::vote_cast(&e, proposal_id, voter, support, voter_power);
    }

    fn get_vote(e: Env, voter: Address, proposal_id: u32) -> Option<u32> {
        storage::get_voter_status(&e, &voter, &proposal_id)
    }

    fn get_proposal_votes(e: Env, proposal_id: u32) -> VoteCount {
        storage::get_proposal_vote_count(&e, &proposal_id)
    }
}
