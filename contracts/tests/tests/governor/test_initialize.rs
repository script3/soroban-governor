#[cfg(test)]
use soroban_governor::types::GovernorSettings;
use soroban_governor::{GovernorContract, GovernorContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env};
use tests::{
    env::EnvTestUtils,
    governor::{create_governor, default_governor_settings},
};

#[test]
fn test_initialize_sets_storage() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let settings = default_governor_settings();
    let (governor_address, _, _) = create_governor(&e, &bombadil, &settings);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let result = governor_client.settings();
    assert_eq!(result.counting_type, settings.counting_type);
    assert_eq!(result.proposal_threshold, settings.proposal_threshold);
    assert_eq!(result.quorum, settings.quorum);
    assert_eq!(result.timelock, settings.timelock);
    assert_eq!(result.vote_delay, settings.vote_delay);
    assert_eq!(result.vote_period, settings.vote_period);
    assert_eq!(result.vote_threshold, settings.vote_threshold);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_initalize_already_initalized() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let settings = default_governor_settings();
    let (governor_address, _, votes_address) = create_governor(&e, &bombadil, &settings);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    governor_client.initialize(&votes_address, &settings);
}

#[test]
#[should_panic(expected = "Error(Contract, #200)")]
fn test_initalize_proprosal_exceeds_time_length() {
    let e = Env::default();
    let address = e.register_contract(None, GovernorContract {});
    let govenor: GovernorContractClient<'_> = GovernorContractClient::new(&e, &address);
    let votes = Address::generate(&e);
    let settings = GovernorSettings {
        proposal_threshold: 1000,
        vote_delay: 500000,
        vote_period: 500000,
        timelock: 814401,
        quorum: 5000,
        counting_type: 6000,
        vote_threshold: 7000,
    };
    govenor.initialize(&votes, &settings);
}

#[test]
#[should_panic(expected = "Error(Contract, #200)")]
fn test_initalize_proprosal_exceeds_vote_period() {
    let e = Env::default();
    let address = e.register_contract(None, GovernorContract {});
    let govenor: GovernorContractClient<'_> = GovernorContractClient::new(&e, &address);
    let votes = Address::generate(&e);
    let mut settings = default_governor_settings();
    settings.vote_period = 7 * 24 * 60 * 60 + 1;
    govenor.initialize(&votes, &settings);
}
