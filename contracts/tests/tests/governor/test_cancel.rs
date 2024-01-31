#[cfg(test)]
use soroban_governor::{
    dependencies::{VotesClient, VOTES_WASM},
    storage::{self, Calldata, Proposal, ProposalStatus, SubCalldata},
};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, IntoVal, String, Symbol};
use tests::common::create_govenor;

#[test]
fn test_cancel() {
    let e = Env::default();
    let (govenor_address, votes_address, _, govenor) = create_govenor(&e);
    e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
    let votes_client = VotesClient::new(&e, &votes_address);
    let creater = Address::generate(&e);

    votes_client.set_votes(&creater, &10_000_000_i128);
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
                proposer: creater.clone(),
                calldata,
                sub_calldata,
                vote_start: 0,
                vote_end: 1000,
            },
        );
        storage::set_proposal_status(&e, &proposal_id, &ProposalStatus::Pending);
        return proposal_id;
    });

    e.mock_all_auths();
    govenor.cancel(&creater.clone(), &proposal_id);
    e.as_contract(&govenor_address, || {
        let proposal_id: u32 = storage::get_proposal_id(&e);
        let proposal_status = storage::get_proposal_status(&e, &proposal_id);
        assert_eq!(proposal_status, ProposalStatus::Expired);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn test_cancel_nonexistent_proposal() {
    let e = Env::default();
    let (govenor_address, votes_address, _, govenor) = create_govenor(&e);
    e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
    let creater = Address::generate(&e);

    let proposal_id = e.as_contract(&govenor_address, || {
        let proposal_id: u32 = storage::get_proposal_id(&e);
        return proposal_id;
    });

    e.mock_all_auths();
    govenor.cancel(&creater.clone(), &proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #207)")]
fn test_cancel_proposal_active() {
    let e = Env::default();
    let (govenor_address, votes_address, _, govenor) = create_govenor(&e);
    e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
    let votes_client = VotesClient::new(&e, &votes_address);
    let creater = Address::generate(&e);

    votes_client.set_votes(&creater, &10_000_000_i128);
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
                proposer: creater.clone(),
                calldata,
                sub_calldata,
                vote_start: 0,
                vote_end: 1000,
            },
        );
        storage::set_proposal_status(&e, &proposal_id, &ProposalStatus::Active);
        return proposal_id;
    });

    e.mock_all_auths();
    govenor.cancel(&creater.clone(), &proposal_id);
}
