#[cfg(test)]
use soroban_governor::storage::{self, Calldata, Proposal, ProposalStatus, SubCalldata};
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    vec, Address, Env, IntoVal, String, Symbol, TryIntoVal, Val,
};
use tests::{
    common::{create_govenor, create_stellar_token, create_token_votes, default_governor_settings},
    env::EnvTestUtils,
};

#[test]
fn test_vote() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);

    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let settings = default_governor_settings();
    let (govenor_address, governor_client) = create_govenor(&e, &votes_address, &settings);

    let total_votes: i128 = 10_000 * 10i128.pow(7);
    token_client.mint(&frodo, &total_votes);
    votes_client.deposit_for(&frodo, &total_votes);

    let samwise_votes = 8_000 * 10i128.pow(7);
    votes_client.transfer(&frodo, &samwise, &samwise_votes);

    let title = String::from_str(&e, "Test Title");
    let description = String::from_str(&e, "Test Description");
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

    // setup a proposal that can be voted on
    let proposal_id =
        governor_client.propose(&samwise, &calldata, &sub_calldata, &title, &description);
    e.jump_with_sequence(settings.vote_delay);

    let voter_support = 1;
    governor_client.vote(&samwise, &proposal_id, &voter_support);

    // validate auth
    assert_eq!(
        e.auths()[0],
        (
            samwise.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    govenor_address.clone(),
                    Symbol::new(&e, "vote"),
                    vec![
                        &e,
                        samwise.to_val(),
                        proposal_id.try_into_val(&e).unwrap(),
                        voter_support.try_into_val(&e).unwrap()
                    ]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    // validate chain results
    let votes = governor_client.get_vote(&samwise, &proposal_id);
    assert_eq!(votes, Some(voter_support));
    // TODO: Expose status of proposal

    // validate events
    let events = e.events().all();
    let tx_events = vec![&e, events.last().unwrap()];
    let event_data: soroban_sdk::Vec<Val> =
        vec![&e, voter_support.into_val(&e), samwise_votes.into_val(&e)];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                govenor_address.clone(),
                (Symbol::new(&e, "vote_cast"), proposal_id, samwise.clone()).into_val(&e),
                event_data.into_val(&e)
            )
        ]
    );
}

#[test]
fn test_vote_user_changes_support() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let settings = default_governor_settings();
    let (govenor_address, governor_client) = create_govenor(&e, &votes_address, &settings);

    let samwise_mint_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &samwise_mint_amount);
    token_client.mint(&samwise, &samwise_mint_amount);
    votes_client.deposit_for(&samwise, &samwise_mint_amount);
    //Update past checkpoints
    // votes_client.deposit_for(&samwise, &samwise_mint_amount);

    let title = String::from_str(&e, "Test Title");
    let description = String::from_str(&e, "Test Description");
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
                vote_start: e.ledger().timestamp() + 1,
                vote_end: e.ledger().timestamp() + 1 + settings.vote_period,
            },
        );
        storage::set_proposal_status(&e, &proposal_id, &ProposalStatus::Active);
        return proposal_id;
    });

    let voter_support = 1;
    governor_client.vote(&samwise, &proposal_id, &voter_support);
    e.as_contract(&govenor_address, || {
        // check voter status
        let voter_status = storage::get_voter_status(&e, &samwise, &proposal_id).unwrap();
        assert_eq!(voter_status, voter_support);
        // check proposal votes
        let votes = storage::get_proposal_vote_count(&e, &proposal_id);
        assert_eq!(votes.votes_abstained, 0);
        assert_eq!(votes.votes_against, samwise_mint_amount);
        assert_eq!(votes.votes_for, 0);
        // check proposal status
        let status = storage::get_proposal_status(&e, &proposal_id);
        assert_eq!(status, ProposalStatus::Active);
    });

    let voter_support = 2;
    governor_client.vote(&samwise, &proposal_id, &voter_support);
    e.as_contract(&govenor_address, || {
        // check voter status
        let voter_status = storage::get_voter_status(&e, &samwise, &proposal_id).unwrap();
        assert_eq!(voter_status, voter_support);
        // check proposal votes
        let votes = storage::get_proposal_vote_count(&e, &proposal_id);
        assert_eq!(votes.votes_abstained, 0);
        assert_eq!(votes.votes_against, 0);
        assert_eq!(votes.votes_for, samwise_mint_amount);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn test_vote_nonexistent_proposal() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let settings = default_governor_settings();
    let (govenor_address, governor_client) = create_govenor(&e, &votes_address, &settings);

    let samwise_mint_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &samwise_mint_amount);
    token_client.mint(&samwise, &samwise_mint_amount);
    votes_client.deposit_for(&samwise, &samwise_mint_amount);
    //Update past checkpoints
    votes_client.deposit_for(&samwise, &samwise_mint_amount);
    let proposal_id = e.as_contract(&govenor_address, || {
        let proposal_id: u32 = storage::get_proposal_id(&e);
        return proposal_id;
    });

    let voter_support = 1;
    governor_client.vote(&samwise, &proposal_id, &voter_support);
}

#[test]
#[should_panic(expected = "Error(Contract, #202)")]
fn test_vote_proposal_not_active() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let settings = default_governor_settings();
    let (govenor_address, governor_client) = create_govenor(&e, &votes_address, &settings);

    let samwise_mint_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &samwise_mint_amount);
    token_client.mint(&samwise, &samwise_mint_amount);
    votes_client.deposit_for(&samwise, &samwise_mint_amount);
    //Update past checkpoints
    votes_client.deposit_for(&samwise, &samwise_mint_amount);

    let title = String::from_str(&e, "Test Title");
    let description = String::from_str(&e, "Test Description");
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
                vote_start: e.ledger().timestamp() + 5,
                vote_end: e.ledger().timestamp() + 5 + settings.vote_period,
            },
        );
        return proposal_id;
    });

    e.mock_all_auths();
    let voter_support = 1;
    governor_client.vote(&samwise, &proposal_id, &voter_support);
}

#[test]
#[should_panic(expected = "Error(Contract, #203)")]
fn test_vote_invalid_support_option() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let settings = default_governor_settings();
    let (govenor_address, governor_client) = create_govenor(&e, &votes_address, &settings);

    let samwise_mint_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &samwise_mint_amount);
    token_client.mint(&samwise, &samwise_mint_amount);
    votes_client.deposit_for(&samwise, &samwise_mint_amount);
    //Update past checkpoints
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
                vote_start: e.ledger().timestamp(),
                vote_end: e.ledger().timestamp() + settings.vote_period,
            },
        );
        return proposal_id;
    });

    e.mock_all_auths();
    let voter_support = 3;
    governor_client.vote(&samwise, &proposal_id, &voter_support);
}
