#[cfg(test)]
use soroban_governor::types::{Calldata, ProposalStatus};
use soroban_sdk::{
    testutils::{Address as _, Events},
    vec, Address, Env, IntoVal, Symbol,
};
use tests::{
    common::{
        create_govenor, create_stellar_token, create_token_votes, default_governor_settings,
        default_proposal_data,
    },
    env::EnvTestUtils,
};

#[test]
fn test_execute() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);

    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let settings = default_governor_settings();
    let (govenor_address, governor_client) = create_govenor(&e, &votes_address, &settings);

    let total_votes: i128 = 10_000 * 10i128.pow(7);
    token_client.mint(&frodo, &total_votes);
    votes_client.deposit_for(&frodo, &total_votes);

    let samwise_votes = 8_000 * 10i128.pow(7);
    votes_client.transfer(&frodo, &samwise, &samwise_votes);

    let pippin_votes = 1_000 * 10i128.pow(7);
    votes_client.transfer(&frodo, &pippin, &pippin_votes);

    let governor_transfer_amount: i128 = 10i128.pow(7);
    token_client.mint(&govenor_address, &governor_transfer_amount);

    let (_, _, title, description) = default_proposal_data(&e);
    let calldata = Calldata {
        contract_id: token_address,
        function: Symbol::new(&e, "transfer"),
        args: (
            govenor_address.clone(),
            samwise.clone(),
            governor_transfer_amount,
        )
            .into_val(&e),
    };

    // setup a proposal that is ready to be executed
    let proposal_id = governor_client.propose(&samwise, &calldata, &vec![&e], &title, &description);
    e.jump_with_sequence(settings.vote_delay);
    governor_client.vote(&samwise, &proposal_id, &2);
    governor_client.vote(&pippin, &proposal_id, &1);
    e.jump_with_sequence(settings.vote_period);
    governor_client.close(&proposal_id);
    e.jump_with_sequence(settings.timelock);

    governor_client.execute(&proposal_id);

    // verify auths
    assert_eq!(e.auths().len(), 0);

    // verify chain results
    assert_eq!(token_client.balance(&samwise), governor_transfer_amount);
    assert_eq!(token_client.balance(&govenor_address), 0);
    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Executed);

    // verify events
    let events = e.events().all();
    let tx_events = vec![&e, events.last().unwrap()];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                govenor_address.clone(),
                (Symbol::new(&e, "proposal_executed"), proposal_id).into_val(&e),
                ().into_val(&e)
            )
        ]
    );
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
    let settings = default_governor_settings();
    let (_, governor_client) = create_govenor(&e, &votes_address, &settings);

    governor_client.execute(&0);
}

#[test]
#[should_panic(expected = "Error(Contract, #205)")]
fn test_execute_proposal_not_queued() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let settings = default_governor_settings();
    let (govenor_address, governor_client) = create_govenor(&e, &votes_address, &settings);

    let total_votes: i128 = 10_000 * 10i128.pow(7);
    token_client.mint(&frodo, &total_votes);
    votes_client.deposit_for(&frodo, &total_votes);

    let samwise_votes = 8_000 * 10i128.pow(7);
    votes_client.transfer(&frodo, &samwise, &samwise_votes);

    let pippin_votes = 1_000 * 10i128.pow(7);
    votes_client.transfer(&frodo, &pippin, &pippin_votes);

    let governor_transfer_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &governor_transfer_amount);

    let (_, _, title, description) = default_proposal_data(&e);
    let calldata = Calldata {
        contract_id: token_address,
        function: Symbol::new(&e, "transfer"),
        args: (
            govenor_address.clone(),
            samwise.clone(),
            governor_transfer_amount,
        )
            .into_val(&e),
    };

    let proposal_id = governor_client.propose(&samwise, &calldata, &vec![&e], &title, &description);
    e.jump_with_sequence(settings.vote_delay);
    governor_client.vote(&samwise, &proposal_id, &2);
    governor_client.vote(&pippin, &proposal_id, &1);
    e.jump_with_sequence(settings.vote_period);
    e.jump_with_sequence(settings.timelock);

    governor_client.execute(&proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #206)")]
fn test_execute_timelock_not_met() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let settings = default_governor_settings();
    let (govenor_address, governor_client) = create_govenor(&e, &votes_address, &settings);

    let total_votes: i128 = 10_000 * 10i128.pow(7);
    token_client.mint(&frodo, &total_votes);
    votes_client.deposit_for(&frodo, &total_votes);

    let samwise_votes = 8_000 * 10i128.pow(7);
    votes_client.transfer(&frodo, &samwise, &samwise_votes);

    let pippin_votes = 1_000 * 10i128.pow(7);
    votes_client.transfer(&frodo, &pippin, &pippin_votes);

    let governor_transfer_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &governor_transfer_amount);

    let (_, _, title, description) = default_proposal_data(&e);
    let calldata = Calldata {
        contract_id: token_address,
        function: Symbol::new(&e, "transfer"),
        args: (
            govenor_address.clone(),
            samwise.clone(),
            governor_transfer_amount,
        )
            .into_val(&e),
    };

    let proposal_id = governor_client.propose(&samwise, &calldata, &vec![&e], &title, &description);
    e.jump_with_sequence(settings.vote_delay);
    governor_client.vote(&samwise, &proposal_id, &2);
    governor_client.vote(&pippin, &proposal_id, &1);
    e.jump_with_sequence(settings.vote_period);
    governor_client.close(&proposal_id);
    e.jump_with_sequence(settings.timelock - 1);

    governor_client.execute(&proposal_id);
}
