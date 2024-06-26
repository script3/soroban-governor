use soroban_sdk::{contracttype, Address, BytesN, String, Symbol, Val, Vec};

/// The governor settings for managing proposals
#[derive(Clone)]
#[contracttype]
pub struct GovernorSettings {
    /// The votes required to create a proposal.
    pub proposal_threshold: i128,
    /// The delay (in ledgers) from the proposal creation to when the voting period begins. The voting
    /// period start time will be the checkpoint used to account for all votes for the proposal.
    pub vote_delay: u32,
    /// The time (in ledgers) the proposal will be open to vote against.
    pub vote_period: u32,
    /// The time (in ledgers) the proposal will have to wait between vote period closing and execution.
    pub timelock: u32,
    /// The time (in ledgers) the proposal has to be executed before it expires. This starts after the timelock.
    pub grace_period: u32,
    /// The percentage of votes (expressed in BPS) needed of the total available votes to consider a vote successful.
    pub quorum: u32,
    /// Determine which votes to count against the quorum out of for, against, and abstain. The value is encoded
    /// such that only the last 3 bits are considered, and follows the structure `MSB...{against}{for}{abstain}`,
    /// such that any value != 0 means that type of vote is counted in the quorum. For example, consider
    /// 5 == `0x0...0101`, this means that votes "against" and "abstain" are included in the quorum, but votes
    /// "for" are not.
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
    pub auths: Vec<Calldata>,
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
    pub action: ProposalAction,
}

/// The action to be taken by a proposal.
///
/// ### Calldata
/// The proposal will execute the calldata from the governor contract on execute.
///
/// ### Upgrade
/// The proposal will upgrade the governor contract to the new WASM hash on execute.
///
/// ### Settings
/// The proposal will update the governor settings on execute.
///
/// ### Council
/// The proposal will update the council address on execute.
///
/// ### Snapshot
/// There is no action to be taken by the proposal.
#[derive(Clone)]
#[contracttype]
pub enum ProposalAction {
    Calldata(Calldata),
    Upgrade(BytesN<32>),
    Settings(GovernorSettings),
    Council(Address),
    Snapshot,
}

/// The data for a proposal
#[derive(Clone)]
#[contracttype]
pub struct ProposalData {
    /// The address of the account creating the proposal
    pub creator: Address,
    /// The ledger sequence when the voting period begins
    pub vote_start: u32,
    /// The ledger sequence when the voting period ends
    pub vote_end: u32,
    /// The ledger sequence when the proposal will be executed, or zero if no execution has been scheduled
    pub eta: u32,
    /// The status of the proposal
    pub status: ProposalStatus,
    /// Whether the proposal is executable
    pub executable: bool,
}

/// The types of votes that can be cast
#[repr(u8)]
pub enum VoteType {
    Against = 0,
    For = 1,
    Abstain = 2,
}

// Stores proposal results
#[derive(Clone)]
#[contracttype]
pub struct VoteCount {
    pub against: i128,
    pub _for: i128,
    pub abstain: i128,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum ProposalStatus {
    /// The proposal exists and voting has not been closed
    Open = 0,
    /// The proposal was voted for. If the proposal is executable, the timelock begins once this state is reached.
    Successful = 1,
    /// The proposal was voted against
    Defeated = 2,
    /// The proposal did not reach quorum before the voting period ended, or was stalled out during the grace period.
    Expired = 3,
    /// The proposal has been executed
    Executed = 4,
    /// The proposal has been canceled
    Canceled = 5,
}
