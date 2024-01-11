use soroban_sdk::contracttype;

/// The governor settings for managing proposals
#[derive(Clone)]
#[contracttype]
pub struct GovernorSettings {
    /// The votes required to create a proposal.
    proposal_threshold: i128,
    /// The delay (in seconds) from the proposal creation to when the voting period begins. The voting
    /// period start time will be the checkpoint used to account for all votes for the proposal.
    vote_delay: u64,
    /// The time (in seconds) the proposal will be open to vote against.
    vote_period: u64,
    /// The time (in seconds) the proposal will have to wait between vote period closing and execution.
    timelock: u64,
    /// The percentage of votes (expressed in BPS) needed of the total available votes to consider a vote successful.
    quorum: u32,
    /// Determine which votes to count against the quorum out of for, against, and abstain. The value is encoded
    /// such that only the last 3 bits are considered, and follows the structure `MSB...{for}{against}{abstain}`,
    /// such that any value != 0 means that type of vote is counted in the quorum. For example, consider
    /// 5 == `0x0...0101`, this means that votes "for" and "abstain" are included in the quorum, but votes
    /// "against" are not.
    counting_type: u32,
    /// The percentage of votes "yes" (expressed in BPS) needed to consider a vote successful.
    vote_threshold: u32,
}
