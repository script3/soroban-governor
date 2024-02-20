#[cfg(test)]
use sep_41_token::testutils::MockTokenClient;
use soroban_governor::types::{Calldata, ProposalStatus};
use soroban_governor::GovernorContractClient;
use soroban_sdk::{
    testutils::{Address as _, Events},
    vec, Address, Env, IntoVal, Symbol,
};
use soroban_votes::TokenVotesClient;
use tests::{
    env::EnvTestUtils,
    governor::{create_governor, default_governor_settings, default_proposal_data},
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

    let settings = default_governor_settings();
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = TokenVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let total_votes: i128 = 10_000 * 10i128.pow(7);
    token_client.mint(&frodo, &total_votes);
    votes_client.deposit_for(&frodo, &total_votes);

    let samwise_votes = 8_000 * 10i128.pow(7);
    votes_client.transfer(&frodo, &samwise, &samwise_votes);

    let pippin_votes = 1_000 * 10i128.pow(7);
    votes_client.transfer(&frodo, &pippin, &pippin_votes);

    let governor_transfer_amount: i128 = 10i128.pow(7);
    token_client.mint(&governor_address, &governor_transfer_amount);

    let (_, _, title, description) = default_proposal_data(&e);
    let calldata = Calldata {
        contract_id: token_address,
        function: Symbol::new(&e, "transfer"),
        args: (
            governor_address.clone(),
            samwise.clone(),
            governor_transfer_amount,
        )
            .into_val(&e),
    };

    // setup a proposal that is ready to be executed
    let proposal_id = governor_client.propose(&samwise, &calldata, &vec![&e], &title, &description);
    e.jump(settings.vote_delay + 1);
    governor_client.vote(&samwise, &proposal_id, &2);
    governor_client.vote(&pippin, &proposal_id, &1);
    e.jump(settings.vote_period);
    governor_client.close(&proposal_id);
    e.jump(settings.timelock);

    governor_client.execute(&proposal_id);

    // verify auths
    assert_eq!(e.auths().len(), 0);

    // verify chain results
    assert_eq!(token_client.balance(&samwise), governor_transfer_amount);
    assert_eq!(token_client.balance(&governor_address), 0);
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
                governor_address.clone(),
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
    let settings = default_governor_settings();
    let (governor_address, _, _) = create_governor(&e, &bombadil, &settings);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

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
    let settings = default_governor_settings();
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = TokenVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

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
            governor_address.clone(),
            samwise.clone(),
            governor_transfer_amount,
        )
            .into_val(&e),
    };

    let proposal_id = governor_client.propose(&samwise, &calldata, &vec![&e], &title, &description);
    e.jump(settings.vote_delay + 1);
    governor_client.vote(&samwise, &proposal_id, &2);
    governor_client.vote(&pippin, &proposal_id, &1);
    e.jump(settings.vote_period - 1);
    e.jump(settings.timelock);

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
    let settings = default_governor_settings();
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = TokenVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

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
            governor_address.clone(),
            samwise.clone(),
            governor_transfer_amount,
        )
            .into_val(&e),
    };

    let proposal_id = governor_client.propose(&samwise, &calldata, &vec![&e], &title, &description);
    e.jump(settings.vote_delay + 1);
    governor_client.vote(&samwise, &proposal_id, &2);
    governor_client.vote(&pippin, &proposal_id, &1);
    e.jump(settings.vote_period - 1);
    governor_client.close(&proposal_id);
    e.jump(settings.timelock - 1);

    governor_client.execute(&proposal_id);
}
