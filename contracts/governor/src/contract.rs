use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env, String, Vec};

use crate::dependencies::VotesClient;
use crate::errors::GovernorError;
use crate::governor::Governor;
use crate::storage::{self, CallData, GovernorSettings, Proposal, SubCallData, MAX_VOTE_PERIOD};
#[contract]
pub struct GovernorContract;

#[contractimpl]
impl Governor for GovernorContract {
    fn initialize(e: Env, votes: Address, settings: GovernorSettings) {
        if storage::get_is_init(&e) {
            panic_with_error!(&e, GovernorError::AlreadyInitializedError);
        }
        if settings.vote_delay + settings.vote_period + settings.timelock > MAX_VOTE_PERIOD {
            panic_with_error!(&e, GovernorError::InternalError)
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
        calldata: CallData,
        sub_calldata: Vec<SubCallData>,
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

#[cfg(test)]
mod tests {
    use crate::dependencies::{VotesClient, VOTES_WASM};
    use crate::storage::{self, CallData, GovernorSettings, ProposalStatus, SubCallData};
    use crate::testutils::create_govenor;
    use crate::{GovernorContract, GovernorContractClient};
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{vec, Address, IntoVal, String};
    use soroban_sdk::{Env, Symbol};
    #[test]
    fn test_initialize_sets_storage() {
        let e = Env::default();
        let (govenor_address, votes_address, settings, _) = create_govenor(&e);
        e.as_contract(&govenor_address, || {
            let storage_settings: GovernorSettings = storage::get_settings(&e);

            assert!(storage::get_is_init(&e));
            assert_eq!(storage::get_voter_token_address(&e), votes_address);
            assert_eq!(storage_settings.counting_type, settings.counting_type);
            assert_eq!(
                storage_settings.proposal_threshold,
                settings.proposal_threshold
            );
            assert_eq!(storage_settings.quorum, settings.quorum);
            assert_eq!(storage_settings.timelock, settings.timelock);
            assert_eq!(storage_settings.vote_delay, settings.vote_delay);
            assert_eq!(storage_settings.vote_period, settings.vote_period);
            assert_eq!(storage_settings.vote_threshold, settings.vote_threshold);
        });
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #3)")]
    fn test_initalize_already_initalized() {
        let e = Env::default();
        let (_, votes_address, settings, govenor) = create_govenor(&e);
        govenor.initialize(&votes_address, &settings);
    }
    #[test]
    #[should_panic(expected = "Error(Contract, #1)")]
    fn test_initalize_proprosal_exceeds_time_length() {
        let e = Env::default();
        let address = e.register_contract(None, GovernorContract {});
        let govenor: GovernorContractClient<'_> = GovernorContractClient::new(&e, &address);
        let votes = Address::generate(&e);
        let settings = GovernorSettings {
            proposal_threshold: 1000,
            vote_delay: 500000,
            vote_period: 500000,
            timelock: 814401,
            quorum: 5000,
            counting_type: 6000,
            vote_threshold: 7000,
        };
        govenor.initialize(&votes, &settings);
    }
    #[test]
    fn test_propose() {
        let e = Env::default();
        let (govenor_address, votes_address, settings, govenor) = create_govenor(&e);
        e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
        let votes_client = VotesClient::new(&e, &votes_address);
        let creater = Address::generate(&e);

        votes_client.set_votes(&creater, &1000_i128);
        let calldata = CallData {
            contract_id: Address::generate(&e),
            function: Symbol::new(&e, "test"),
            args: (1, 2, 3).into_val(&e),
        };
        let sub_calldata = &vec![
            &e,
            SubCallData {
                contract_id: Address::generate(&e),
                function: Symbol::new(&e, "test"),
                args: (1, 2, 3).into_val(&e),
                sub_auth: vec![&e],
            },
        ];
        let title = String::from_str(&e, "Test Title");
        let description = String::from_str(&e, "Test Description");
        govenor.propose(&creater, &calldata, sub_calldata, &title, &description);

        e.as_contract(&govenor_address, || {
            let proposal = storage::get_proposal(&e, &0).unwrap();
            let next_proposal_id = storage::get_proposal_id(&e);
            let status = storage::get_proposal_status(&e, &0);

            assert_eq!(proposal.calldata.function, calldata.function);
            assert_eq!(proposal.calldata.contract_id, calldata.contract_id);
            assert_eq!(proposal.calldata.args, calldata.args);
            assert_eq!(
                proposal.sub_calldata.get(0).unwrap().contract_id,
                sub_calldata.get(0).unwrap().contract_id
            );
            assert_eq!(
                proposal.sub_calldata.get(0).unwrap().function,
                sub_calldata.get(0).unwrap().function
            );
            assert_eq!(
                proposal.sub_calldata.get(0).unwrap().args,
                sub_calldata.get(0).unwrap().args
            );
            assert_eq!(
                proposal.sub_calldata.get(0).unwrap().sub_auth.len(),
                sub_calldata.get(0).unwrap().sub_auth.len()
            );
            assert_eq!(proposal.id, 0);
            assert_eq!(proposal.proposer, creater);
            assert_eq!(proposal.title, title);
            assert_eq!(proposal.description, description);
            assert_eq!(proposal.vote_start, settings.vote_delay);
            assert_eq!(
                proposal.vote_end,
                settings.vote_delay + settings.vote_period
            );
            assert_eq!(next_proposal_id, 1);
            assert_eq!(status, ProposalStatus::Pending);
        });
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #10)")]
    fn test_propose_below_proposal_threshold() {
        let e = Env::default();
        let (_, votes_address, _, govenor) = create_govenor(&e);
        e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
        let votes_client = VotesClient::new(&e, &votes_address);
        let creater = Address::generate(&e);

        votes_client.set_votes(&creater, &0_i128);
        let calldata = CallData {
            contract_id: Address::generate(&e),
            function: Symbol::new(&e, "test"),
            args: (1, 2, 3).into_val(&e),
        };
        let sub_calldata = &vec![
            &e,
            SubCallData {
                contract_id: Address::generate(&e),
                function: Symbol::new(&e, "test"),
                args: (1, 2, 3).into_val(&e),
                sub_auth: vec![&e],
            },
        ];
        let title = String::from_str(&e, "Test Title");
        let description = String::from_str(&e, "Test Description");
        govenor.propose(&creater, &calldata, sub_calldata, &title, &description);
    }
}
