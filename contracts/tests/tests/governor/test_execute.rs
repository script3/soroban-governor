#[cfg(test)]
use soroban_governor::storage::{self, Calldata, Proposal, ProposalStatus};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, IntoVal, String, Symbol};
use tests::{
    common::{create_govenor, create_stellar_token, create_token_votes},
    env::EnvTestUtils,
};

#[test]
fn test_execute() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let samwise = Address::generate(&e);
    // let transfer_recipient = Address::generate(&e);
    let bombadil = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, _) = create_token_votes(&e, &token_address);
    let (govenor_address, governor_client, settings) = create_govenor(&e, &votes_address);

    let governor_mint_amount: i128 = 1_000_000;
    token_client.mint(&govenor_address, &governor_mint_amount);

    let title = String::from_str(&e, "Test Title");
    let description = String::from_str(&e, "Test Description");
    let calldata = Calldata {
        contract_id: token_address,
        function: Symbol::new(&e, "transfer"),
        args: (
            govenor_address.clone(),
            samwise.clone(),
            governor_mint_amount,
        )
            .into_val(&e),
    };

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
                sub_calldata: vec![&e],
                vote_start: e.ledger().timestamp(),
                vote_end: e.ledger().timestamp() + settings.vote_period,
            },
        );

        storage::set_proposal_status(&e, &proposal_id, &ProposalStatus::Queued);
        return proposal_id;
    });
    e.jump(settings.vote_period + settings.timelock);

    governor_client.execute(&proposal_id);
    e.as_contract(&govenor_address, || {
        let proposal_status = storage::get_proposal_status(&e, &proposal_id);
        assert_eq!(proposal_status, ProposalStatus::Succeeded);
        assert_eq!(token_client.balance(&samwise), governor_mint_amount);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn test_execute_nonexistent_proposal() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let (token_address, _) = create_stellar_token(&e, &bombadil);
    let (votes_address, _) = create_token_votes(&e, &token_address);
    let (govenor_address, governor_client, _) = create_govenor(&e, &votes_address);

    let proposal_id = e.as_contract(&govenor_address, || {
        return storage::get_proposal_id(&e);
    });
    governor_client.execute(&proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #205)")]
fn test_execute_proposal_not_queued() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, _) = create_token_votes(&e, &token_address);
    let (govenor_address, governor_client, settings) = create_govenor(&e, &votes_address);

    let governor_mint_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &governor_mint_amount);

    let title = String::from_str(&e, "Test Title");
    let description = String::from_str(&e, "Test Description");
    let calldata = Calldata {
        contract_id: token_address,
        function: Symbol::new(&e, "transfer"),
        args: (
            govenor_address.clone(),
            samwise.clone(),
            governor_mint_amount,
        )
            .into_val(&e),
    };

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
                sub_calldata: vec![&e],
                vote_start: e.ledger().timestamp(),
                vote_end: e.ledger().timestamp() + settings.vote_period,
            },
        );
        storage::set_proposal_status(&e, &proposal_id, &ProposalStatus::Active);
        return proposal_id;
    });

    e.jump(settings.vote_period + settings.timelock);
    governor_client.execute(&proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #206)")]
fn test_execute_timelock_not_met() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, _) = create_token_votes(&e, &token_address);
    let (govenor_address, governor_client, settings) = create_govenor(&e, &votes_address);

    let governor_mint_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &governor_mint_amount);

    let title = String::from_str(&e, "Test Title");
    let description = String::from_str(&e, "Test Description");
    let calldata = Calldata {
        contract_id: token_address,
        function: Symbol::new(&e, "transfer"),
        args: (
            govenor_address.clone(),
            samwise.clone(),
            governor_mint_amount,
        )
            .into_val(&e),
    };

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
                sub_calldata: vec![&e],
                vote_start: e.ledger().timestamp(),
                vote_end: e.ledger().timestamp() + settings.vote_period,
            },
        );
        storage::set_proposal_status(&e, &proposal_id, &ProposalStatus::Queued);
        return proposal_id;
    });

    e.jump(settings.vote_period + settings.timelock - 1);
    governor_client.execute(&proposal_id);
}
