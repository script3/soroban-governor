use soroban_sdk::contracterror;

/// The error codes for the contract.
#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GovernorError {
    // Default errors to align with built-in contract
    InternalError = 1,
    InvalidSettingsError = 2,
    AlreadyInitializedError = 3,

    UnauthorizedError = 4,

    NegativeAmountError = 8,
    AllowanceError = 9,
    BalanceError = 10,
    OverflowError = 12,

    NonExistentProposalError = 13,
    ProposalNotActiveError = 14,
    InvalidProposalSupportError = 15,
    VotePeriodNotFinishedError = 16,
    ProposalNotQueuedError = 17,
    TimelockNotMetError = 18,
    CancelActiveProposalError = 19,
}
