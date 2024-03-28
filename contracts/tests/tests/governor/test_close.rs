use sep_41_token::testutils::MockTokenClient;
#[cfg(test)]
use soroban_governor::{storage, types::ProposalStatus};
use soroban_governor::{types::ProposalAction, GovernorContractClient};
use soroban_sdk::{
    testutils::{Address as _, Events},
    vec, Address, Env, Error, IntoVal, Symbol,
};
use tests::{
    env::EnvTestUtils,
    governor::{create_governor, default_governor_settings, default_proposal_data},
    votes::BondingVotesClient,
};

#[test]
fn test_close_successful() {
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
    let votes_client = BondingVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_votes = 105 * 10i128.pow(7);
    let pippin_votes = 100 * 10i128.pow(7);
    let total_votes: i128 = 10_000 * 10i128.pow(7);
    let frodo_votes = total_votes - samwise_votes - pippin_votes;
    token_client.mint(&frodo, &frodo_votes);
    votes_client.deposit(&frodo, &frodo_votes);

    token_client.mint(&samwise, &samwise_votes);
    votes_client.deposit(&samwise, &samwise_votes);

    token_client.mint(&pippin, &pippin_votes);
    votes_client.deposit(&pippin, &pippin_votes);

    let (title, description, action) = default_proposal_data(&e);

    let proposal_id = governor_client.propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    governor_client.vote(&samwise, &proposal_id, &1);
    governor_client.vote(&pippin, &proposal_id, &0);
    e.jump(settings.vote_period);

    governor_client.close(&proposal_id);

    // verify auth
    assert_eq!(e.auths().len(), 0);

    // verify chain results
    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Successful);
    assert_eq!(proposal.data.eta, e.ledger().sequence() + settings.timelock);

    // verify events
    let proposal_votes = governor_client.get_proposal_votes(&proposal_id);
    let events = e.events().all();
    let tx_events = vec![&e, events.last().unwrap()];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                governor_address.clone(),
                (
                    Symbol::new(&e, "proposal_voting_closed"),
                    proposal_id,
                    ProposalStatus::Successful as u32,
                    e.ledger().sequence() + settings.timelock
                )
                    .into_val(&e),
                proposal_votes.into_val(&e)
            )
        ]
    );

    // verify creator can create another proposal
    let proposal_id_new = governor_client.propose(&samwise, &title, &description, &action);
    assert_eq!(proposal_id_new, proposal_id + 1);
}

#[test]
fn test_close_defeated_quorum_not_met() {
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
    let votes_client = BondingVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_votes = 99 * 10i128.pow(7); // quorum is 1%
    let pippin_votes = 10 * 10i128.pow(7);
    let total_votes: i128 = 10_000 * 10i128.pow(7);
    let frodo_votes = total_votes - samwise_votes - pippin_votes;
    token_client.mint(&frodo, &frodo_votes);
    votes_client.deposit(&frodo, &frodo_votes);

    token_client.mint(&samwise, &samwise_votes);
    votes_client.deposit(&samwise, &samwise_votes);

    token_client.mint(&pippin, &pippin_votes);
    votes_client.deposit(&pippin, &pippin_votes);

    let (title, description, action) = default_proposal_data(&e);

    let proposal_id = governor_client.propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    governor_client.vote(&samwise, &proposal_id, &1);
    governor_client.vote(&pippin, &proposal_id, &0);
    e.jump(settings.vote_period);

    governor_client.close(&proposal_id);

    // verify chain results
    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Defeated);
    assert_eq!(proposal.data.eta, 0);

    // verify events
    let proposal_votes = governor_client.get_proposal_votes(&proposal_id);
    let events = e.events().all();
    let tx_events = vec![&e, events.last().unwrap()];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                governor_address.clone(),
                (
                    Symbol::new(&e, "proposal_voting_closed"),
                    proposal_id,
                    ProposalStatus::Defeated as u32,
                    0 as u32
                )
                    .into_val(&e),
                proposal_votes.into_val(&e)
            )
        ]
    );

    // verify creator can create another proposal
    let proposal_id_new = governor_client.propose(&samwise, &title, &description, &action);
    assert_eq!(proposal_id_new, proposal_id + 1);
}

#[test]
fn test_close_defeated_threshold_not_met() {
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
    let votes_client = BondingVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_votes = 200 * 10i128.pow(7);
    let pippin_votes = 200 * 10i128.pow(7);
    let total_votes: i128 = 10_000 * 10i128.pow(7);
    let frodo_votes = total_votes - samwise_votes - pippin_votes;
    token_client.mint(&frodo, &frodo_votes);
    votes_client.deposit(&frodo, &frodo_votes);

    token_client.mint(&samwise, &samwise_votes);
    votes_client.deposit(&samwise, &samwise_votes);

    token_client.mint(&pippin, &pippin_votes);
    votes_client.deposit(&pippin, &pippin_votes);

    let (title, description, action) = default_proposal_data(&e);

    let proposal_id = governor_client.propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    governor_client.vote(&samwise, &proposal_id, &1);
    governor_client.vote(&pippin, &proposal_id, &0);
    e.jump(settings.vote_period);

    governor_client.close(&proposal_id);

    // verify chain results
    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Defeated);
    assert_eq!(proposal.data.eta, 0);

    // verify events
    let proposal_votes = governor_client.get_proposal_votes(&proposal_id);
    let events = e.events().all();
    let tx_events = vec![&e, events.last().unwrap()];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                governor_address.clone(),
                (
                    Symbol::new(&e, "proposal_voting_closed"),
                    proposal_id,
                    ProposalStatus::Defeated as u32,
                    0 as u32
                )
                    .into_val(&e),
                proposal_votes.into_val(&e)
            )
        ]
    );

    // verify creator can create another proposal
    let proposal_id_new = governor_client.propose(&samwise, &title, &description, &action);
    assert_eq!(proposal_id_new, proposal_id + 1);
}

#[test]
fn test_close_expired() {
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
    let votes_client = BondingVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_votes = 105 * 10i128.pow(7);
    let pippin_votes = 100 * 10i128.pow(7);
    let total_votes: i128 = 10_000 * 10i128.pow(7);
    let frodo_votes = total_votes - samwise_votes - pippin_votes;
    token_client.mint(&frodo, &frodo_votes);
    votes_client.deposit(&frodo, &frodo_votes);

    token_client.mint(&samwise, &samwise_votes);
    votes_client.deposit(&samwise, &samwise_votes);

    token_client.mint(&pippin, &pippin_votes);
    votes_client.deposit(&pippin, &pippin_votes);

    let (title, description, action) = default_proposal_data(&e);

    let proposal_id = governor_client.propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    governor_client.vote(&samwise, &proposal_id, &1);
    governor_client.vote(&pippin, &proposal_id, &0);
    e.jump(settings.vote_period);

    e.jump(settings.grace_period + 1);

    governor_client.close(&proposal_id);

    // verify chain results
    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Expired);
    assert_eq!(proposal.data.eta, 0);

    // verify events
    let proposal_votes = governor_client.get_proposal_votes(&proposal_id);
    let events = e.events().all();
    let tx_events = vec![&e, events.last().unwrap()];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                governor_address.clone(),
                (
                    Symbol::new(&e, "proposal_voting_closed"),
                    proposal_id,
                    ProposalStatus::Expired as u32,
                    0 as u32
                )
                    .into_val(&e),
                proposal_votes.into_val(&e)
            )
        ]
    );

    // verify creator can create another proposal
    let proposal_id_new = governor_client.propose(&samwise, &title, &description, &action);
    assert_eq!(proposal_id_new, proposal_id + 1);
}

#[test]
fn test_close_tracks_quorum_with_counting_type() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);
    let merry = Address::generate(&e);

    let mut settings = default_governor_settings(&e);
    settings.counting_type = 0b011; // include against and abstain in quorum
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = BondingVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_votes = 50 * 10i128.pow(7);
    let pippin_votes = 10 * 10i128.pow(7);
    let merry_votes = 90 * 10i128.pow(7);
    let total_votes: i128 = 10_000 * 10i128.pow(7);
    let frodo_votes = total_votes - samwise_votes - pippin_votes - merry_votes;
    token_client.mint(&frodo, &frodo_votes);
    votes_client.deposit(&frodo, &frodo_votes);

    token_client.mint(&samwise, &samwise_votes);
    votes_client.deposit(&samwise, &samwise_votes);

    token_client.mint(&pippin, &pippin_votes);
    votes_client.deposit(&pippin, &pippin_votes);

    token_client.mint(&merry, &merry_votes);
    votes_client.deposit(&merry, &merry_votes);

    let (title, description, action) = default_proposal_data(&e);

    let proposal_id = governor_client.propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    governor_client.vote(&samwise, &proposal_id, &1);
    governor_client.vote(&pippin, &proposal_id, &0);
    governor_client.vote(&merry, &proposal_id, &2);
    e.jump(settings.vote_period);

    governor_client.close(&proposal_id);

    // verify chain results
    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Successful);
    assert_eq!(proposal.data.eta, e.ledger().sequence() + settings.timelock);
}

#[test]
fn test_close_successful_non_executable() {
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
    let votes_client = BondingVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_votes = 105 * 10i128.pow(7);
    let pippin_votes = 100 * 10i128.pow(7);
    let total_votes: i128 = 10_000 * 10i128.pow(7);
    let frodo_votes = total_votes - samwise_votes - pippin_votes;
    token_client.mint(&frodo, &frodo_votes);
    votes_client.deposit(&frodo, &frodo_votes);

    token_client.mint(&samwise, &samwise_votes);
    votes_client.deposit(&samwise, &samwise_votes);

    token_client.mint(&pippin, &pippin_votes);
    votes_client.deposit(&pippin, &pippin_votes);

    let (title, description, _) = default_proposal_data(&e);
    let action = ProposalAction::Snapshot;

    let proposal_id = governor_client.propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    governor_client.vote(&samwise, &proposal_id, &1);
    governor_client.vote(&pippin, &proposal_id, &0);
    e.jump(settings.vote_period);

    governor_client.close(&proposal_id);

    // verify chain results
    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Successful);
    assert_eq!(proposal.data.eta, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn test_close_nonexistent_proposal() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings(&e);
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = BondingVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_mint_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &samwise_mint_amount);
    votes_client.deposit(&samwise, &samwise_mint_amount);

    let proposal_id = e.as_contract(&governor_address, || {
        return storage::get_next_proposal_id(&e);
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
    let settings = default_governor_settings(&e);
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = BondingVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_mint_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &samwise_mint_amount);
    votes_client.deposit(&samwise, &samwise_mint_amount);

    let (title, description, action) = default_proposal_data(&e);

    let proposal_id = governor_client.propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    governor_client.vote(&samwise, &proposal_id, &1);
    e.jump(settings.vote_period - 1);

    governor_client.close(&proposal_id);
}

#[test]
fn test_close_already_closed() {
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
    let votes_client = BondingVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_votes = 105 * 10i128.pow(7);
    let pippin_votes = 100 * 10i128.pow(7);
    let total_votes: i128 = 10_000 * 10i128.pow(7);
    let frodo_votes = total_votes - samwise_votes - pippin_votes;
    token_client.mint(&frodo, &frodo_votes);
    votes_client.deposit(&frodo, &frodo_votes);

    token_client.mint(&samwise, &samwise_votes);
    votes_client.deposit(&samwise, &samwise_votes);

    token_client.mint(&pippin, &pippin_votes);
    votes_client.deposit(&pippin, &pippin_votes);

    let (title, description, action) = default_proposal_data(&e);

    let proposal_id = governor_client.propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);
    governor_client.vote(&samwise, &proposal_id, &1);
    governor_client.vote(&pippin, &proposal_id, &0);
    e.jump(settings.vote_period);

    // close
    governor_client.close(&proposal_id);

    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Successful);

    // close again
    let result = governor_client.try_close(&proposal_id);
    assert_eq!(result.err(), Some(Ok(Error::from_contract_error(202))));
}
