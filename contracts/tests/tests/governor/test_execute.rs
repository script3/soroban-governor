#[cfg(test)]
use sep_41_token::testutils::MockTokenClient;
use soroban_governor::types::{Calldata, GovernorSettings, ProposalAction, ProposalStatus};
use soroban_governor::GovernorContractClient;
use soroban_sdk::testutils::{Ledger, LedgerInfo};
use soroban_sdk::{
    testutils::{Address as _, Events},
    vec, Address, Env, IntoVal, String, Symbol,
};
use soroban_votes::TokenVotesClient;
use tests::mocks::create_mock_subcall_contract_wasm;
use tests::ONE_DAY_LEDGERS;
use tests::{
    env::EnvTestUtils,
    governor::{create_governor, default_governor_settings, default_proposal_data},
};

#[test]
fn test_execute_calldata_no_auths() {
    let e = Env::default();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);

    let settings = default_governor_settings(&e);
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = TokenVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_votes = 8_000 * 10i128.pow(7);
    let pippin_votes = 1_000 * 10i128.pow(7);
    let total_votes: i128 = 10_000 * 10i128.pow(7);
    let frodo_votes = total_votes - samwise_votes - pippin_votes;
    token_client.mock_all_auths().mint(&frodo, &frodo_votes);
    votes_client.mock_all_auths().deposit(&frodo, &frodo_votes);
    token_client.mock_all_auths().mint(&samwise, &samwise_votes);
    votes_client
        .mock_all_auths()
        .deposit(&samwise, &samwise_votes);
    token_client.mock_all_auths().mint(&pippin, &pippin_votes);
    votes_client
        .mock_all_auths()
        .deposit(&pippin, &pippin_votes);

    let governor_transfer_amount: i128 = 10i128.pow(7);
    token_client
        .mock_all_auths()
        .mint(&governor_address, &governor_transfer_amount);
    let (title, description, _) = default_proposal_data(&e);
    let action = ProposalAction::Calldata(Calldata {
        contract_id: token_address,
        function: Symbol::new(&e, "transfer"),
        args: (
            governor_address.clone(),
            samwise.clone(),
            governor_transfer_amount,
        )
            .into_val(&e),
        auths: vec![&e],
    });

    // setup a proposal that is ready to be executed
    let proposal_id =
        governor_client
            .mock_all_auths()
            .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    governor_client
        .mock_all_auths()
        .vote(&samwise, &proposal_id, &1);
    governor_client
        .mock_all_auths()
        .vote(&pippin, &proposal_id, &0);
    e.jump(settings.vote_period);
    governor_client.mock_all_auths().close(&proposal_id);
    e.jump(settings.timelock);

    // remove any potential auth mocking
    e.set_auths(&[]);
    governor_client.set_auths(&[]);
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
                (Symbol::new(&e, "proposal_executed"), proposal_id,).into_val(&e),
                ().into_val(&e)
            )
        ]
    );
}

#[test]
fn test_execute_calldata_auth_chain() {
    let e = Env::default();
    e.set_default_info();
    e.budget().reset_unlimited();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);

    let settings = default_governor_settings(&e);
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = TokenVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);
    let (outter_subcall_address, _) =
        create_mock_subcall_contract_wasm(&e, &token_address, &governor_address);
    let (inner_subcall_address, _) =
        create_mock_subcall_contract_wasm(&e, &token_address, &governor_address);

    // set intial votes
    let frodo_votes: i128 = 10_000 * 10i128.pow(7);
    token_client.mock_all_auths().mint(&frodo, &frodo_votes);
    votes_client.mock_all_auths().deposit(&frodo, &frodo_votes);

    // create a proposal
    let (title, description, _) = default_proposal_data(&e);
    let call_amount: i128 = 100 * 10i128.pow(7);
    token_client
        .mock_all_auths()
        .mint(&governor_address, &call_amount);
    let action = ProposalAction::Calldata(Calldata {
        contract_id: outter_subcall_address.clone(),
        function: Symbol::new(&e, "call_subcall"),
        args: (inner_subcall_address.clone(), call_amount.clone(), true).into_val(&e),
        auths: vec![
            &e,
            Calldata {
                contract_id: inner_subcall_address.clone(),
                function: Symbol::new(&e, "subcall"),
                args: (call_amount.clone(),).into_val(&e),
                auths: vec![
                    &e,
                    Calldata {
                        contract_id: token_address,
                        function: Symbol::new(&e, "transfer"),
                        args: (
                            governor_address.clone(),
                            inner_subcall_address.clone(),
                            call_amount.clone(),
                        )
                            .into_val(&e),
                        auths: vec![&e],
                    },
                ],
            },
        ],
    });

    let proposal_id =
        governor_client
            .mock_all_auths()
            .propose(&frodo, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    governor_client
        .mock_all_auths()
        .vote(&frodo, &proposal_id, &1);
    e.jump(settings.vote_period);
    governor_client.mock_all_auths().close(&proposal_id);
    e.jump(settings.timelock);

    // remove any potential auth mocking
    governor_client.set_auths(&[]);
    governor_client.execute(&proposal_id);

    assert_eq!(token_client.balance(&inner_subcall_address), call_amount);
    assert_eq!(token_client.balance(&governor_address), 0);
    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Executed);
}

#[test]
fn test_execute_calldata_single_auth() {
    let e = Env::default();
    e.set_default_info();
    e.budget().reset_unlimited();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);

    let settings = default_governor_settings(&e);
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = TokenVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);
    let (outter_subcall_address, _) =
        create_mock_subcall_contract_wasm(&e, &token_address, &governor_address);
    let (inner_subcall_address, _) =
        create_mock_subcall_contract_wasm(&e, &token_address, &governor_address);

    // set intial votes
    let frodo_votes: i128 = 10_000 * 10i128.pow(7);
    token_client.mock_all_auths().mint(&frodo, &frodo_votes);
    votes_client.mock_all_auths().deposit(&frodo, &frodo_votes);

    // create a proposal
    let (title, description, _) = default_proposal_data(&e);
    let call_amount: i128 = 100 * 10i128.pow(7);
    token_client
        .mock_all_auths()
        .mint(&governor_address, &call_amount);
    let action = ProposalAction::Calldata(Calldata {
        contract_id: outter_subcall_address.clone(),
        function: Symbol::new(&e, "call_subcall"),
        args: (inner_subcall_address.clone(), call_amount.clone(), false).into_val(&e),
        auths: vec![
            &e,
            Calldata {
                contract_id: token_address,
                function: Symbol::new(&e, "transfer"),
                args: (
                    governor_address.clone(),
                    inner_subcall_address.clone(),
                    call_amount.clone(),
                )
                    .into_val(&e),
                auths: vec![&e],
            },
        ],
    });

    let proposal_id =
        governor_client
            .mock_all_auths()
            .propose(&frodo, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    governor_client
        .mock_all_auths()
        .vote(&frodo, &proposal_id, &1);
    e.jump(settings.vote_period);
    governor_client.mock_all_auths().close(&proposal_id);
    e.jump(settings.timelock);

    // remove any potential auth mocking
    e.set_auths(&[]);
    governor_client.set_auths(&[]);
    governor_client.execute(&proposal_id);

    assert_eq!(token_client.balance(&inner_subcall_address), call_amount);
    assert_eq!(token_client.balance(&governor_address), 0);
    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Executed);
}

#[test]
fn test_execute_settings() {
    let e = Env::default();
    e.set_default_info();
    e.budget().reset_unlimited();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);

    let settings = default_governor_settings(&e);
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = TokenVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    // set intial votes
    let frodo_votes: i128 = 10_000 * 10i128.pow(7);
    token_client.mock_all_auths().mint(&frodo, &frodo_votes);
    votes_client.mock_all_auths().deposit(&frodo, &frodo_votes);

    // create a proposal
    let new_settings = GovernorSettings {
        council: Address::generate(&e),
        proposal_threshold: 829421,
        vote_delay: 1231,
        vote_period: 7456,
        timelock: 15678,
        grace_period: 35678,
        quorum: 300,
        counting_type: 1,
        vote_threshold: 2000,
    };
    let (title, description, _) = default_proposal_data(&e);
    let call_amount: i128 = 100 * 10i128.pow(7);
    token_client
        .mock_all_auths()
        .mint(&governor_address, &call_amount);
    let action = ProposalAction::Settings(new_settings.clone());

    let proposal_id =
        governor_client
            .mock_all_auths()
            .propose(&frodo, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    governor_client
        .mock_all_auths()
        .vote(&frodo, &proposal_id, &1);
    e.jump(settings.vote_period);
    governor_client.mock_all_auths().close(&proposal_id);
    e.jump(settings.timelock);

    // remove any potential auth mocking
    e.set_auths(&[]);
    governor_client.set_auths(&[]);
    governor_client.execute(&proposal_id);

    let gov_settings = governor_client.settings();
    assert_eq!(
        gov_settings.proposal_threshold,
        new_settings.proposal_threshold
    );
    assert_eq!(gov_settings.vote_delay, new_settings.vote_delay);
    assert_eq!(gov_settings.vote_period, new_settings.vote_period);
    assert_eq!(gov_settings.timelock, new_settings.timelock);
    assert_eq!(gov_settings.grace_period, new_settings.grace_period);
    assert_eq!(gov_settings.quorum, new_settings.quorum);
    assert_eq!(gov_settings.counting_type, new_settings.counting_type);
    assert_eq!(gov_settings.vote_threshold, new_settings.vote_threshold);
}

#[test]
fn test_execute_upgrade() {
    let e = Env::default();
    e.set_default_info();
    e.budget().reset_unlimited();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);

    let mut settings = default_governor_settings(&e);
    settings.council = frodo.clone();
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = TokenVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    // set intial votes
    let frodo_votes: i128 = 10_000 * 10i128.pow(7);
    token_client.mock_all_auths().mint(&frodo, &frodo_votes);
    votes_client.mock_all_auths().deposit(&frodo, &frodo_votes);

    // create a proposal
    let new_wasm = e
        .deployer()
        .upload_contract_wasm(sep_41_token::testutils::MockTokenWASM);
    let (title, description, _) = default_proposal_data(&e);
    let action = ProposalAction::Upgrade(new_wasm);

    let proposal_id =
        governor_client
            .mock_all_auths()
            .propose(&frodo, &title, &description, &action);

    e.jump(settings.vote_delay + 1);
    governor_client
        .mock_all_auths()
        .vote(&frodo, &proposal_id, &1);
    e.jump(settings.vote_period);
    governor_client.mock_all_auths().close(&proposal_id);
    e.jump(settings.timelock);

    // remove any potential auth mocking
    e.set_auths(&[]);
    governor_client.set_auths(&[]);
    governor_client.execute(&proposal_id);

    let new_client = MockTokenClient::new(&e, &governor_address);
    new_client.mock_all_auths().initialize(
        &bombadil,
        &7,
        &String::from_str(&e, "tea"),
        &String::from_str(&e, "pot"),
    );
    new_client.mock_all_auths().mint(&frodo, &123);
    assert_eq!(new_client.balance(&frodo), 123);
}

#[test]
fn test_execute_expired() {
    let e = Env::default();
    e.ledger().set(LedgerInfo {
        timestamp: 1441065600, // Sept 1st, 2015 12:00:00 AM UTC
        protocol_version: 20,
        sequence_number: 100,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 20 * ONE_DAY_LEDGERS,
        min_persistent_entry_ttl: 20 * ONE_DAY_LEDGERS,
        max_entry_ttl: 365 * ONE_DAY_LEDGERS,
    });

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);

    let settings = default_governor_settings(&e);
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = TokenVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_votes = 8_000 * 10i128.pow(7);
    let pippin_votes = 1_000 * 10i128.pow(7);
    let total_votes: i128 = 10_000 * 10i128.pow(7);
    let frodo_votes = total_votes - samwise_votes - pippin_votes;
    token_client.mock_all_auths().mint(&frodo, &frodo_votes);
    votes_client.mock_all_auths().deposit(&frodo, &frodo_votes);
    token_client.mock_all_auths().mint(&samwise, &samwise_votes);
    votes_client
        .mock_all_auths()
        .deposit(&samwise, &samwise_votes);
    token_client.mock_all_auths().mint(&pippin, &pippin_votes);
    votes_client
        .mock_all_auths()
        .deposit(&pippin, &pippin_votes);

    let governor_transfer_amount: i128 = 10i128.pow(7);
    token_client
        .mock_all_auths()
        .mint(&governor_address, &governor_transfer_amount);
    let (title, description, _) = default_proposal_data(&e);
    let action = ProposalAction::Calldata(Calldata {
        contract_id: token_address,
        function: Symbol::new(&e, "transfer"),
        args: (
            governor_address.clone(),
            samwise.clone(),
            governor_transfer_amount,
        )
            .into_val(&e),
        auths: vec![&e],
    });

    // setup a proposal that is ready to be executed - then wait past the grace period
    let proposal_id =
        governor_client
            .mock_all_auths()
            .propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    governor_client
        .mock_all_auths()
        .vote(&samwise, &proposal_id, &1);
    governor_client
        .mock_all_auths()
        .vote(&pippin, &proposal_id, &0);
    e.jump(settings.vote_period);
    governor_client.mock_all_auths().close(&proposal_id);
    e.jump(settings.timelock);
    e.jump(settings.grace_period + 1);

    // remove any potential auth mocking
    e.set_auths(&[]);
    governor_client.set_auths(&[]);
    governor_client.execute(&proposal_id);

    // verify auths
    assert_eq!(e.auths().len(), 0);

    // verify chain results (proposal not executed)
    assert_eq!(token_client.balance(&samwise), 0);
    assert_eq!(
        token_client.balance(&governor_address),
        governor_transfer_amount
    );
    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Expired);

    // verify events
    let events = e.events().all();
    let tx_events = vec![&e, events.last().unwrap()];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                governor_address.clone(),
                (Symbol::new(&e, "proposal_expired"), proposal_id,).into_val(&e),
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
    let settings = default_governor_settings(&e);
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
    let settings = default_governor_settings(&e);
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = TokenVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_votes = 8_000 * 10i128.pow(7);
    let pippin_votes = 1_000 * 10i128.pow(7);
    let total_votes: i128 = 10_000 * 10i128.pow(7);
    let frodo_votes = total_votes - samwise_votes - pippin_votes;
    token_client.mock_all_auths().mint(&frodo, &frodo_votes);
    votes_client.mock_all_auths().deposit(&frodo, &frodo_votes);
    token_client.mock_all_auths().mint(&samwise, &samwise_votes);
    votes_client
        .mock_all_auths()
        .deposit(&samwise, &samwise_votes);
    token_client.mock_all_auths().mint(&pippin, &pippin_votes);
    votes_client
        .mock_all_auths()
        .deposit(&pippin, &pippin_votes);

    let governor_transfer_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &governor_transfer_amount);

    let (title, description, _) = default_proposal_data(&e);
    let action = ProposalAction::Calldata(Calldata {
        contract_id: token_address,
        function: Symbol::new(&e, "transfer"),
        args: (
            governor_address.clone(),
            samwise.clone(),
            governor_transfer_amount,
        )
            .into_val(&e),
        auths: vec![&e],
    });

    let proposal_id = governor_client.propose(&samwise, &title, &description, &action);

    e.jump(settings.vote_delay + 1);
    governor_client.vote(&samwise, &proposal_id, &1);
    governor_client.vote(&pippin, &proposal_id, &0);
    e.jump(settings.vote_period);
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
    let settings = default_governor_settings(&e);
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = TokenVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_votes = 8_000 * 10i128.pow(7);
    let pippin_votes = 1_000 * 10i128.pow(7);
    let total_votes: i128 = 10_000 * 10i128.pow(7);
    let frodo_votes = total_votes - samwise_votes - pippin_votes;
    token_client.mock_all_auths().mint(&frodo, &frodo_votes);
    votes_client.mock_all_auths().deposit(&frodo, &frodo_votes);
    token_client.mock_all_auths().mint(&samwise, &samwise_votes);
    votes_client
        .mock_all_auths()
        .deposit(&samwise, &samwise_votes);
    token_client.mock_all_auths().mint(&pippin, &pippin_votes);
    votes_client
        .mock_all_auths()
        .deposit(&pippin, &pippin_votes);

    let governor_transfer_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &governor_transfer_amount);

    let (title, description, _) = default_proposal_data(&e);
    let action = ProposalAction::Calldata(Calldata {
        contract_id: token_address,
        function: Symbol::new(&e, "transfer"),
        args: (
            governor_address.clone(),
            samwise.clone(),
            governor_transfer_amount,
        )
            .into_val(&e),
        auths: vec![&e],
    });

    let proposal_id = governor_client.propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    governor_client.vote(&samwise, &proposal_id, &1);
    governor_client.vote(&pippin, &proposal_id, &0);
    e.jump(settings.vote_period);
    governor_client.close(&proposal_id);
    e.jump(settings.timelock - 1);

    governor_client.execute(&proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #205)")]
fn test_execute_defeated_errors() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);
    let settings = default_governor_settings(&e);
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = TokenVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_votes = 8_000 * 10i128.pow(7);
    let pippin_votes = 1_000 * 10i128.pow(7);
    let total_votes: i128 = 10_000 * 10i128.pow(7);
    let frodo_votes = total_votes - samwise_votes - pippin_votes;
    token_client.mock_all_auths().mint(&frodo, &frodo_votes);
    votes_client.mock_all_auths().deposit(&frodo, &frodo_votes);
    token_client.mock_all_auths().mint(&samwise, &samwise_votes);
    votes_client
        .mock_all_auths()
        .deposit(&samwise, &samwise_votes);
    token_client.mock_all_auths().mint(&pippin, &pippin_votes);
    votes_client
        .mock_all_auths()
        .deposit(&pippin, &pippin_votes);

    let governor_transfer_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &governor_transfer_amount);

    let (title, description, _) = default_proposal_data(&e);
    let action = ProposalAction::Calldata(Calldata {
        contract_id: token_address,
        function: Symbol::new(&e, "transfer"),
        args: (
            governor_address.clone(),
            samwise.clone(),
            governor_transfer_amount,
        )
            .into_val(&e),
        auths: vec![&e],
    });

    let proposal_id = governor_client.propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    governor_client.vote(&samwise, &proposal_id, &0);
    governor_client.vote(&pippin, &proposal_id, &1);
    e.jump(settings.vote_period);
    governor_client.close(&proposal_id);

    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Defeated);

    e.jump(settings.timelock);

    governor_client.execute(&proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #205)")]
fn test_execute_snapshot_errors() {
    let e = Env::default();
    e.set_default_info();
    e.budget().reset_unlimited();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);

    let settings = default_governor_settings(&e);
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = TokenVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    // set intial votes
    let frodo_votes: i128 = 10_000 * 10i128.pow(7);
    token_client.mock_all_auths().mint(&frodo, &frodo_votes);
    votes_client.mock_all_auths().deposit(&frodo, &frodo_votes);

    // create a proposal
    let (title, description, _) = default_proposal_data(&e);
    let action = ProposalAction::Snapshot;

    let proposal_id =
        governor_client
            .mock_all_auths()
            .propose(&frodo, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    governor_client
        .mock_all_auths()
        .vote(&frodo, &proposal_id, &1);
    e.jump(settings.vote_period);
    governor_client.mock_all_auths().close(&proposal_id);
    e.jump(settings.timelock);

    // remove any potential auth mocking
    e.set_auths(&[]);
    governor_client.set_auths(&[]);
    governor_client.execute(&proposal_id);
}
