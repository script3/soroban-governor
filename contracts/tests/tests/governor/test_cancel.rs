#[cfg(test)]
use sep_41_token::testutils::MockTokenClient;
use soroban_governor::{types::ProposalStatus, GovernorContractClient};
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    vec, Address, Env, IntoVal, Symbol, TryIntoVal,
};
use tests::{
    env::EnvTestUtils,
    governor::{create_governor, default_governor_settings, default_proposal_data},
    votes::StakingVotesClient,
};

#[test]
fn test_cancel() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings(&e);
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = StakingVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_votes: i128 = 1 * 10i128.pow(7);
    token_client.mint(&samwise, &samwise_votes);
    votes_client.deposit(&samwise, &samwise_votes);

    let (title, description, action) = default_proposal_data(&e);

    // setup a proposal
    let proposal_id = governor_client.propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay / 2);

    governor_client.cancel(&samwise, &proposal_id);

    // verify auths
    assert_eq!(
        e.auths()[0],
        (
            samwise.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    governor_address.clone(),
                    Symbol::new(&e, "cancel"),
                    vec![&e, samwise.to_val(), proposal_id.try_into_val(&e).unwrap()]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    // verify chain results
    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Canceled);

    // verify events
    let events = e.events().all();
    let tx_events = vec![&e, events.last().unwrap()];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                governor_address.clone(),
                (Symbol::new(&e, "proposal_canceled"), proposal_id).into_val(&e),
                ().into_val(&e)
            )
        ]
    );

    // verify creator can create another proposal
    let proposal_id_new = governor_client.propose(&samwise, &title, &description, &action);
    assert_eq!(proposal_id_new, proposal_id + 1);
}

#[test]
fn test_cancel_council() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings(&e);
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = StakingVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_votes: i128 = 1 * 10i128.pow(7);
    token_client.mint(&samwise, &samwise_votes);
    votes_client.deposit(&samwise, &samwise_votes);

    let (title, description, action) = default_proposal_data(&e);

    // setup a proposal
    let proposal_id = governor_client.propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay / 2);

    governor_client.cancel(&settings.council, &proposal_id);

    // verify auths
    assert_eq!(
        e.auths()[0],
        (
            settings.council.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    governor_address.clone(),
                    Symbol::new(&e, "cancel"),
                    vec![
                        &e,
                        settings.council.to_val(),
                        proposal_id.try_into_val(&e).unwrap()
                    ]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    // verify chain results
    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Canceled);

    // verify events
    let events = e.events().all();
    let tx_events = vec![&e, events.last().unwrap()];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                governor_address.clone(),
                (Symbol::new(&e, "proposal_canceled"), proposal_id,).into_val(&e),
                ().into_val(&e)
            )
        ]
    );

    // verify creator can create another proposal
    let proposal_id_new = governor_client.propose(&samwise, &title, &description, &action);
    assert_eq!(proposal_id_new, proposal_id + 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn test_cancel_nonexistent_proposal() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings(&e);
    let (governor_address, _, _) = create_governor(&e, &bombadil, &settings);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    governor_client.cancel(&samwise, &1);
}

#[test]
#[should_panic(expected = "Error(Contract, #207)")]
fn test_cancel_proposal_active() {
    let e = Env::default();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings(&e);
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = StakingVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_mint_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &samwise_mint_amount);
    votes_client.deposit(&samwise, &samwise_mint_amount);

    let (title, description, action) = default_proposal_data(&e);

    // setup a proposal, vote to make it active
    let proposal_id = governor_client.propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay + 1);

    governor_client.vote(&samwise, &proposal_id, &1);

    governor_client.cancel(&samwise, &proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_cancel_unauthorized_address() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings(&e);
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = StakingVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_votes: i128 = 1 * 10i128.pow(7);
    token_client.mint(&samwise, &samwise_votes);
    votes_client.deposit(&samwise, &samwise_votes);

    let (title, description, action) = default_proposal_data(&e);

    // setup a proposal
    let proposal_id = governor_client.propose(&samwise, &title, &description, &action);
    e.jump(settings.vote_delay / 2);

    governor_client.cancel(&bombadil, &proposal_id);
}
