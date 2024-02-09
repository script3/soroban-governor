use soroban_sdk::{Address, Env};
use soroban_votes::{TokenVotes, TokenVotesClient};

mod token_votes_wasm {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/optimized/soroban_votes.wasm"
    );
}

/// Create a voting token contract for an underyling token
///
/// ### Arguments
/// * `token` - The underlying token address
pub fn create_token_votes<'a>(e: &Env, token: &Address) -> (Address, TokenVotesClient<'a>) {
    let vote_token_id = e.register_contract(None, TokenVotes {});
    let vote_token_client = TokenVotesClient::new(e, &vote_token_id);
    vote_token_client.initialize(&token);
    (vote_token_id, vote_token_client)
}

/// Create a WASM voting token contract for an underyling token
///
/// ### Arguments
/// * `token` - The underlying token address
pub fn create_token_votes_wasm<'a>(e: &Env, token: &Address) -> (Address, TokenVotesClient<'a>) {
    let vote_token_id = e.register_contract_wasm(None, token_votes_wasm::WASM);
    let vote_token_client = TokenVotesClient::new(e, &vote_token_id);
    vote_token_client.initialize(&token);
    (vote_token_id, vote_token_client)
}
