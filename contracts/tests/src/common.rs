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
