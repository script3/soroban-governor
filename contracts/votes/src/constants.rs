pub(crate) const ONE_DAY_LEDGERS: u32 = 17280; // assumes 5s a ledger

/// The maximum number of ledgers a checkpoint needs to exist for. Once a checkpoint is written, that means
/// a voting period has already started, and the max voting period is 7 days worth of ledgers.
pub(crate) const MAX_CHECKPOINT_AGE_LEDGERS: u32 = 8 * ONE_DAY_LEDGERS;

/// The maximum number of ledgers a proposal can exist for.
pub(crate) const MAX_PROPOSAL_AGE_LEDGERS: u32 = 31 * ONE_DAY_LEDGERS;

#[cfg(feature = "staking")]
pub(crate) const SCALAR_7: i128 = 1_0000000;
