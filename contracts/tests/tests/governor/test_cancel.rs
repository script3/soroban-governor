#[cfg(test)]
use soroban_governor::storage::{self, Calldata, Proposal, ProposalStatus, SubCalldata};
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    vec, Address, Env, IntoVal, String, Symbol, TryIntoVal,
};
use tests::{
    common::{create_govenor, create_stellar_token, create_token_votes, default_governor_settings},
    env::EnvTestUtils,
};

#[test]
fn test_cancel() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let settings = default_governor_settings();
    let (govenor_address, governor_client) = create_govenor(&e, &votes_address, &settings);

    let samwise_mint_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &samwise_mint_amount);
    votes_client.deposit_for(&samwise, &samwise_mint_amount);

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

    // setup a proposal
    // -> Sam votes for
    // -> Pippin votes against
    let proposal_id =
        governor_client.propose(&samwise, &calldata, &sub_calldata, &title, &description);
    e.jump_with_sequence(settings.vote_delay / 2);

    governor_client.cancel(&samwise.clone(), &proposal_id);

    // verify auths
    assert_eq!(
        e.auths()[0],
        (
            samwise.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    govenor_address.clone(),
                    Symbol::new(&e, "cancel"),
                    vec![&e, samwise.to_val(), proposal_id.try_into_val(&e).unwrap()]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    // verify chain results
    // TODO: Expose proposal status

    // verify events
    let events = e.events().all();
    let tx_events = vec![&e, events.last().unwrap()];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                govenor_address.clone(),
                (Symbol::new(&e, "proposal_canceled"), proposal_id).into_val(&e),
                ().into_val(&e)
            )
        ]
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn test_cancel_nonexistent_proposal() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();
    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let settings = default_governor_settings();
    let (govenor_address, governor_client) = create_govenor(&e, &votes_address, &settings);

    let samwise_mint_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &samwise_mint_amount);
    votes_client.deposit_for(&samwise, &samwise_mint_amount);

    let proposal_id = e.as_contract(&govenor_address, || {
        let proposal_id: u32 = storage::get_proposal_id(&e);
        return proposal_id;
    });

    e.mock_all_auths();
    governor_client.cancel(&samwise.clone(), &proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #207)")]
fn test_cancel_proposal_active() {
    let e = Env::default();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let settings = default_governor_settings();
    let (govenor_address, governor_client) = create_govenor(&e, &votes_address, &settings);

    let samwise_mint_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &samwise_mint_amount);
    votes_client.deposit_for(&samwise, &samwise_mint_amount);

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
                proposer: samwise.clone(),
                calldata,
                sub_calldata,
                vote_start: 0,
                vote_end: 1000,
            },
        );
        storage::set_proposal_status(&e, &proposal_id, &ProposalStatus::Active);
        return proposal_id;
    });

    governor_client.cancel(&samwise.clone(), &proposal_id);
}
