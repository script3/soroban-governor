use soroban_governor::types::ProposalStatus;
#[cfg(test)]
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    vec, Address, Env, IntoVal, Symbol, TryIntoVal,
};
use tests::{
    common::{
        create_govenor, create_stellar_token, create_token_votes, default_governor_settings,
        default_proposal_data,
    },
    env::EnvTestUtils,
};

#[test]
fn test_cancel() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let settings = default_governor_settings();
    let (govenor_address, governor_client) = create_govenor(&e, &votes_address, &settings);

    let samwise_votes: i128 = 1 * 10i128.pow(7);
    token_client.mint(&samwise, &samwise_votes);
    votes_client.deposit_for(&samwise, &samwise_votes);

    let (calldata, sub_calldata, title, description) = default_proposal_data(&e);

    // setup a proposal
    let proposal_id =
        governor_client.propose(&samwise, &calldata, &sub_calldata, &title, &description);
    e.jump_with_sequence(settings.vote_delay / 2);

    governor_client.cancel(&samwise, &proposal_id);

    // verify auths
    assert_eq!(
        e.auths()[0],
        (
            samwise.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    govenor_address.clone(),
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
                govenor_address.clone(),
                (Symbol::new(&e, "proposal_canceled"), proposal_id).into_val(&e),
                ().into_val(&e)
            )
        ]
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn test_cancel_nonexistent_proposal() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let (token_address, _) = create_stellar_token(&e, &bombadil);
    let (votes_address, _) = create_token_votes(&e, &token_address);
    let settings = default_governor_settings();
    let (_, governor_client) = create_govenor(&e, &votes_address, &settings);

    governor_client.cancel(&samwise, &1);
}

#[test]
#[should_panic(expected = "Error(Contract, #207)")]
fn test_cancel_proposal_active() {
    let e = Env::default();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_address, votes_client) = create_token_votes(&e, &token_address);
    let settings = default_governor_settings();
    let (_, governor_client) = create_govenor(&e, &votes_address, &settings);

    let samwise_mint_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &samwise_mint_amount);
    votes_client.deposit_for(&samwise, &samwise_mint_amount);

    let (calldata, sub_calldata, title, description) = default_proposal_data(&e);

    // setup a proposal, vote to make it active
    let proposal_id =
        governor_client.propose(&samwise, &calldata, &sub_calldata, &title, &description);
    e.jump_with_sequence(settings.vote_delay);
    governor_client.vote(&samwise, &proposal_id, &2);

    governor_client.cancel(&samwise, &proposal_id);
}
