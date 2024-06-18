/// One day assuming 5s a ledger
pub(crate) const ONE_DAY_LEDGERS: u32 = 17280;
/// One hour assuming 5s a ledger
pub(crate) const ONE_HOUR_LEDGERS: u32 = 720;
/// 1 in basis points
pub(crate) const BPS_SCALAR: u32 = 10_000;

/// The maximum number of ledgers a proposal can exist for (31 days)
pub(crate) const MAX_PROPOSAL_LIFETIME: u32 = 31 * ONE_DAY_LEDGERS;
/// The maximum number of ledgers a proposal can be voted on for (7 days)
pub(crate) const MAX_VOTE_PERIOD: u32 = 7 * ONE_DAY_LEDGERS;
/// The minimum number of ledgers a proposal can be voted on for
pub(crate) const MIN_VOTE_PERIOD: u32 = ONE_HOUR_LEDGERS;
/// The maximum number of ledgers a proposal has between state changes before expiration
pub(crate) const MAX_GRACE_PERIOD: u32 = 7 * ONE_DAY_LEDGERS;
/// The minimum number of ledgers a proposal has between state changes before expiration
pub(crate) const MIN_GRACE_PERIOD: u32 = ONE_DAY_LEDGERS;
/// The minimum number of tokens required to create a proposal
pub(crate) const MIN_VOTE_THRESHOLD: i128 = 1;
