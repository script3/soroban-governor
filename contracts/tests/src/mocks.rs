use soroban_sdk::{Address, Env};

mod mock_subcall_wasm {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/mock_subcall.wasm"
    );
}

/// Create a WASM mock contract that executes subcalls
///
/// ### Arguments
/// * `token` - The underlying token address
/// * `governor` - The governor address
pub fn create_mock_subcall_contract_wasm<'a>(
    e: &Env,
    token: &Address,
    governor: &Address,
) -> (Address, mock_subcall_wasm::Client<'a>) {
    let vote_token_id = e.register_contract_wasm(None, mock_subcall_wasm::WASM);
    let vote_token_client = mock_subcall_wasm::Client::new(e, &vote_token_id);
    vote_token_client.initialize(&token, &governor);
    (vote_token_id, vote_token_client)
}
