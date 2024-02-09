use soroban_governor::{
    types::{Calldata, GovernorSettings, SubCalldata},
    GovernorContract, GovernorContractClient,
};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, IntoVal, String, Symbol, Vec};

mod governor_contract_wasm {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/optimized/soroban_governor.wasm"
    );
}

/// Create a governor contract
///
/// ### Arguments
/// * `votes` - The address of the voting token
/// * `settings` - The settings for the governor
pub fn create_governor<'a>(
    e: &Env,
    votes: &Address,
    settings: &GovernorSettings,
) -> (Address, GovernorContractClient<'a>) {
    let governor_address = e.register_contract(None, GovernorContract {});
    let govenor_client: GovernorContractClient<'a> =
        GovernorContractClient::new(&e, &governor_address);
    govenor_client.initialize(&votes, settings);
    return (governor_address, govenor_client);
}

/// Create a governor contract with the wasm contract
///
/// ### Arguments
/// * `votes` - The address of the voting token
/// * `settings` - The settings for the governor
pub fn create_governor_wasm<'a>(
    e: &Env,
    votes: &Address,
    settings: &GovernorSettings,
) -> (Address, GovernorContractClient<'a>) {
    let governor_address = e.register_contract_wasm(None, governor_contract_wasm::WASM);
    let govenor_client: GovernorContractClient<'a> =
        GovernorContractClient::new(&e, &governor_address);
    govenor_client.initialize(&votes, settings);
    return (governor_address, govenor_client);
}

/// Default governor settings
pub fn default_governor_settings() -> GovernorSettings {
    GovernorSettings {
        proposal_threshold: 1_0000000,
        vote_delay: 60 * 60 * 24,
        vote_period: 60 * 60 * 24 * 7,
        timelock: 60 * 60 * 24,
        quorum: 100,          // 1%
        counting_type: 4,     // 0x001 (for)
        vote_threshold: 5100, // 51%
    }
}

/// Default test proposal data - cannot be submitted
pub fn default_proposal_data(e: &Env) -> (Calldata, Vec<SubCalldata>, String, String) {
    let calldata = Calldata {
        contract_id: Address::generate(&e),
        function: Symbol::new(e, "test"),
        args: (1, 2, 3).into_val(e),
    };
    let sub_calldata = vec![
        e,
        SubCalldata {
            contract_id: Address::generate(e),
            function: Symbol::new(e, "test"),
            args: (1, 2, 3).into_val(e),
            sub_auth: vec![e],
        },
    ];
    let title = String::from_str(e, "Test Title");
    let description = String::from_str(e, "Test Description");
    return (calldata, sub_calldata, title, description);
}
