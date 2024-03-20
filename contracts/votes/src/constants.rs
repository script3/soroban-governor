pub(crate) const ONE_DAY_LEDGERS: u32 = 17280; // assumes 5s a ledger
pub(crate) const MAX_CHECKPOINT_AGE_LEDGERS: u32 = 8 * ONE_DAY_LEDGERS;

#[cfg(feature = "emissions")]
pub(crate) const SCALAR_7: i128 = 1_0000000; // 21 days
