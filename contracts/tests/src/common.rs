use soroban_governor::{storage::GovernorSettings, GovernorContract, GovernorContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env};
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
        proposal_threshold: 10_000_000,
        vote_delay: 60 * 60 * 24,
        vote_period: 60 * 60 * 24 * 7,
        timelock: 60 * 60 * 24,
        quorum: 80,
        counting_type: 5,
        vote_threshold: 51,
    };
    govenor.initialize(&votes, &settings);
    return (address, votes, settings, govenor);
}
