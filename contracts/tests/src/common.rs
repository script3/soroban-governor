use sep_41_token::testutils::{MockTokenClient, MockTokenWASM};
use soroban_governor::{storage::GovernorSettings, GovernorContract, GovernorContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, IntoVal};
use soroban_votes::{TokenVotes, TokenVotesClient};

//********** Governor **********//

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

//********** Votes **********//

/// Create a voting token for an underyling token
///
/// ### Arguments
/// * `token` - The underlying token address
pub fn create_token_votes<'a>(e: &Env, token: &Address) -> (Address, TokenVotesClient<'a>) {
    let vote_token_id = e.register_contract(None, TokenVotes {});
    let vote_token_client = TokenVotesClient::new(e, &vote_token_id);
    vote_token_client.initialize(&token);
    (vote_token_id, vote_token_client)
}

//********** Token **********//

pub fn create_stellar_token<'a>(e: &Env, admin: &Address) -> (Address, MockTokenClient<'a>) {
    let contract_id = e.register_stellar_asset_contract(admin.clone());
    let client = MockTokenClient::new(e, &contract_id);
    // set admin to bump instance
    client.set_admin(admin);
    (contract_id, client)
}

pub fn create_token<'a>(
    e: &Env,
    admin: &Address,
    decimals: u32,
    symbol: &str,
) -> (Address, MockTokenClient<'a>) {
    let contract_id = Address::generate(e);
    e.register_contract_wasm(&contract_id, MockTokenWASM);
    let client = MockTokenClient::new(e, &contract_id);
    client.initialize(
        admin,
        &decimals,
        &"test token".into_val(e),
        &symbol.into_val(e),
    );
    // set admin to bump instance
    client.set_admin(admin);
    (contract_id.clone(), client)
}
