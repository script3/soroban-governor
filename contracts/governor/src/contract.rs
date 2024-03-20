use soroban_sdk::{
    contract, contractimpl, panic_with_error, unwrap::UnwrapOptimized, Address, Env, String,
};

use crate::{
    dependencies::VotesClient,
    errors::GovernorError,
    events::GovernorEvents,
    governor::Governor,
    settings::require_valid_settings,
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
        require_valid_settings(&e, &settings);
        storage::set_settings(&e, &settings);
        storage::set_voter_token_address(&e, &votes);
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

        if storage::has_open_proposal(&e, &creator) {
            panic_with_error!(&e, GovernorError::ProposalAlreadyOpenError);
        }

        let settings = storage::get_settings(&e);
        let votes_client = VotesClient::new(&e, &storage::get_voter_token_address(&e));
        let creater_votes = votes_client.get_votes(&creator);
        if creater_votes < settings.proposal_threshold {
            panic_with_error!(&e, GovernorError::InsufficientVotingUnitsError)
        }

        let proposal_config =
            ProposalConfig::new(&e, title.clone(), description.clone(), action.clone());
        let proposal_id = storage::get_next_proposal_id(&e);
        let vote_start = match action {
            // no vote delay for snapshot proposals as they cannot be executed
            ProposalAction::Snapshot => e.ledger().sequence(),
            // all other proposals have a vote delay
            _ => e.ledger().sequence() + settings.vote_delay,
        };
        let vote_end = vote_start + settings.vote_period;
        let proposal_data = ProposalData {
            creator: creator.clone(),
            vote_start,
            vote_end,
            eta: 0,
            status: ProposalStatus::Open,
            executable: proposal_config.is_executable(),
        };
        storage::set_next_proposal_id(&e, proposal_id + 1);

        storage::create_proposal_config(&e, proposal_id, &proposal_config);
        storage::create_proposal_data(&e, proposal_id, &proposal_data);
        storage::create_proposal_vote_count(&e, proposal_id);
        storage::create_open_proposal(&e, &creator);

        votes_client.set_vote_sequence(&vote_start);

        GovernorEvents::proposal_created(
            &e,
            proposal_id,
            creator,
            title,
            description,
            action,
            vote_start,
            vote_end,
        );
        proposal_id
    }

    fn get_proposal(e: Env, proposal_id: u32) -> Option<Proposal> {
        let config = storage::get_proposal_config(&e, proposal_id);
        let data = storage::get_proposal_data(&e, proposal_id);
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
        let mut proposal_data = storage::get_proposal_data(&e, proposal_id)
            .unwrap_or_else(|| panic_with_error!(&e, GovernorError::NonExistentProposalError));

        if e.ledger().sequence() <= proposal_data.vote_end {
            panic_with_error!(&e, GovernorError::VotePeriodNotFinishedError)
        }

        let settings = storage::get_settings(&e);
        let votes_client = VotesClient::new(&e, &storage::get_voter_token_address(&e));
        let total_vote_supply = votes_client.get_past_total_supply(&proposal_data.vote_start);

        let vote_count = storage::get_proposal_vote_count(&e, proposal_id).unwrap_optimized();
        let passed_quorum =
            vote_count.is_over_quorum(settings.quorum, settings.counting_type, total_vote_supply);
        let passed_vote_threshold = vote_count.is_over_threshold(settings.vote_threshold);

        if passed_vote_threshold && passed_quorum {
            proposal_data.status = ProposalStatus::Successful;
            if proposal_data.executable {
                proposal_data.eta = e.ledger().sequence() + settings.timelock;
            }
        } else {
            proposal_data.status = ProposalStatus::Defeated;
        }
        storage::set_proposal_data(&e, proposal_id, &proposal_data);
        storage::del_open_proposal(&e, &proposal_data.creator);
        GovernorEvents::proposal_voting_closed(
            &e,
            proposal_id,
            proposal_data.status as u32,
            vote_count,
        );
    }

    fn execute(e: Env, proposal_id: u32) {
        storage::extend_instance(&e);
        let mut proposal_data = storage::get_proposal_data(&e, proposal_id)
            .unwrap_or_else(|| panic_with_error!(&e, GovernorError::NonExistentProposalError));

        if proposal_data.status != ProposalStatus::Successful
            || !proposal_data.executable
            || proposal_data.eta == 0
        {
            panic_with_error!(&e, GovernorError::ProposalNotExecutableError);
        }

        if e.ledger().sequence() < proposal_data.eta {
            panic_with_error!(&e, GovernorError::TimelockNotMetError);
        }

        let settings = storage::get_settings(&e);
        if e.ledger().sequence() > proposal_data.eta + settings.grace_period {
            proposal_data.status = ProposalStatus::Expired;
            GovernorEvents::proposal_expired(&e, proposal_id);
        } else {
            let proposal_config = storage::get_proposal_config(&e, proposal_id).unwrap_optimized();
            proposal_config.execute(&e);
            proposal_data.status = ProposalStatus::Executed;
            GovernorEvents::proposal_executed(&e, proposal_id);
        }
        storage::set_proposal_data(&e, proposal_id, &proposal_data);
    }

    fn cancel(e: Env, from: Address, proposal_id: u32) {
        storage::extend_instance(&e);
        from.require_auth();

        let mut proposal_data = storage::get_proposal_data(&e, proposal_id)
            .unwrap_or_else(|| panic_with_error!(&e, GovernorError::NonExistentProposalError));
        // require from to be the creator or the council
        if from != proposal_data.creator {
            let settings = storage::get_settings(&e);
            if from != settings.council {
                panic_with_error!(&e, GovernorError::UnauthorizedError);
            }
        }

        if proposal_data.vote_start <= e.ledger().sequence() {
            panic_with_error!(&e, GovernorError::ProposalVotePeriodStartedError);
        }
        proposal_data.status = ProposalStatus::Canceled;
        storage::set_proposal_data(&e, proposal_id, &proposal_data);
        storage::del_open_proposal(&e, &proposal_data.creator);
        GovernorEvents::proposal_canceled(&e, proposal_id);
    }

    fn vote(e: Env, voter: Address, proposal_id: u32, support: u32) {
        voter.require_auth();
        storage::extend_instance(&e);
        let proposal_data = storage::get_proposal_data(&e, proposal_id)
            .unwrap_or_else(|| panic_with_error!(&e, GovernorError::NonExistentProposalError));

        if proposal_data.status != ProposalStatus::Open {
            panic_with_error!(&e, GovernorError::ProposalClosedError);
        }
        if proposal_data.vote_start > e.ledger().sequence()
            || proposal_data.vote_end < e.ledger().sequence()
        {
            panic_with_error!(&e, GovernorError::OutsideOfVotePeriodError);
        }
        if storage::get_voter_support(&e, &voter, proposal_id).is_some() {
            panic_with_error!(&e, GovernorError::AlreadyVotedError);
        }

        let voter_power = VotesClient::new(&e, &storage::get_voter_token_address(&e))
            .get_past_votes(&voter, &proposal_data.vote_start);
        if voter_power <= 0 {
            panic_with_error!(&e, GovernorError::InsufficientVotingUnitsError);
        }

        let mut vote_count = storage::get_proposal_vote_count(&e, proposal_id).unwrap_optimized();
        vote_count.add_vote(&e, support, voter_power);

        storage::create_voter_support(&e, &voter, proposal_id, support);
        storage::set_proposal_vote_count(&e, proposal_id, &vote_count);

        GovernorEvents::vote_cast(&e, proposal_id, voter, support, voter_power);
    }

    fn get_vote(e: Env, voter: Address, proposal_id: u32) -> Option<u32> {
        storage::get_voter_support(&e, &voter, proposal_id)
    }

    fn get_proposal_votes(e: Env, proposal_id: u32) -> Option<VoteCount> {
        storage::get_proposal_vote_count(&e, proposal_id)
    }
}
