use soroban_sdk::{Address, Env, String};
use soroban_votes::{TokenVotes, TokenVotesClient};

mod token_votes_wasm {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/optimized/soroban_votes.wasm"
    );
}

mod wrapped_token_votes_wasm {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/optimized/soroban_votes_wrapped.wasm"
    );
}

/// Create a voting token contract for an underyling token
///
/// ### Arguments
/// * `token` - The underlying token address
pub fn create_wrapped_token_votes<'a>(
    e: &Env,
    token: &Address,
    governor: &Address,
) -> (Address, TokenVotesClient<'a>) {
    let vote_token_id = e.register_contract(None, TokenVotes {});
    let vote_token_client = TokenVotesClient::new(e, &vote_token_id);
    vote_token_client.initialize(&token, &governor);
    (vote_token_id, vote_token_client)
}

/// Create a WASM voting token contract for an underyling token
///
/// ### Arguments
/// * `token` - The underlying token address
pub fn create_wrapped_token_votes_wasm<'a>(
    e: &Env,
    token: &Address,
    governor: &Address,
) -> (Address, TokenVotesClient<'a>) {
    let vote_token_id = e.register_contract_wasm(None, wrapped_token_votes_wasm::WASM);
    let vote_token_client = TokenVotesClient::new(e, &vote_token_id);
    vote_token_client.initialize(&token, &governor);
    (vote_token_id, vote_token_client)
}

/// Create a WASM soroban voting token contract
///
/// ### Arguments
/// * `token` - The underlying token address
pub fn create_soroban_token_votes_wasm<'a>(
    e: &Env,
    admin: &Address,
    governor: &Address,
) -> (Address, token_votes_wasm::Client<'a>) {
    let vote_token_id = e.register_contract_wasm(None, token_votes_wasm::WASM);
    let vote_token_client = token_votes_wasm::Client::new(e, &vote_token_id);
    vote_token_client.initialize(
        &admin,
        &governor,
        &7,
        &String::from_str(e, "Voting Token"),
        &String::from_str(e, "VOTES"),
    );
    (vote_token_id, vote_token_client)
}
