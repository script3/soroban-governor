#[cfg(test)]
use sep_41_token::testutils::MockTokenClient;
use soroban_governor::types::ProposalStatus;
use soroban_governor::GovernorContractClient;
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    vec, Address, Env, IntoVal, Symbol, TryIntoVal, Val,
};
use soroban_votes::TokenVotesClient;
use tests::{
    env::EnvTestUtils,
    governor::{create_governor, default_governor_settings, default_proposal_data},
};

#[test]
fn test_propose() {
    let e = Env::default();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings();
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = TokenVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_mint_amount: i128 = 10_000_000;
    token_client.mint(&samwise, &samwise_mint_amount);
    votes_client.deposit_for(&samwise, &samwise_mint_amount);

    let (calldata, sub_calldata, title, description) = default_proposal_data(&e);

    let proposal_id =
        governor_client.propose(&samwise, &calldata, &sub_calldata, &title, &description);

    // verify auth
    assert_eq!(
        e.auths()[0],
        (
            samwise.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    governor_address.clone(),
                    Symbol::new(&e, "propose"),
                    vec![
                        &e,
                        samwise.to_val(),
                        calldata.try_into_val(&e).unwrap(),
                        sub_calldata.to_val(),
                        title.to_val(),
                        description.to_val()
                    ]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    // verify chain results
    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.id, proposal_id);
    assert_eq!(proposal.id, 0);
    assert_eq!(proposal.config.calldata.function, calldata.function);
    assert_eq!(proposal.config.calldata.contract_id, calldata.contract_id);
    assert_eq!(proposal.config.calldata.args, calldata.args);
    assert_eq!(
        proposal.config.sub_calldata.get(0).unwrap().contract_id,
        sub_calldata.get(0).unwrap().contract_id
    );
    assert_eq!(
        proposal.config.sub_calldata.get(0).unwrap().function,
        sub_calldata.get(0).unwrap().function
    );
    assert_eq!(
        proposal.config.sub_calldata.get(0).unwrap().args,
        sub_calldata.get(0).unwrap().args
    );
    assert_eq!(
        proposal.config.sub_calldata.get(0).unwrap().sub_auth.len(),
        sub_calldata.get(0).unwrap().sub_auth.len()
    );
    assert_eq!(proposal.id, 0);
    assert_eq!(proposal.config.proposer, samwise);
    assert_eq!(proposal.config.title, title);
    assert_eq!(proposal.config.description, description);
    assert_eq!(proposal.data.vote_start, settings.vote_delay);
    assert_eq!(
        proposal.data.vote_end,
        settings.vote_delay + settings.vote_period
    );

    assert_eq!(proposal.data.status, ProposalStatus::Pending);

    // verify events
    let events = e.events().all();
    let tx_events = vec![&e, events.last().unwrap()];
    let event_data: soroban_sdk::Vec<Val> = vec![&e, title.into_val(&e), calldata.into_val(&e)];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                governor_address.clone(),
                (
                    Symbol::new(&e, "proposal_created"),
                    proposal_id,
                    samwise.clone()
                )
                    .into_val(&e),
                event_data.into_val(&e)
            )
        ]
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #208)")]
fn test_propose_below_proposal_threshold() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings();
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = TokenVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_mint_amount: i128 = 999_999;
    token_client.mint(&samwise, &samwise_mint_amount);
    votes_client.deposit_for(&samwise, &samwise_mint_amount);

    let (calldata, sub_calldata, title, description) = default_proposal_data(&e);

    governor_client.propose(&samwise, &calldata, &sub_calldata, &title, &description);
}
