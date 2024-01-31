#[cfg(test)]
use soroban_governor::{
    dependencies::{VotesClient, VOTES_WASM},
    storage::{self, Calldata, ProposalStatus, SubCalldata},
};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, IntoVal, String, Symbol};
use tests::common::create_govenor;
#[test]
fn test_propose() {
    let e = Env::default();
    let (govenor_address, votes_address, settings, govenor) = create_govenor(&e);
    e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
    let votes_client = VotesClient::new(&e, &votes_address);
    let creater = Address::generate(&e);

    votes_client.set_votes(&creater, &10_000_000_i128);
    let calldata = Calldata {
        contract_id: Address::generate(&e),
        function: Symbol::new(&e, "test"),
        args: (1, 2, 3).into_val(&e),
    };
    let sub_calldata = &vec![
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
#[should_panic(expected = "Error(Contract, #208)")]
fn test_propose_below_proposal_threshold() {
    let e = Env::default();
    let (_, votes_address, _, govenor) = create_govenor(&e);
    e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
    let votes_client = VotesClient::new(&e, &votes_address);
    let creater = Address::generate(&e);

    votes_client.set_votes(&creater, &0_i128);
    let calldata = Calldata {
        contract_id: Address::generate(&e),
        function: Symbol::new(&e, "test"),
        args: (1, 2, 3).into_val(&e),
    };
    let sub_calldata = &vec![
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
    govenor.propose(&creater, &calldata, sub_calldata, &title, &description);
}
