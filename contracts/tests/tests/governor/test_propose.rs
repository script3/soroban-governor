#[cfg(test)]
use soroban_governor::storage::{self, Calldata, ProposalStatus, SubCalldata};
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Env, IntoVal, String, Symbol, TryIntoVal,
};
use tests::{
    common::{create_govenor, create_stellar_token, create_token_votes},
    env::EnvTestUtils,
};
#[test]
fn test_propose() {
    let e = Env::default();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let (govenor_address, governor_client, settings) = create_govenor(&e, &votes_address);

    let samwise_mint_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &samwise_mint_amount);
    votes_client.deposit_for(&samwise, &samwise_mint_amount);

    let title = String::from_str(&e, "Test Title");
    let description = String::from_str(&e, "Test Description");
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

    governor_client.propose(&samwise, &calldata, sub_calldata, &title, &description);
    assert_eq!(
        e.auths()[0],
        (
            samwise.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    govenor_address.clone(),
                    Symbol::new(&e, "propose"),
                    vec![
                        &e,
                        samwise.to_val(),
                        calldata.try_into_val(&e).unwrap(),
                        sub_calldata.to_val(),
                        title.to_val(),
                        description.to_val()
                    ]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
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
        assert_eq!(proposal.proposer, samwise);
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
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let (_, governor_client, _) = create_govenor(&e, &votes_address);

    let samwise_mint_amount: i128 = 999_999;
    token_client.mint(&samwise, &samwise_mint_amount);
    votes_client.deposit_for(&samwise, &samwise_mint_amount);

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
    governor_client.propose(&samwise, &calldata, sub_calldata, &title, &description);
}
