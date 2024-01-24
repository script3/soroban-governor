#![cfg(test)]
use crate::contract::{GovernorContract, GovernorContractClient};
use crate::storage::GovernorSettings;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Address;
use soroban_sdk::Env;

/// Creates and initializes a Governor contract
/// Returns governor_address, voter_address, governor settings, and governor client
pub fn create_govenor<'a>(
    e: &Env,
) -> (
    Address,
    Address,
    GovernorSettings,
    GovernorContractClient<'a>,
) {
    let address = e.register_contract(None, GovernorContract {});
    let govenor: GovernorContractClient<'a> = GovernorContractClient::new(&e, &address);
    let votes = Address::generate(&e);
    let settings = GovernorSettings {
        proposal_threshold: 1000,
        vote_delay: 2000,
        vote_period: 3000,
        timelock: 4000,
        quorum: 5000,
        counting_type: 6000,
        vote_threshold: 7000,
    };
    govenor.initialize(&votes, &settings);
    return (address, votes, settings, govenor);
}
