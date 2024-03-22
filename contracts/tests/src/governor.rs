use soroban_governor::{
    types::{Calldata, GovernorSettings, ProposalAction},
    GovernorContract, GovernorContractClient,
};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, IntoVal, String, Symbol};

use crate::{common, votes, ONE_DAY_LEDGERS};

mod governor_contract_wasm {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/optimized/soroban_governor.wasm"
    );
}

/// Create a governor contract
///
/// Returns (governor, underlying_token, vote_token)
///
/// ### Arguments
/// * `admin` - The address of the admin
/// * `settings` - The settings for the governor
pub fn create_governor<'a>(
    e: &Env,
    admin: &Address,
    settings: &GovernorSettings,
) -> (Address, Address, Address) {
    let governor_address = e.register_contract(None, GovernorContract {});
    let (underlying_token, _) = common::create_stellar_token(e, admin);
    let (vote_address, _) =
        votes::create_staking_token_votes(e, &underlying_token, &governor_address);
    let govenor_client: GovernorContractClient<'a> =
        GovernorContractClient::new(&e, &governor_address);
    govenor_client.initialize(&vote_address, settings);
    return (governor_address, underlying_token, vote_address);
}

/// Create a governor contract with the wasm contract
///
/// Returns (governor, underlying_token, vote_token)
///
/// ### Arguments
/// * `admin` - The address of the admin
/// * `settings` - The settings for the governor
pub fn create_governor_wasm<'a>(
    e: &Env,
    admin: &Address,
    settings: &GovernorSettings,
) -> (Address, Address, Address) {
    let governor_address = e.register_contract_wasm(None, governor_contract_wasm::WASM);
    let (underlying_token, _) = common::create_stellar_token(e, admin);
    let (vote_address, _) =
        votes::create_staking_token_votes_wasm(e, &underlying_token, &governor_address);
    let govenor_client: GovernorContractClient<'a> =
        GovernorContractClient::new(&e, &governor_address);
    govenor_client.initialize(&vote_address, settings);
    return (governor_address, underlying_token, vote_address);
}

/// Create a governor contract with the wasm contract and a soroban vote token
///
/// Returns (governor, vote_token, vote_token)
///
/// ### Arguments
/// * `admin` - The address of the admin
/// * `settings` - The settings for the governor
pub fn create_soroban_governor_wasm<'a>(
    e: &Env,
    admin: &Address,
    settings: &GovernorSettings,
) -> (Address, Address) {
    let governor_address = e.register_contract_wasm(None, governor_contract_wasm::WASM);
    let (vote_address, _) = votes::create_soroban_token_votes_wasm(e, &admin, &governor_address);
    let govenor_client: GovernorContractClient<'a> =
        GovernorContractClient::new(&e, &governor_address);
    govenor_client.initialize(&vote_address, settings);
    return (governor_address, vote_address);
}

/// Default governor settings
pub fn default_governor_settings(e: &Env) -> GovernorSettings {
    GovernorSettings {
        council: Address::generate(e),
        proposal_threshold: 1_0000000,
        vote_delay: ONE_DAY_LEDGERS,
        vote_period: ONE_DAY_LEDGERS * 7,
        timelock: ONE_DAY_LEDGERS,
        grace_period: ONE_DAY_LEDGERS * 7,
        quorum: 100,          // 1%
        counting_type: 2,     // 0x...010 (for)
        vote_threshold: 5100, // 51%
    }
}

/// Default test proposal information
pub fn default_proposal_data(e: &Env) -> (String, String, ProposalAction) {
    let calldata = Calldata {
        contract_id: Address::generate(&e),
        function: Symbol::new(e, "test"),
        args: (1, 2, 3).into_val(e),
        auths: vec![
            e,
            Calldata {
                contract_id: Address::generate(e),
                function: Symbol::new(e, "test"),
                args: (1, 2, 3).into_val(e),
                auths: vec![e],
            },
        ],
    };
    let title = String::from_str(e, "Test Title");
    let description = String::from_str(e, "# This is a cool proposal");

    (title, description, ProposalAction::Calldata(calldata))
}
