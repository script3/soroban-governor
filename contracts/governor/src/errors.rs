use soroban_sdk::contracterror;

/// The error codes for the contract.
#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GovernorError {
    // Default errors to align with built-in contract
    InternalError = 1,
    AlreadyInitializedError = 3,

    UnauthorizedError = 4,

    NegativeAmountError = 8,
    AllowanceError = 9,
    BalanceError = 10,
    OverflowError = 12,

    InvalidSettingsError = 200,
    NonExistentProposalError = 201,
    ProposalNotActiveError = 202,
    InvalidProposalSupportError = 203,
    VotePeriodNotFinishedError = 204,
    ProposalNotQueuedError = 205,
    TimelockNotMetError = 206,
    CancelActiveProposalError = 207,
    InsifficientVotingUnitsError = 208,
}
