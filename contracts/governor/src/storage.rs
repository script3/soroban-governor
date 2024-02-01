use soroban_sdk::{
    contracttype, unwrap::UnwrapOptimized, Address, Env, IntoVal, String, Symbol, TryFromVal, Val,
    Vec,
};

const VOTER_TOKEN_ADDRESS_KEY: &str = "Votes";
const SETTINGS_KEY: &str = "Settings";
const IS_INIT_KEY: &str = "IsInit";
const PROPOSAL_ID_KEY: &str = "ProposalId";
pub(crate) const LEDGER_THRESHOLD_SHARED: u32 = 518400; // ~ 10 days
pub(crate) const LEDGER_BUMP_SHARED: u32 = 535680; // ~ 14 days
pub(crate) const MAX_VOTE_PERIOD: u64 = 1814400; // ~ 21 days represented in seconds

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
pub struct Calldata {
    pub contract_id: Address,
    pub function: Symbol,
    pub args: Vec<Val>,
}
/// Object for storing Pre-auth call data
#[derive(Clone)]
#[contracttype]
pub struct SubCalldata {
    pub contract_id: Address,
    pub function: Symbol,
    pub args: Vec<Val>,
    pub sub_auth: Vec<SubCalldata>,
}

/// Object for storing proposals
#[derive(Clone)]
#[contracttype]
pub struct Proposal {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub proposer: Address,
    pub calldata: Calldata,
    pub sub_calldata: Vec<SubCalldata>,
    pub vote_start: u64,
    pub vote_end: u64,
}

// Key for storing Voter's decision
#[derive(Clone)]
#[contracttype]
pub struct VoterStatusKey {
    pub proposal_id: u32,
    pub voter: Address,
}

// Stores proposal results
#[derive(Clone)]
#[contracttype]
pub struct VoteCount {
    pub votes_for: i128,
    pub votes_against: i128,
    pub votes_abstained: i128,
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

/********** Proposal **********/
/// Set the next proposal id
///
/// ### Arguments
/// * `proposal_id` - The new proposal_id
pub fn set_proposal_id(e: &Env, proposal_id: &u32) {
    let key = Symbol::new(&e, PROPOSAL_ID_KEY);
    e.storage()
        .persistent()
        .set::<Symbol, u32>(&key, &proposal_id);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
}

/// Get the current proposal id
pub fn get_proposal_id(e: &Env) -> u32 {
    let key = Symbol::new(&e, PROPOSAL_ID_KEY);
    get_persistent_default::<Symbol, u32>(
        &e,
        &key,
        0_u32,
        LEDGER_THRESHOLD_SHARED,
        LEDGER_BUMP_SHARED,
    )
}

/// Fetch proposal at `proposal_id`
///
/// ### Arguments
/// * `proposal_id` - The id of the proposal to fetch
pub fn get_proposal(e: &Env, proposal_id: &u32) -> Option<Proposal> {
    let key = GovernorDataKey::Proposal(*proposal_id);
    get_temporary_default(&e, &key, None, LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED)
}

/// Store proposal in vec of proposals
///
/// ### Arguments
/// * `proposal` - The proposal to store
pub fn set_proposal(e: &Env, proposal_id: &u32, proposal: &Proposal) {
    let key = GovernorDataKey::Proposal(*proposal_id);
    e.storage()
        .temporary()
        .set::<GovernorDataKey, Proposal>(&key, proposal);
    e.storage()
        .temporary()
        .extend_ttl(&key, LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
}

// Get the proposal status for proposal at `proposal_id`
///
/// ### Arguments
/// * `proposal_id` - The proposal status id
pub fn get_proposal_status(e: &Env, proposal_id: &u32) -> ProposalStatus {
    let key = GovernorDataKey::ProposalStatus(*proposal_id);
    get_temporary_default::<GovernorDataKey, ProposalStatus>(
        &e,
        &key,
        ProposalStatus::Pending,
        LEDGER_THRESHOLD_SHARED,
        LEDGER_BUMP_SHARED,
    )
}

/// Set the proposal status for proposal at `proposal_id`
///
/// ### Arguments
/// * `proposal_id` - The proposal id
pub fn set_proposal_status(e: &Env, id: &u32, proposal_status: &ProposalStatus) {
    let key = GovernorDataKey::ProposalStatus(*id);
    e.storage()
        .temporary()
        .set::<GovernorDataKey, ProposalStatus>(&key, &proposal_status);
    e.storage()
        .temporary()
        .extend_ttl(&key, LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
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
        .extend_ttl(&key, LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
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
    get_temporary_default(&e, &key, None, LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED)
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
        .extend_ttl(&key, LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
}

/// Get the vote count of proposal at `proposal_id`
///
/// ### Arguments
/// * `proposal_id` - The proposal id
pub fn get_proposal_vote_count(e: &Env, proposal_id: &u32) -> VoteCount {
    let key = GovernorDataKey::ProposalVotes(*proposal_id);
    get_temporary_default::<GovernorDataKey, VoteCount>(
        &e,
        &key,
        VoteCount {
            votes_for: 0,
            votes_against: 0,
            votes_abstained: 0,
        },
        LEDGER_THRESHOLD_SHARED,
        LEDGER_BUMP_SHARED,
    )
}
