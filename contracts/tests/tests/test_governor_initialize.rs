#[cfg(test)]
use soroban_governor::{
    storage::{self, GovernorSettings},
    GovernorContract, GovernorContractClient,
};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};
use tests::common::create_govenor;
#[test]
fn test_initialize_sets_storage() {
    let e = Env::default();
    let (govenor_address, votes_address, settings, _) = create_govenor(&e);
    e.as_contract(&govenor_address, || {
        let storage_settings: GovernorSettings = storage::get_settings(&e);

        assert!(storage::get_is_init(&e));
        assert_eq!(storage::get_voter_token_address(&e), votes_address);
        assert_eq!(storage_settings.counting_type, settings.counting_type);
        assert_eq!(
            storage_settings.proposal_threshold,
            settings.proposal_threshold
        );
        assert_eq!(storage_settings.quorum, settings.quorum);
        assert_eq!(storage_settings.timelock, settings.timelock);
        assert_eq!(storage_settings.vote_delay, settings.vote_delay);
        assert_eq!(storage_settings.vote_period, settings.vote_period);
        assert_eq!(storage_settings.vote_threshold, settings.vote_threshold);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_initalize_already_initalized() {
    let e = Env::default();
    let (_, votes_address, settings, govenor) = create_govenor(&e);
    govenor.initialize(&votes_address, &settings);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
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
