use soroban_governor::storage::{Proposal, VoteCount};
#[cfg(test)]
use soroban_governor::{
    dependencies::{VotesClient, VOTES_WASM},
    storage::{self, Calldata, ProposalStatus, SubCalldata},
};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{vec, Address, IntoVal, String};
use soroban_sdk::{Env, Symbol};
use tests::common::create_govenor;
#[test]
fn test_vote() {
    let e = Env::default();
    let (govenor_address, votes_address, _, govenor) = create_govenor(&e);
    e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
    let votes_client = VotesClient::new(&e, &votes_address);
    let creater = Address::generate(&e);
    let voter = Address::generate(&e);

    votes_client.set_votes(&creater, &1000_i128);
    votes_client.set_past_votes(&voter, &0, &1000_i128);
    let calldata = Calldata {
        contract_id: Address::generate(&e),
        function: Symbol::new(&e, "test"),
        args: (1, 2, 3).into_val(&e),
    };
    let sub_calldata = vec![
        &e,
        SubCalldata {
            contract_id: Address::generate(&e),
            function: Symbol::new(&e, "test"),
            args: (1, 2, 3).into_val(&e),
            sub_auth: vec![&e],
        },
    ];
    let title = String::from_str(&e, "Test Title");
    let description = String::from_str(&e, "Test Description");
    let proposal_id = e.as_contract(&govenor_address, || {
        let proposal_id: u32 = storage::get_proposal_id(&e);
        storage::set_proposal(
            &e,
            &proposal_id,
            &Proposal {
                id: proposal_id,
                title,
                description,
                proposer: creater,
                calldata,
                sub_calldata,
                vote_start: 0,
                vote_end: 1000,
            },
        );
        return proposal_id;
    });

    e.mock_all_auths();
    let voter_support = 1;
    govenor.vote(&voter, &proposal_id, &voter_support);
    e.as_contract(&govenor_address, || {
        // check voter status
        let voter_status = storage::get_voter_status(&e, &voter, &proposal_id).unwrap();
        assert_eq!(voter_status, voter_support);
        // check proposal votes
        let total_proposal_votes = storage::get_proposal_vote_count(&e, &proposal_id);
        assert_eq!(total_proposal_votes.votes_abstained, 0);
        assert_eq!(total_proposal_votes.votes_against, 1000);
        assert_eq!(total_proposal_votes.votes_for, 0);

        // check proposal status
        let proposal_status = storage::get_proposal_status(&e, &proposal_id);
        assert_eq!(proposal_status, ProposalStatus::Active);
    });
}

#[test]
fn test_vote_user_changes_support() {
    let e = Env::default();
    let (govenor_address, votes_address, _, govenor) = create_govenor(&e);
    e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
    let votes_client = VotesClient::new(&e, &votes_address);
    let creater = Address::generate(&e);
    let voter = Address::generate(&e);

    votes_client.set_votes(&creater, &1000_i128);
    votes_client.set_past_votes(&voter, &0, &1000_i128);
    let calldata = Calldata {
        contract_id: Address::generate(&e),
        function: Symbol::new(&e, "test"),
        args: (1, 2, 3).into_val(&e),
    };
    let sub_calldata = vec![
        &e,
        SubCalldata {
            contract_id: Address::generate(&e),
            function: Symbol::new(&e, "test"),
            args: (1, 2, 3).into_val(&e),
            sub_auth: vec![&e],
        },
    ];
    let title = String::from_str(&e, "Test Title");
    let description = String::from_str(&e, "Test Description");
    let proposal_id = e.as_contract(&govenor_address, || {
        let proposal_id: u32 = storage::get_proposal_id(&e);
        storage::set_proposal(
            &e,
            &proposal_id,
            &Proposal {
                id: proposal_id,
                title,
                description,
                proposer: creater,
                calldata,
                sub_calldata,
                vote_start: 0,
                vote_end: 1000,
            },
        );
        storage::set_voter_status(&e, &voter, &proposal_id, &1);
        storage::set_proposal_status(&e, &proposal_id, &ProposalStatus::Active);
        storage::set_proposal_vote_count(
            &e,
            &proposal_id,
            &VoteCount {
                votes_abstained: 0,
                votes_against: 1000,
                votes_for: 0,
            },
        );
        return proposal_id;
    });

    e.mock_all_auths();
    let voter_support = 2;
    govenor.vote(&voter, &proposal_id, &voter_support);
    e.as_contract(&govenor_address, || {
        // check voter status
        let voter_status = storage::get_voter_status(&e, &voter, &proposal_id).unwrap();
        assert_eq!(voter_status, voter_support);
        // check proposal votes
        let total_proposal_votes = storage::get_proposal_vote_count(&e, &proposal_id);
        assert_eq!(total_proposal_votes.votes_abstained, 0);
        assert_eq!(total_proposal_votes.votes_against, 0);
        assert_eq!(total_proposal_votes.votes_for, 1000);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_vote_nonexistent_proposal() {
    let e = Env::default();
    let (govenor_address, votes_address, _, govenor) = create_govenor(&e);
    e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
    let voter = Address::generate(&e);

    let proposal_id = e.as_contract(&govenor_address, || {
        let proposal_id: u32 = storage::get_proposal_id(&e);
        return proposal_id;
    });

    e.mock_all_auths();
    let voter_support = 1;
    govenor.vote(&voter, &proposal_id, &voter_support);
}

#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_vote_proposal_not_active() {
    let e = Env::default();
    let (govenor_address, votes_address, _, govenor) = create_govenor(&e);
    e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
    let creater = Address::generate(&e);
    let voter = Address::generate(&e);

    let calldata = Calldata {
        contract_id: Address::generate(&e),
        function: Symbol::new(&e, "test"),
        args: (1, 2, 3).into_val(&e),
    };
    let sub_calldata = vec![
        &e,
        SubCalldata {
            contract_id: Address::generate(&e),
            function: Symbol::new(&e, "test"),
            args: (1, 2, 3).into_val(&e),
            sub_auth: vec![&e],
        },
    ];
    let title = String::from_str(&e, "Test Title");
    let description = String::from_str(&e, "Test Description");
    let proposal_id = e.as_contract(&govenor_address, || {
        let proposal_id: u32 = storage::get_proposal_id(&e);
        storage::set_proposal(
            &e,
            &proposal_id,
            &Proposal {
                id: proposal_id,
                title,
                description,
                proposer: creater,
                calldata,
                sub_calldata,
                vote_start: 1,
                vote_end: 1000,
            },
        );
        return proposal_id;
    });

    e.mock_all_auths();
    let voter_support = 1;
    govenor.vote(&voter, &proposal_id, &voter_support);
}

#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn test_vote_invalid_support_option() {
    let e = Env::default();
    let (govenor_address, votes_address, _, govenor) = create_govenor(&e);
    e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
    let creater = Address::generate(&e);
    let voter = Address::generate(&e);

    let calldata = Calldata {
        contract_id: Address::generate(&e),
        function: Symbol::new(&e, "test"),
        args: (1, 2, 3).into_val(&e),
    };
    let sub_calldata = vec![
        &e,
        SubCalldata {
            contract_id: Address::generate(&e),
            function: Symbol::new(&e, "test"),
            args: (1, 2, 3).into_val(&e),
            sub_auth: vec![&e],
        },
    ];
    let title = String::from_str(&e, "Test Title");
    let description = String::from_str(&e, "Test Description");
    let proposal_id = e.as_contract(&govenor_address, || {
        let proposal_id: u32 = storage::get_proposal_id(&e);
        storage::set_proposal(
            &e,
            &proposal_id,
            &Proposal {
                id: proposal_id,
                title,
                description,
                proposer: creater,
                calldata,
                sub_calldata,
                vote_start: 0,
                vote_end: 1000,
            },
        );
        return proposal_id;
    });

    e.mock_all_auths();
    let voter_support = 3;
    govenor.vote(&voter, &proposal_id, &voter_support);
}
