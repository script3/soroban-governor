use soroban_sdk::{
    contracttype, unwrap::UnwrapOptimized, Address, Env, IntoVal, Symbol, TryFromVal, Val,
};

use crate::types::{GovernorSettings, ProposalConfig, ProposalData, VoteCount};

const VOTER_TOKEN_ADDRESS_KEY: &str = "Votes";
const SETTINGS_KEY: &str = "Settings";
const IS_INIT_KEY: &str = "IsInit";
const PROPOSAL_ID_KEY: &str = "ProposalId";

const ONE_DAY_LEDGERS: u32 = 17280; // assumes 5 seconds per ledger on average
const LEDGER_THRESHOLD_SHARED: u32 = 14 * ONE_DAY_LEDGERS;
const LEDGER_BUMP_SHARED: u32 = 15 * ONE_DAY_LEDGERS;
const LEDGER_THRESHOLD_PROPOSAL: u32 = 30 * ONE_DAY_LEDGERS;
const LEDGER_BUMP_PROPOSAL: u32 = 31 * ONE_DAY_LEDGERS;

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
    // A map of proposal id to proposal
    Proposal(u32),
    // A map of underlying asset's contract address to reserve config
    ProposalStatus(u32),
    // The voter's decision
    VoterStatus(VoterStatusKey),
    // The proposal results
    ProposalVotes(u32),
}

//********** Storage Utils **********//

/// Bump the instance lifetime by the defined amount
pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
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

/// Set the next proposal id
///
/// ### Arguments
/// * `proposal_id` - The new proposal_id
pub fn set_next_proposal_id(e: &Env, proposal_id: &u32) {
    let key = Symbol::new(&e, PROPOSAL_ID_KEY);
    e.storage()
        .persistent()
        .set::<Symbol, u32>(&key, &proposal_id);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD_PROPOSAL, LEDGER_BUMP_PROPOSAL);
}

/// Get the current proposal id
pub fn get_next_proposal_id(e: &Env) -> u32 {
    let key = Symbol::new(&e, PROPOSAL_ID_KEY);
    get_persistent_default::<Symbol, u32>(
        &e,
        &key,
        0_u32,
        LEDGER_THRESHOLD_PROPOSAL,
        LEDGER_BUMP_PROPOSAL,
    )
}

/********** Temporary **********/

/// Fetch proposal config at `proposal_id`
///
/// ### Arguments
/// * `proposal_id` - The id of the proposal to fetch
pub fn get_proposal_config(e: &Env, proposal_id: &u32) -> Option<ProposalConfig> {
    let key = GovernorDataKey::Proposal(*proposal_id);
    e.storage()
        .temporary()
        .get::<GovernorDataKey, ProposalConfig>(&key)
}

/// Store proposal config at `proposal_id`
///
/// ### Arguments
/// * `proposal_id` - The proposal id
/// * `proposal_config` - The proposal config to store
pub fn set_proposal_config(e: &Env, proposal_id: &u32, proposal_data: &ProposalConfig) {
    let key = GovernorDataKey::Proposal(*proposal_id);
    e.storage()
        .temporary()
        .set::<GovernorDataKey, ProposalConfig>(&key, proposal_data);
    e.storage()
        .temporary()
        .extend_ttl(&key, LEDGER_BUMP_PROPOSAL, LEDGER_BUMP_PROPOSAL);
}

// Get the proposal status for proposal at `proposal_id`
///
/// ### Arguments
/// * `proposal_id` - The proposal status id
pub fn get_proposal_data(e: &Env, proposal_id: &u32) -> Option<ProposalData> {
    let key = GovernorDataKey::ProposalStatus(*proposal_id);
    e.storage()
        .temporary()
        .get::<GovernorDataKey, ProposalData>(&key)
}

/// Set the proposal status for proposal at `proposal_id`
///
/// ### Arguments
/// * `proposal_id` - The proposal id
pub fn set_proposal_data(e: &Env, id: &u32, proposal_status: &ProposalData) {
    let key = GovernorDataKey::ProposalStatus(*id);
    e.storage()
        .temporary()
        .set::<GovernorDataKey, ProposalData>(&key, &proposal_status);
    e.storage()
        .temporary()
        .extend_ttl(&key, LEDGER_BUMP_PROPOSAL, LEDGER_BUMP_PROPOSAL);
}

/********** Vote **********/

/// Set the voter status of `voter` for proposal at `proposal_id` to `status`
///
/// ### Arguments
/// * `voter` - The address of the voter
/// * `proposal_id` - The proposal id
/// * `status` - The status the user chose
///                 - 0 to vote abstain
///                 - 1 to vote against
///                 - 2 to vote for
pub fn set_voter_status(e: &Env, voter: &Address, proposal_id: &u32, status: &u32) {
    let key = GovernorDataKey::VoterStatus(VoterStatusKey {
        voter: voter.clone(),
        proposal_id: *proposal_id,
    });
    e.storage()
        .temporary()
        .set::<GovernorDataKey, u32>(&key, &status);
    e.storage()
        .temporary()
        .extend_ttl(&key, LEDGER_THRESHOLD_PROPOSAL, LEDGER_BUMP_PROPOSAL);
}

/// Get the voter status of `voter` for proposal at `proposal_id`
///
/// ### Arguments
/// * `voter` - The address of the voter
/// * `proposal_id` - The proposal id
pub fn get_voter_status(e: &Env, voter: &Address, proposal_id: &u32) -> Option<u32> {
    let key = GovernorDataKey::VoterStatus(VoterStatusKey {
        voter: voter.clone(),
        proposal_id: *proposal_id,
    });
    e.storage().temporary().get::<GovernorDataKey, u32>(&key)
}

/// Set the vote count of proposal at `proposal_id`
///
/// ### Arguments
/// * `proposal_id` - The proposal id
/// * `count` - The vote count to store
pub fn set_proposal_vote_count(e: &Env, proposal_id: &u32, count: &VoteCount) {
    let key = GovernorDataKey::ProposalVotes(*proposal_id);
    e.storage()
        .temporary()
        .set::<GovernorDataKey, VoteCount>(&key, count);
    e.storage()
        .temporary()
        .extend_ttl(&key, LEDGER_BUMP_PROPOSAL, LEDGER_BUMP_PROPOSAL);
}

/// Get the vote count of proposal at `proposal_id`
///
/// ### Arguments
/// * `proposal_id` - The proposal id
pub fn get_proposal_vote_count(e: &Env, proposal_id: &u32) -> VoteCount {
    let key = GovernorDataKey::ProposalVotes(*proposal_id);
    if let Some(result) = e
        .storage()
        .temporary()
        .get::<GovernorDataKey, VoteCount>(&key)
    {
        result
    } else {
        VoteCount {
            votes_for: 0,
            votes_against: 0,
            votes_abstained: 0,
        }
    }
}
