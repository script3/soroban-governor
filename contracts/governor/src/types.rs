use soroban_sdk::{contracttype, Address, String, Symbol, Val, Vec};

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

/// The proposal object
#[derive(Clone)]
#[contracttype]
pub struct Proposal {
    pub id: u32,
    pub config: ProposalConfig,
    pub data: ProposalData,
}

/// The configuration for a proposal. Set by the proposal creator.
#[derive(Clone)]
#[contracttype]
pub struct ProposalConfig {
    pub title: String,
    pub description: String,
    pub proposer: Address,
    pub calldata: Calldata,
    pub sub_calldata: Vec<SubCalldata>,
}

/// The data for a proposal
#[derive(Clone)]
#[contracttype]
pub struct ProposalData {
    pub vote_start: u64,
    pub vote_end: u64,
    pub status: ProposalStatus,
}

// Stores proposal results
#[derive(Clone)]
#[contracttype]
pub struct VoteCount {
    pub votes_for: i128,
    pub votes_against: i128,
    pub votes_abstained: i128,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
#[contracttype]
pub enum ProposalStatus {
    Pending = 0,
    Active = 1,
    Defeated = 2,
    Queued = 3,
    Expired = 4,
    Executed = 5,
    Canceled = 6,
}
