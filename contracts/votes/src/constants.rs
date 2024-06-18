pub(crate) const ONE_DAY_LEDGERS: u32 = 17280; // assumes 5s a ledger

/// The maximum number of ledgers a checkpoint needs to exist for. Once a checkpoint is written, that means
/// a voting period has already started. The longest a checkpoint will be needed is the max `vote_period` and
/// `grace_period` combined, which is 14 days. We add 1 day for buffer.
///
/// See: https://github.com/script3/soroban-governor/blob/main/contracts/governor/src/constants.rs
pub(crate) const MAX_CHECKPOINT_AGE_LEDGERS: u32 = 15 * ONE_DAY_LEDGERS;

/// The maximum number of ledgers a proposal can exist for.
pub(crate) const MAX_PROPOSAL_AGE_LEDGERS: u32 = 31 * ONE_DAY_LEDGERS;

#[cfg(feature = "bonding")]
pub(crate) const SCALAR_7: i128 = 1_0000000;
