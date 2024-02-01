use soroban_sdk::{contracttype, Address, Env, IntoVal, Symbol, TryFromVal, Val};

pub(crate) const LEDGER_THRESHOLD_SHARED: u32 = 172800; // ~ 10 days
pub(crate) const LEDGER_BUMP_SHARED: u32 = 241920; // ~ 14 days

#[contracttype]
pub struct PastUserVotesKey {
    user: Address,
    timestamp: u64,
}
#[contracttype]
pub enum MockVotesDataKey {
    UserVotes(Address),
    PastUserVotes(PastUserVotesKey),
}

/// Fetch an entry in persistent storage that has a default value if it doesn't exist
fn get_persistent_default<K: IntoVal<Env, Val>, V: TryFromVal<Env, Val>>(
    e: &Env,
    key: &K,
    default: V,
    bump_threshold: u32,
    bump_amount: u32,
) -> V {
    if let Some(result) = e.storage().persistent().get::<K, V>(key) {
        e.storage()
            .persistent()
            .extend_ttl(key, bump_threshold, bump_amount);
        result
    } else {
        default
    }
}

/// Get the voter power of `user`
///
/// ### Arguments
/// * `user` - The address of the user
pub fn get_votes(e: &Env, user: &Address) -> i128 {
    let key = MockVotesDataKey::UserVotes(user.clone());
    get_persistent_default::<MockVotesDataKey, i128>(
        &e,
        &key,
        0_i128,
        LEDGER_THRESHOLD_SHARED,
        LEDGER_BUMP_SHARED,
    )
}

/// Set the voter power of `user` to `amount`
///
/// ### Arguments
/// * `user` - The address of the user
/// * `amount` - The voter power
pub fn set_votes(e: &Env, user: &Address, amount: &i128) {
    let key = MockVotesDataKey::UserVotes(user.clone());
    e.storage()
        .persistent()
        .set::<MockVotesDataKey, i128>(&key, amount);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
}

/// Get the past voter power of `user` at `timestamp`
///
/// ### Arguments
/// * `user` - The address of the user
/// * `timestamp` - The timestamp number
pub fn get_past_votes(e: &Env, user: &Address, timestamp: &u64) -> i128 {
    let key = MockVotesDataKey::PastUserVotes(PastUserVotesKey {
        user: user.clone(),
        timestamp: *timestamp,
    });
    get_persistent_default::<MockVotesDataKey, i128>(
        &e,
        &key,
        0_i128,
        LEDGER_THRESHOLD_SHARED,
        LEDGER_BUMP_SHARED,
    )
}

/// Set the voter power of `user` at `timestamp` to `amount`
///
/// ### Arguments
/// * `user` - The address of the user
/// * `timestamp` - The timestamp number
/// * `amount` - The voter power
pub fn set_past_votes(e: &Env, user: &Address, timestamp: &u64, amount: &i128) {
    let key = MockVotesDataKey::PastUserVotes(PastUserVotesKey {
        user: user.clone(),
        timestamp: *timestamp,
    });
    e.storage()
        .persistent()
        .set::<MockVotesDataKey, i128>(&key, &amount);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
}

/// Set total supply of vote tokens
///
/// ### Arguments
/// * `amount` - The voter power
pub fn set_total_supply(e: &Env, amount: &i128) {
    let key = Symbol::new(e, "total_supply");

    e.storage().persistent().set::<Symbol, i128>(&key, &amount);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
}

/// Get total supply of vote tokens
pub fn get_total_supply(e: &Env) -> i128 {
    let key = Symbol::new(e, "total_supply");
    get_persistent_default::<Symbol, i128>(
        e,
        &key,
        0_i128,
        LEDGER_THRESHOLD_SHARED,
        LEDGER_BUMP_SHARED,
    )
}
