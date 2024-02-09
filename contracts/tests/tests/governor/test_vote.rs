#[cfg(test)]
use soroban_governor::types::ProposalStatus;
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    vec, Address, Env, IntoVal, Symbol, TryIntoVal, Val,
};
use tests::{
    common::create_stellar_token,
    env::EnvTestUtils,
    governor::{create_governor, default_governor_settings, default_proposal_data},
    votes::create_token_votes,
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
    let (govenor_address, governor_client) = create_governor(&e, &votes_address, &settings);

    let total_votes: i128 = 10_000 * 10i128.pow(7);
    token_client.mint(&frodo, &total_votes);
    votes_client.deposit_for(&frodo, &total_votes);

    let samwise_votes = 8_000 * 10i128.pow(7);
    votes_client.transfer(&frodo, &samwise, &samwise_votes);

    let (calldata, sub_calldata, title, description) = default_proposal_data(&e);

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
    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Active);
    let vote_count = governor_client.get_proposal_votes(&proposal_id);
    assert_eq!(vote_count.votes_for, 0);
    assert_eq!(vote_count.votes_against, samwise_votes);
    assert_eq!(vote_count.votes_abstained, 0);

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
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let settings = default_governor_settings();
    let (_, governor_client) = create_governor(&e, &votes_address, &settings);

    let total_votes: i128 = 10_000 * 10i128.pow(7);
    token_client.mint(&frodo, &total_votes);
    votes_client.deposit_for(&frodo, &total_votes);

    let samwise_votes = 8_000 * 10i128.pow(7);
    votes_client.transfer(&frodo, &samwise, &samwise_votes);

    let (calldata, sub_calldata, title, description) = default_proposal_data(&e);

    // setup a proposal that can be voted on
    let proposal_id =
        governor_client.propose(&samwise, &calldata, &sub_calldata, &title, &description);
    e.jump_with_sequence(settings.vote_delay);

    let voter_support = 1;
    governor_client.vote(&samwise, &proposal_id, &voter_support);

    let votes = governor_client.get_vote(&samwise, &proposal_id);
    assert_eq!(votes, Some(voter_support));
    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Active);
    let vote_count = governor_client.get_proposal_votes(&proposal_id);
    assert_eq!(vote_count.votes_for, 0);
    assert_eq!(vote_count.votes_against, samwise_votes);
    assert_eq!(vote_count.votes_abstained, 0);

    let voter_support = 2;
    governor_client.vote(&samwise, &proposal_id, &voter_support);

    let votes = governor_client.get_vote(&samwise, &proposal_id);
    assert_eq!(votes, Some(voter_support));
    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Active);
    let vote_count = governor_client.get_proposal_votes(&proposal_id);
    assert_eq!(vote_count.votes_for, samwise_votes);
    assert_eq!(vote_count.votes_against, 0);
    assert_eq!(vote_count.votes_abstained, 0);
}

#[test]
fn test_vote_multiple_users() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);
    let merry = Address::generate(&e);
    let bilbo = Address::generate(&e);

    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let settings = default_governor_settings();
    let (_, governor_client) = create_governor(&e, &votes_address, &settings);

    let total_votes: i128 = 10_000 * 10i128.pow(7);
    token_client.mint(&frodo, &total_votes);
    votes_client.deposit_for(&frodo, &total_votes);

    let samwise_votes = 1_000 * 10i128.pow(7);
    votes_client.transfer(&frodo, &samwise, &samwise_votes);
    let pippin_votes = 500 * 10i128.pow(7);
    votes_client.transfer(&frodo, &pippin, &pippin_votes);
    let merry_votes = 1234567;
    votes_client.transfer(&frodo, &merry, &merry_votes);
    let bilbo_votes = 2345 * 10i128.pow(7);
    votes_client.transfer(&frodo, &bilbo, &bilbo_votes);

    let (calldata, sub_calldata, title, description) = default_proposal_data(&e);

    // setup a proposal that can be voted on
    let proposal_id =
        governor_client.propose(&samwise, &calldata, &sub_calldata, &title, &description);
    e.jump_with_sequence(settings.vote_delay);

    governor_client.vote(&samwise, &proposal_id, &2);
    e.jump_with_sequence(10);
    governor_client.vote(&pippin, &proposal_id, &1);
    e.jump_with_sequence(123);
    governor_client.vote(&merry, &proposal_id, &1);
    governor_client.vote(&bilbo, &proposal_id, &0);
    e.jump_with_sequence(50);

    let votes = governor_client.get_vote(&samwise, &proposal_id);
    assert_eq!(votes, Some(2));
    let votes = governor_client.get_vote(&pippin, &proposal_id);
    assert_eq!(votes, Some(1));
    let votes = governor_client.get_vote(&merry, &proposal_id);
    assert_eq!(votes, Some(1));
    let votes = governor_client.get_vote(&bilbo, &proposal_id);
    assert_eq!(votes, Some(0));
    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Active);
    let vote_count = governor_client.get_proposal_votes(&proposal_id);
    assert_eq!(vote_count.votes_for, samwise_votes);
    assert_eq!(vote_count.votes_against, pippin_votes + merry_votes);
    assert_eq!(vote_count.votes_abstained, bilbo_votes);
}

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn test_vote_nonexistent_proposal() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let settings = default_governor_settings();
    let (_, governor_client) = create_governor(&e, &votes_address, &settings);

    let total_votes: i128 = 10_000 * 10i128.pow(7);
    token_client.mint(&frodo, &total_votes);
    votes_client.deposit_for(&frodo, &total_votes);

    let samwise_votes = 8_000 * 10i128.pow(7);
    votes_client.transfer(&frodo, &samwise, &samwise_votes);

    let voter_support = 1;
    governor_client.vote(&samwise, &0, &voter_support);
}

#[test]
#[should_panic(expected = "Error(Contract, #202)")]
fn test_vote_delay_not_ended() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let settings = default_governor_settings();
    let (_, governor_client) = create_governor(&e, &votes_address, &settings);

    let total_votes: i128 = 10_000 * 10i128.pow(7);
    token_client.mint(&frodo, &total_votes);
    votes_client.deposit_for(&frodo, &total_votes);

    let samwise_votes = 8_000 * 10i128.pow(7);
    votes_client.transfer(&frodo, &samwise, &samwise_votes);

    let (calldata, sub_calldata, title, description) = default_proposal_data(&e);

    // setup a proposal that can be voted on
    let proposal_id =
        governor_client.propose(&samwise, &calldata, &sub_calldata, &title, &description);
    e.jump_with_sequence(settings.vote_delay - 1);

    governor_client.vote(&samwise, &proposal_id, &2);
}

#[test]
#[should_panic(expected = "Error(Contract, #203)")]
fn test_vote_invalid_support_option() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);

    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let settings = default_governor_settings();
    let (_, governor_client) = create_governor(&e, &votes_address, &settings);

    let total_votes: i128 = 10_000 * 10i128.pow(7);
    token_client.mint(&frodo, &total_votes);
    votes_client.deposit_for(&frodo, &total_votes);

    let samwise_votes = 8_000 * 10i128.pow(7);
    votes_client.transfer(&frodo, &samwise, &samwise_votes);

    let (calldata, sub_calldata, title, description) = default_proposal_data(&e);

    // setup a proposal that can be voted on
    let proposal_id =
        governor_client.propose(&samwise, &calldata, &sub_calldata, &title, &description);
    e.jump_with_sequence(settings.vote_delay);

    governor_client.vote(&samwise, &proposal_id, &3);
}
