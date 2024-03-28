#[cfg(test)]
use sep_41_token::testutils::MockTokenClient;
use soroban_governor::types::{Calldata, ProposalAction, ProposalStatus};
use soroban_governor::GovernorContractClient;
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _, Events},
    vec, Address, BytesN, Env, Error, IntoVal, Symbol, TryIntoVal, Val,
};
use tests::{
    env::EnvTestUtils,
    governor::{create_governor, default_governor_settings, default_proposal_data},
    votes::BondingVotesClient,
};

#[test]
fn test_propose_calldata() {
    let e = Env::default();
    e.mock_all_auths();

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
                        title.to_val(),
                        description.to_val(),
                        action.try_into_val(&e).unwrap()
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
    match proposal.config.action {
        ProposalAction::Calldata(calldata) => {
            assert_eq!(calldata.contract_id, calldata.contract_id);
            assert_eq!(calldata.function, calldata.function);
            assert_eq!(calldata.args, calldata.args);
            if let ProposalAction::Calldata(action_calldata) = action.clone() {
                assert_eq!(
                    calldata.auths.get(0).unwrap().contract_id,
                    action_calldata.auths.get(0).unwrap().contract_id
                );
                assert_eq!(
                    calldata.auths.get(0).unwrap().function,
                    action_calldata.auths.get(0).unwrap().function
                );
                assert_eq!(
                    calldata.auths.get(0).unwrap().args,
                    action_calldata.auths.get(0).unwrap().args
                );
                assert_eq!(calldata.auths.get(0).unwrap().auths.len(), 0);
            } else {
                assert!(false, "test setup error");
            }
        }
        _ => assert!(false, "expected calldata proposal action"),
    }
    assert_eq!(proposal.config.title, title);
    assert_eq!(proposal.config.description, description);
    assert_eq!(proposal.data.creator, samwise);
    assert_eq!(proposal.data.vote_start, settings.vote_delay);
    assert_eq!(
        proposal.data.vote_end,
        settings.vote_delay + settings.vote_period
    );
    assert_eq!(proposal.data.status, ProposalStatus::Open);

    let votes = governor_client.get_proposal_votes(&proposal_id);
    assert!(votes.is_some());
    let votes = votes.unwrap();
    assert_eq!(votes.against, 0);
    assert_eq!(votes._for, 0);
    assert_eq!(votes.abstain, 0);

    // verify events
    let events = e.events().all();
    let tx_events = vec![&e, events.last().unwrap()];
    let event_data: soroban_sdk::Vec<Val> = vec![
        &e,
        title.into_val(&e),
        description.into_val(&e),
        action.try_into_val(&e).unwrap(),
        proposal.data.vote_start.into_val(&e),
        proposal.data.vote_end.into_val(&e),
    ];
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
#[should_panic(expected = "Error(Contract, #213)")]
fn test_propose_calldata_validates() {
    let e = Env::default();
    e.mock_all_auths();

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

    let (title, description, _) = default_proposal_data(&e);
    let calldata = Calldata {
        contract_id: governor_address,
        function: Symbol::new(&e, "test"),
        args: (1, 2, 3).into_val(&e),
        auths: vec![&e],
    };
    let action = ProposalAction::Calldata(calldata);

    governor_client.propose(&samwise, &title, &description, &action);
}

#[test]
fn test_propose_with_active_proposal() {
    let e = Env::default();
    e.mock_all_auths();

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

    let (title, description, _) = default_proposal_data(&e);
    let action = ProposalAction::Snapshot;

    let proposal_id = governor_client.propose(&samwise, &title, &description, &action);
    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.id, 0);
    assert_eq!(proposal.data.creator, samwise);
    assert_eq!(proposal.data.status, ProposalStatus::Open);

    e.jump(settings.vote_delay + 1);

    // verify additional proposal cannot be made
    let bytesn = BytesN::<32>::random(&e);
    let action2 = ProposalAction::Upgrade(bytesn);
    let result = governor_client.try_propose(&samwise, &title, &description, &action2);
    assert_eq!(result.err(), Some(Ok(Error::from_contract_error(211))));
}

#[test]
fn test_propose_snapshot() {
    let e = Env::default();
    e.mock_all_auths();

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

    let (title, description, _) = default_proposal_data(&e);
    let action = ProposalAction::Snapshot;

    let proposal_id = governor_client.propose(&samwise, &title, &description, &action);
    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.id, 0);
    assert_eq!(proposal.config.title, title);
    assert_eq!(proposal.config.description, description);
    matches!(proposal.config.action, ProposalAction::Snapshot);
    assert_eq!(proposal.data.creator, samwise);
    assert_eq!(proposal.data.vote_start, e.ledger().sequence());
    assert_eq!(
        proposal.data.vote_end,
        e.ledger().sequence() + settings.vote_period
    );
    assert_eq!(proposal.data.status, ProposalStatus::Open);
}

#[test]
fn test_propose_upgrade() {
    let e = Env::default();
    e.mock_all_auths();

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

    let (title, description, _) = default_proposal_data(&e);
    let bytes = BytesN::<32>::random(&e);
    let action = ProposalAction::Upgrade(bytes);

    let proposal_id = governor_client.propose(&samwise, &title, &description, &action);

    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.id, 0);
    assert_eq!(proposal.config.title, title);
    assert_eq!(proposal.config.description, description);
    matches!(proposal.config.action, ProposalAction::Upgrade(_));
    assert_eq!(proposal.data.creator, samwise);
    assert_eq!(
        proposal.data.vote_start,
        e.ledger().sequence() + settings.vote_delay
    );
    assert_eq!(
        proposal.data.vote_end,
        e.ledger().sequence() + settings.vote_delay + settings.vote_period
    );
    assert_eq!(proposal.data.status, ProposalStatus::Open);
}

#[test]
fn test_propose_settings() {
    let e = Env::default();
    e.mock_all_auths();

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

    let (title, description, _) = default_proposal_data(&e);
    let mut new_settings = settings.clone();
    new_settings.vote_delay = 123;
    let action = ProposalAction::Settings(new_settings);

    let proposal_id = governor_client.propose(&samwise, &title, &description, &action);

    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.id, 0);
    assert_eq!(proposal.config.title, title);
    assert_eq!(proposal.config.description, description);
    matches!(proposal.config.action, ProposalAction::Settings(_));
    assert_eq!(proposal.data.creator, samwise);
    assert_eq!(
        proposal.data.vote_start,
        e.ledger().sequence() + settings.vote_delay
    );
    assert_eq!(
        proposal.data.vote_end,
        e.ledger().sequence() + settings.vote_delay + settings.vote_period
    );
    assert_eq!(proposal.data.status, ProposalStatus::Open);
}

#[test]
#[should_panic(expected = "Error(Contract, #200)")]
fn test_propose_settings_validates() {
    let e = Env::default();
    e.mock_all_auths();

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

    let (title, description, _) = default_proposal_data(&e);
    let mut new_settings = settings.clone();
    new_settings.vote_delay = 5 * 17280;
    new_settings.vote_period = 5 * 17280;
    new_settings.timelock = 7 * 17280;
    new_settings.grace_period = 7 * 17280 + 1;
    let action = ProposalAction::Settings(new_settings.clone());

    governor_client.propose(&samwise, &title, &description, &action);
}

#[test]
#[should_panic(expected = "Error(Contract, #208)")]
fn test_propose_below_proposal_threshold() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let settings = default_governor_settings(&e);
    let (governor_address, token_address, votes_address) =
        create_governor(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = BondingVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let samwise_mint_amount: i128 = 999_999;
    token_client.mint(&samwise, &samwise_mint_amount);
    votes_client.deposit(&samwise, &samwise_mint_amount);

    let (title, description, action) = default_proposal_data(&e);

    governor_client.propose(&samwise, &title, &description, &action);
}
