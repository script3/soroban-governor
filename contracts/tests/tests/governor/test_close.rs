#[cfg(test)]
use soroban_governor::storage::{self, Calldata, Proposal, ProposalStatus, SubCalldata, VoteCount};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, IntoVal, String, Symbol};
use tests::{
    common::{create_govenor, create_stellar_token, create_token_votes},
    env::EnvTestUtils,
};

#[test]
fn test_close_proposal_queued() {
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
                proposer: samwise,
                calldata,
                sub_calldata,
                vote_start: e.ledger().timestamp(),
                vote_end: e.ledger().timestamp() + settings.vote_period,
            },
        );
        // Set proposal votes to ensure quorum is met
        storage::set_proposal_vote_count(
            &e,
            &proposal_id,
            &VoteCount {
                votes_abstained: 3_000_000,
                votes_against: 1_000_000,
                votes_for: 5_000_000,
            },
        );
        return proposal_id;
    });

    e.jump(settings.vote_period);
    governor_client.close(&proposal_id);
    e.as_contract(&govenor_address, || {
        let proposal_status = storage::get_proposal_status(&e, &proposal_id);
        assert_eq!(proposal_status, ProposalStatus::Queued);
    });
}
#[test]
fn test_close_quorum_not_met() {
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
                proposer: samwise,
                calldata,
                sub_calldata,
                vote_start: e.ledger().timestamp(),
                vote_end: e.ledger().timestamp() + settings.vote_period,
            },
        );
        // Set proposal votes to ensure quorum is met
        storage::set_proposal_vote_count(
            &e,
            &proposal_id,
            &VoteCount {
                votes_abstained: 2_000_000,
                votes_against: 1_000_000,
                votes_for: 3_000_000,
            },
        );
        return proposal_id;
    });

    e.jump(settings.vote_period);
    governor_client.close(&proposal_id);
    e.as_contract(&govenor_address, || {
        let proposal_status = storage::get_proposal_status(&e, &proposal_id);
        assert_eq!(proposal_status, ProposalStatus::Defeated);
    });
}

#[test]
fn test_close_quorum_vote_threshold_not_met() {
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
                proposer: samwise,
                calldata,
                sub_calldata,
                vote_start: e.ledger().timestamp(),
                vote_end: e.ledger().timestamp() + settings.vote_period,
            },
        );
        // Set proposal votes to ensure quorum is met
        storage::set_proposal_vote_count(
            &e,
            &proposal_id,
            &VoteCount {
                votes_abstained: 6_000_000,
                votes_against: 2_000_000,
                votes_for: 2_000_000,
            },
        );
        return proposal_id;
    });

    e.jump(settings.vote_period);
    governor_client.close(&proposal_id);
    e.as_contract(&govenor_address, || {
        let proposal_status = storage::get_proposal_status(&e, &proposal_id);
        assert_eq!(proposal_status, ProposalStatus::Defeated);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn test_close_nonexistent_proposal() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let (govenor_address, governor_client, _) = create_govenor(&e, &votes_address);

    let samwise_mint_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &samwise_mint_amount);
    votes_client.deposit_for(&samwise, &samwise_mint_amount);

    let proposal_id = e.as_contract(&govenor_address, || {
        return storage::get_proposal_id(&e);
    });
    governor_client.close(&proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_close_vote_period_unfinished() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let (govenor_address, governor_client, settings) = create_govenor(&e, &votes_address);

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
                proposer: samwise,
                calldata,
                sub_calldata,
                vote_start: e.ledger().timestamp(),
                vote_end: e.ledger().timestamp() + settings.vote_period,
            },
        );
        // Set proposal votes to ensure quorum is met
        storage::set_proposal_vote_count(
            &e,
            &proposal_id,
            &VoteCount {
                votes_abstained: 3_000_000,
                votes_against: 1_000_000,
                votes_for: 5_000_000,
            },
        );
        return proposal_id;
    });

    e.jump(settings.vote_period - 1);
    governor_client.close(&proposal_id);
}
