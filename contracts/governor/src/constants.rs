pub(crate) const ONE_DAY_LEDGERS: u32 = 17280; // assumes 5s a ledger
pub(crate) const MAX_PROPOSAL_LIFETIME: u32 = 31 * ONE_DAY_LEDGERS; // 31 days
pub(crate) const MAX_VOTE_PERIOD: u32 = 7 * ONE_DAY_LEDGERS; // 7 days
pub(crate) const BPS_SCALAR: i128 = 10_000;
