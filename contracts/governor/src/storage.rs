use soroban_sdk::{
    contracttype, panic_with_error, unwrap::UnwrapOptimized, Address, Env, IntoVal, String, Symbol, TryFromVal, Val, Vec
};

use crate::errors::GovernorError;
/// The governor settings for managing proposals
#[derive(Clone)]
#[contracttype]
pub struct GovernorSettings {
    /// The votes required to create a proposal.
    pub proposal_threshold: i128,
    /// The delay (in seconds) from the proposal creation to when the voting period begins. The voting
    /// period start time will be the checkpoint used to account for all votes for the proposal.
    pub vote_delay: u64,
    /// The time (in seconds) the proposal will be open to vote against.
    pub vote_period: u64,
    /// The time (in seconds) the proposal will have to wait between vote period closing and execution.
    pub timelock: u64,
    /// The percentage of votes (expressed in BPS) needed of the total available votes to consider a vote successful.
    pub quorum: u32,
    /// Determine which votes to count against the quorum out of for, against, and abstain. The value is encoded
    /// such that only the last 3 bits are considered, and follows the structure `MSB...{for}{against}{abstain}`,
    /// such that any value != 0 means that type of vote is counted in the quorum. For example, consider
    /// 5 == `0x0...0101`, this means that votes "for" and "abstain" are included in the quorum, but votes
    /// "against" are not.
    pub counting_type: u32,
    /// The percentage of votes "yes" (expressed in BPS) needed to consider a vote successful.
    pub vote_threshold: u32,
}
/// Object for storing call data
#[derive(Clone)]
#[contracttype]
pub struct CallData {
    pub contract_address: Address,
    pub function: String,
    pub args: Vec<Val>,
}
/// Object for storing Pre-auth call data
#[derive(Clone)]
#[contracttype]
pub struct SubCallData {
    pub contract_address: Address,
    pub function: String,
    pub args: Vec<Val>,
    pub sub_auth: Vec<SubCallData>,
}

/// Object for storing proposals
#[derive(Clone)]
#[contracttype]
pub struct Proposal {
    pub title: String,
    pub description: String,
    pub proposer: Address,
    pub calldata: CallData,
    pub sub_calldata: Vec<SubCallData>,
    pub vote_start: u64,
    pub vote_end: u64,
}

const VOTER_TOKEN_ADDRESS_KEY: &str = "Votes";
const SETTINGS_KEY: &str = "Settings";
const IS_INIT_KEY: &str = "IsInit";
const PROPOSAL_ID_KEY: &str = "ProposalId";
pub(crate) const LEDGER_THRESHOLD_SHARED: u32 = 172800; // ~ 10 days
pub(crate) const LEDGER_BUMP_SHARED: u32 = 241920; // ~ 14 days
pub(crate) const MAX_VOTE_PERIOD: u64 = 1814400; // ~ 21 days represented in seconds

#[derive(Clone)]
#[contracttype]
pub enum GovernorDataKey {
    // A map of underlying asset's contract address to reserve config
    Proposals(u32),
    ProposalStatus(u32)
}
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
#[contracttype]
pub enum ProposalStatus {
    Pending = 0,
    Active = 1,
    Defeated = 2,
    Succeeded = 3,
    Queued = 4,
    Expired = 5,
    Executed = 6,
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
/// Fetch an entry in persistent storage that has a default value if it doesn't exist
fn get_temporary_default<K: IntoVal<Env, Val>, V: TryFromVal<Env, Val>>(
    e: &Env,
    key: &K,
    default: V,
    bump_threshold: u32,
    bump_amount: u32,
) -> V {
    if let Some(result) = e.storage().temporary().get::<K, V>(key) {
        e.storage()
            .temporary()
            .extend_ttl(key, bump_threshold, bump_amount);
        result
    } else {
        default
    }
}
/********** Init **********/

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

pub fn set_voter_token_address(e: &Env, voter: Address) {
    e.storage()
        .instance()
        .set::<Symbol, Address>(&Symbol::new(&e, VOTER_TOKEN_ADDRESS_KEY), &voter);
}

pub fn set_settings(e: &Env, settings: GovernorSettings) {
    e.storage()
        .instance()
        .set::<Symbol, GovernorSettings>(&Symbol::new(&e, SETTINGS_KEY), &settings);
}

pub fn get_voter_token_address(e: &Env) -> Address {
    e.storage()
        .instance()
        .get::<Symbol, Address>(&Symbol::new(&e, VOTER_TOKEN_ADDRESS_KEY))
        .unwrap_optimized()
}

pub fn get_settings(e: &Env) -> GovernorSettings {
    e.storage()
        .instance()
        .get::<Symbol, GovernorSettings>(&Symbol::new(&e, SETTINGS_KEY))
        .unwrap_optimized()
}

/********** Proposal **********/
pub fn set_proposal_id(e: &Env, proposal_id: u32) {
    let key = Symbol::new(&e, PROPOSAL_ID_KEY);
    e.storage()
        .persistent()
        .set::<Symbol, u32>(&key, &proposal_id);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
}

pub fn get_proposal_id(e: &Env) -> u32 {
    let key = Symbol::new(&e, PROPOSAL_ID_KEY);
    get_persistent_default::<Symbol, u32>(&e, &key, 0_u32, LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED)
}

pub fn get_proposal(e: &Env, proposal_id: u32) -> Option<Proposal> {
    let key = GovernorDataKey::Proposals(proposal_id);
    if let Some(result) = e.storage().temporary().get::<GovernorDataKey, Proposal>(&key) {
        e.storage()
            .temporary()
            .extend_ttl(&key, LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
        Some(result)
    } else {
        None
    }
}

pub fn set_proposal(e: &Env, id: u32, proposal: Proposal) {
    let key = GovernorDataKey::Proposals(id);
    e.storage()
        .temporary()
        .set::<GovernorDataKey, Proposal>(&key, &proposal);
    e.storage()
        .temporary()
        .extend_ttl(&key, LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
} 

pub fn get_proposal_status(e: &Env, proposal_id: u32) -> Option<ProposalStatus> {
    let key = GovernorDataKey::ProposalStatus(proposal_id);
    get_temporary_default::<GovernorDataKey, Option<ProposalStatus>>(&e, &key, None, LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED)
}

pub fn set_proposal_status(e: &Env, id: u32, proposal_status: ProposalStatus) {
    let key = GovernorDataKey::ProposalStatus(id);
    e.storage()
        .temporary()
        .set::<GovernorDataKey, ProposalStatus>(&key, &proposal_status);
    e.storage()
        .temporary()
        .extend_ttl(&key, LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
} 


