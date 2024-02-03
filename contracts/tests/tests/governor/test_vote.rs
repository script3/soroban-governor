#[cfg(test)]
use soroban_governor::storage::{self, Calldata, Proposal, ProposalStatus, SubCalldata};
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Env, IntoVal, String, Symbol, TryIntoVal,
};
use tests::{
    common::{create_govenor, create_stellar_token, create_token_votes},
    env::EnvTestUtils,
};
#[test]
fn test_vote() {
    let e = Env::default();
    e.set_default_info();
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
                proposer: bombadil,
                calldata,
                sub_calldata,
                vote_start: e.ledger().timestamp(),
                vote_end: e.ledger().timestamp() + settings.vote_period,
            },
        );
        return proposal_id;
    });

    let voter_support = 1;
    governor_client.vote(&samwise, &proposal_id, &voter_support);
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
        let proposal_status = storage::get_proposal_status(&e, &proposal_id);
        assert_eq!(proposal_status, ProposalStatus::Active);
    });
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
    let (govenor_address, governor_client, settings) = create_govenor(&e, &votes_address);

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
    let (govenor_address, governor_client, _) = create_govenor(&e, &votes_address);

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
    let (govenor_address, governor_client, settings) = create_govenor(&e, &votes_address);

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
    let (govenor_address, governor_client, settings) = create_govenor(&e, &votes_address);

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
