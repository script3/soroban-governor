#[cfg(test)]
use soroban_governor::{GovernorContract, GovernorContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env};
use tests::{
    env::EnvTestUtils,
    governor::{create_governor, default_governor_settings},
    votes::create_soroban_token_votes_wasm,
};

#[test]
fn test_initialize_sets_storage() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let settings = default_governor_settings();
    let governor_address = e.register_contract(None, GovernorContract {});
    let (vote_address, _) = create_soroban_token_votes_wasm(&e, &bombadil, &governor_address);

    let governor_client = GovernorContractClient::new(&e, &governor_address);
    governor_client.initialize(&vote_address, &bombadil, &settings);

    let result = governor_client.settings();
    assert_eq!(result.counting_type, settings.counting_type);
    assert_eq!(result.proposal_threshold, settings.proposal_threshold);
    assert_eq!(result.quorum, settings.quorum);
    assert_eq!(result.timelock, settings.timelock);
    assert_eq!(result.vote_delay, settings.vote_delay);
    assert_eq!(result.vote_period, settings.vote_period);
    assert_eq!(result.vote_threshold, settings.vote_threshold);

    let council = governor_client.council();
    assert_eq!(council, bombadil);

    let vote_token = governor_client.vote_token();
    assert_eq!(vote_token, vote_address);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_initalize_already_initalized() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let settings = default_governor_settings();
    let (governor_address, _, votes_address) = create_governor(&e, &bombadil, &bombadil, &settings);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    governor_client.initialize(&votes_address, &bombadil, &settings);
}

#[test]
#[should_panic(expected = "Error(Contract, #200)")]
fn test_initialize_vote_period_exceeds_max() {
    let e = Env::default();
    let address = e.register_contract(None, GovernorContract {});
    let govenor: GovernorContractClient<'_> = GovernorContractClient::new(&e, &address);
    let votes = Address::generate(&e);
    let mut settings = default_governor_settings();
    settings.vote_period = 7 * 17280 + 1;

    govenor.initialize(&votes, &Address::generate(&e), &settings);
}

#[test]
#[should_panic(expected = "Error(Contract, #200)")]
fn test_initialize_proposal_exceeds_max_lifetime() {
    let e = Env::default();
    let address = e.register_contract(None, GovernorContract {});
    let govenor: GovernorContractClient<'_> = GovernorContractClient::new(&e, &address);
    let votes = Address::generate(&e);
    let mut settings = default_governor_settings();
    settings.vote_delay = 5 * 17280;
    settings.vote_period = 5 * 17280;
    settings.timelock = 7 * 17280;
    settings.grace_period = 7 * 17280 + 1;

    govenor.initialize(&votes, &Address::generate(&e), &settings);
}
