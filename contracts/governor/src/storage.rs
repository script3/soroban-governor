use soroban_sdk::{
    contracttype, unwrap::UnwrapOptimized, Address, Env, IntoVal, Symbol, TryFromVal, Val,
};

use crate::{
    constants::{MAX_PROPOSAL_LIFETIME, ONE_DAY_LEDGERS},
    types::{GovernorSettings, ProposalConfig, ProposalData, VoteCount},
};

const VOTER_TOKEN_ADDRESS_KEY: &str = "Votes";
const SETTINGS_KEY: &str = "Settings";
const IS_INIT_KEY: &str = "IsInit";
const PROPOSAL_ID_KEY: &str = "ProposalId";

// All stored data is used on a per proposal basis outside of the instance. Extend past the max possible
// proposal lifetime to ensure all data is available after the proposal is concluced.
const LEDGER_BUMP: u32 = 14 * ONE_DAY_LEDGERS + MAX_PROPOSAL_LIFETIME;
const LEDGER_THRESHOLD: u32 = LEDGER_BUMP - 3 * ONE_DAY_LEDGERS;

//********** Storage Keys **********//

// Key for storing Voter's decision
#[derive(Clone)]
#[contracttype]
pub struct VoterStatusKey {
    pub proposal_id: u32,
    pub voter: Address,
}

#[derive(Clone)]
#[contracttype]
pub enum GovernorDataKey {
    // A map of proposal id to proposal config
    Config(u32),
    // A map of proposal id to proposal data
    Data(u32),
    // The voter's decision
    VoterSup(VoterStatusKey),
    // The proposal results
    Votes(u32),
    // A flag for an active proposal by a creator
    Open(Address),
}

//********** Storage Utils **********//

/// Bump the instance lifetime by the defined amount
pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD, LEDGER_BUMP);
}

/// Fetch an entry in persistent storage that has a default value if it doesn't exist
fn get_persistent_default<K: IntoVal<Env, Val>, V: TryFromVal<Env, Val>>(
    e: &Env,
    key: &K,
    default: V,
    bump_threshold: u32,
    bump_amount: u32,
) -> V {
    if let Some(result) = e.storage().persistent().get::<K, V>(key) {
        e.storage()
            .persistent()
            .extend_ttl(key, bump_threshold, bump_amount);
        result
    } else {
        default
    }
}

/********** Instance **********/

/// Check if the contract has been initialized
pub fn get_is_init(e: &Env) -> bool {
    e.storage().instance().has(&Symbol::new(e, IS_INIT_KEY))
}

/// Set the contract as initialized
pub fn set_is_init(e: &Env) {
    e.storage()
        .instance()
        .set::<Symbol, bool>(&Symbol::new(e, IS_INIT_KEY), &true);
}

/// Set the voter token address
///
/// ### Arguments
/// * `voter` - The address of voter contract
pub fn set_voter_token_address(e: &Env, voter: &Address) {
    e.storage()
        .instance()
        .set::<Symbol, Address>(&Symbol::new(&e, VOTER_TOKEN_ADDRESS_KEY), &voter);
}

/// Get the voter token address
pub fn get_voter_token_address(e: &Env) -> Address {
    e.storage()
        .instance()
        .get::<Symbol, Address>(&Symbol::new(&e, VOTER_TOKEN_ADDRESS_KEY))
        .unwrap_optimized()
}

/// Set the contract settings
///
/// ### Arguments
/// * `settings` - The contract settings
pub fn set_settings(e: &Env, settings: &GovernorSettings) {
    e.storage()
        .instance()
        .set::<Symbol, GovernorSettings>(&Symbol::new(&e, SETTINGS_KEY), &settings);
}

/// Get the contract settings
pub fn get_settings(e: &Env) -> GovernorSettings {
    e.storage()
        .instance()
        .get::<Symbol, GovernorSettings>(&Symbol::new(&e, SETTINGS_KEY))
        .unwrap_optimized()
}

/********** Persistent **********/

/// Set the next proposal id and bump if necessary
///
/// ### Arguments
/// * `proposal_id` - The new proposal_id
pub fn set_next_proposal_id(e: &Env, proposal_id: u32) {
    let key = Symbol::new(&e, PROPOSAL_ID_KEY);
    e.storage()
        .persistent()
        .set::<Symbol, u32>(&key, &proposal_id);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

/// Get the current proposal id
pub fn get_next_proposal_id(e: &Env) -> u32 {
    let key = Symbol::new(&e, PROPOSAL_ID_KEY);
    get_persistent_default::<Symbol, u32>(&e, &key, 0_u32, LEDGER_THRESHOLD, LEDGER_BUMP)
}

/********** Temporary **********/

/***** Proposal Config *****/

/// Fetch proposal config at `proposal_id`
///
/// ### Arguments
/// * `proposal_id` - The id of the proposal to fetch
pub fn get_proposal_config(e: &Env, proposal_id: u32) -> Option<ProposalConfig> {
    let key = GovernorDataKey::Config(proposal_id);
    e.storage()
        .temporary()
        .get::<GovernorDataKey, ProposalConfig>(&key)
}

/// Create the proposal config at `proposal_id` and bump it for the life of the proposal.
///
/// ### Arguments
/// * `proposal_id` - The proposal id
/// * `proposal_config` - The proposal config to store
pub fn create_proposal_config(e: &Env, proposal_id: u32, proposal_data: &ProposalConfig) {
    let key = GovernorDataKey::Config(proposal_id);
    e.storage()
        .temporary()
        .set::<GovernorDataKey, ProposalConfig>(&key, proposal_data);
    e.storage()
        .temporary()
        .extend_ttl(&key, LEDGER_BUMP, LEDGER_BUMP);
}

/***** Proposal Data *****/

// Get the proposal data for proposal at `proposal_id`
///
/// ### Arguments
/// * `proposal_id` - The proposal status id
pub fn get_proposal_data(e: &Env, proposal_id: u32) -> Option<ProposalData> {
    let key = GovernorDataKey::Data(proposal_id);
    e.storage()
        .temporary()
        .get::<GovernorDataKey, ProposalData>(&key)
}

/// Set the proposal data for proposal at `proposal_id`.
///
/// Does not perform a ledger ttl bump.
///
/// ### Arguments
/// * `proposal_id` - The proposal id
pub fn set_proposal_data(e: &Env, proposal_id: u32, proposal_data: &ProposalData) {
    let key = GovernorDataKey::Data(proposal_id);
    e.storage()
        .temporary()
        .set::<GovernorDataKey, ProposalData>(&key, &proposal_data);
}

/// Create the proposal status for proposal at `proposal_id` and bump
/// it for the life of the proposal.
///
/// ### Arguments
/// * `proposal_id` - The proposal id
pub fn create_proposal_data(e: &Env, proposal_id: u32, proposal_data: &ProposalData) {
    let key = GovernorDataKey::Data(proposal_id);
    e.storage()
        .temporary()
        .set::<GovernorDataKey, ProposalData>(&key, &proposal_data);
    e.storage()
        .temporary()
        .extend_ttl(&key, LEDGER_BUMP, LEDGER_BUMP);
}

/***** Open Proposal *****/

/// Check if an open proposal exists created by `address`
pub fn has_open_proposal(e: &Env, address: &Address) -> bool {
    let key = GovernorDataKey::Open(address.clone());
    e.storage().temporary().has(&key)
}

/// Create the open proposal flag for `address` and bump it for the life of the proposal.
///
/// ### Arguments
/// * `address` - The address of the creator
pub fn create_open_proposal(e: &Env, address: &Address) {
    let key = GovernorDataKey::Open(address.clone());
    e.storage().temporary().set(&key, &true);
    e.storage()
        .temporary()
        .extend_ttl(&key, LEDGER_BUMP, LEDGER_BUMP);
}

/// Remove the open proposal flag for `address`
///
/// ### Arguments
/// * `address` - The address of the creator
pub fn del_open_proposal(e: &Env, address: &Address) {
    let key = GovernorDataKey::Open(address.clone());
    e.storage().temporary().remove(&key);
}

/***** Voter Support *****/

/// Get the voter support of `voter` for proposal at `proposal_id`
///
/// ### Arguments
/// * `voter` - The address of the voter
/// * `proposal_id` - The proposal id
pub fn get_voter_support(e: &Env, voter: &Address, proposal_id: u32) -> Option<u32> {
    let key = GovernorDataKey::VoterSup(VoterStatusKey {
        voter: voter.clone(),
        proposal_id,
    });
    e.storage().temporary().get::<GovernorDataKey, u32>(&key)
}

/// Create the voter support of `voter` for proposal at `proposal_id` to `support` and
/// bump it for the life of the proposal.
///
/// ### Arguments
/// * `voter` - The address of the voter
/// * `proposal_id` - The proposal id
/// * `support` - The support the user chose
///                 - 0 to vote against
///                 - 1 to vote for
///                 - 2 to vote abstain
pub fn create_voter_support(e: &Env, voter: &Address, proposal_id: u32, support: u32) {
    let key = GovernorDataKey::VoterSup(VoterStatusKey {
        voter: voter.clone(),
        proposal_id,
    });
    e.storage()
        .temporary()
        .set::<GovernorDataKey, u32>(&key, &support);
    e.storage()
        .temporary()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

/***** Proposal Votes *****/

/// Get the vote count of proposal at `proposal_id`
///
/// ### Arguments
/// * `proposal_id` - The proposal id
pub fn get_proposal_vote_count(e: &Env, proposal_id: u32) -> Option<VoteCount> {
    let key = GovernorDataKey::Votes(proposal_id);
    e.storage()
        .temporary()
        .get::<GovernorDataKey, VoteCount>(&key)
}

/// Set the vote count of proposal at `proposal_id`
///
/// Does not perform a ledger ttl bump.
///
/// ### Arguments
/// * `proposal_id` - The proposal id
/// * `count` - The vote count to store
pub fn set_proposal_vote_count(e: &Env, proposal_id: u32, count: &VoteCount) {
    let key = GovernorDataKey::Votes(proposal_id);
    e.storage()
        .temporary()
        .set::<GovernorDataKey, VoteCount>(&key, count);
}

/// Create the vote count of proposal at `proposal_id` to a new
/// `VoteCount` of 0 and bump it for the life of the proposal.
///
/// ### Arguments
/// * `proposal_id` - The proposal id
pub fn create_proposal_vote_count(e: &Env, proposal_id: u32) {
    let key = GovernorDataKey::Votes(proposal_id);
    e.storage().temporary().set::<GovernorDataKey, VoteCount>(
        &key,
        &VoteCount {
            against: 0,
            _for: 0,
            abstain: 0,
        },
    );
    e.storage()
        .temporary()
        .extend_ttl(&key, LEDGER_BUMP, LEDGER_BUMP);
}
