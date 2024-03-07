use soroban_sdk::{
    contracttype, symbol_short, unwrap::UnwrapOptimized, Address, Env, IntoVal, String, Symbol,
    TryFromVal, Val, Vec,
};

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 8 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub(crate) const MAX_VOTE_CHECKPOINT_LEDGERS: u32 = INSTANCE_BUMP_AMOUNT;

pub(crate) const BALANCE_BUMP_AMOUNT: u32 = 31 * DAY_IN_LEDGERS - 1;
pub(crate) const BALANCE_LIFETIME_THRESHOLD: u32 = BALANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

//********** Storage Keys **********//

const IS_INIT_KEY: Symbol = symbol_short!("IsInit");
const GOV_KEY: Symbol = symbol_short!("GOV");
const METADATA_KEY: Symbol = symbol_short!("METADATA");
const TOTAL_SUPPLY_KEY: Symbol = symbol_short!("SUPPLY");
const TOTAL_SUPPLY_CHECK_KEY: Symbol = symbol_short!("SPLYCHECK");
const VOTE_LEDGERS_KEY: Symbol = symbol_short!("VOTE_SEQ");

#[cfg(feature = "admin")]
const ADMIN_KEY: Symbol = symbol_short!("ADMIN");

#[cfg(feature = "wrapped")]
const TOKEN_KEY: Symbol = symbol_short!("TOKEN");

#[derive(Clone)]
#[contracttype]
pub struct AllowanceDataKey {
    pub from: Address,
    pub spender: Address,
}

#[contracttype]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Allowance(AllowanceDataKey),
    Balance(Address),
    Votes(Address),
    VotesCheck(Address),
    Delegate(Address),
}

//********** Storage Types **********//

#[derive(Clone)]
#[contracttype]
pub struct TokenMetadata {
    pub decimal: u32,
    pub name: String,
    pub symbol: String,
}

//********** Storage Utils **********//

/// Bump the instance lifetime by the defined amount
pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

/// Fetch an entry in persistent storage that has a default value if it doesn't exist
fn get_persistent_default<K: IntoVal<Env, Val>, V: TryFromVal<Env, Val>, F: FnOnce() -> V>(
    e: &Env,
    key: &K,
    default: F,
    bump_threshold: u32,
    bump_amount: u32,
) -> V {
    if let Some(result) = e.storage().persistent().get::<K, V>(key) {
        e.storage()
            .persistent()
            .extend_ttl(key, bump_threshold, bump_amount);
        result
    } else {
        default()
    }
}

/// Fetch an entry in temporary storage that has a default value if it doesn't exist
fn get_temporary_default<K: IntoVal<Env, Val>, V: TryFromVal<Env, Val>, F: FnOnce() -> V>(
    e: &Env,
    key: &K,
    default: F,
) -> V {
    if let Some(result) = e.storage().temporary().get::<K, V>(key) {
        result
    } else {
        default()
    }
}

//********** Instance **********//

// Initialization flag

/// Check if the contract has been initialized
pub fn get_is_init(e: &Env) -> bool {
    e.storage().instance().has(&IS_INIT_KEY)
}

/// Set the contract as initialized
pub fn set_is_init(e: &Env) {
    e.storage()
        .instance()
        .set::<Symbol, bool>(&IS_INIT_KEY, &true);
}

// Token

pub fn get_governor(e: &Env) -> Address {
    e.storage().instance().get(&GOV_KEY).unwrap_optimized()
}

pub fn set_governor(e: &Env, address: &Address) {
    e.storage().instance().set(&GOV_KEY, address);
}

// Metadata

pub fn get_metadata(e: &Env) -> TokenMetadata {
    e.storage().instance().get(&METADATA_KEY).unwrap_optimized()
}

pub fn set_metadata(e: &Env, metadata: &TokenMetadata) {
    e.storage().instance().set(&METADATA_KEY, metadata);
}

// --- Admin

#[cfg(feature = "admin")]
pub fn get_admin(e: &Env) -> Address {
    e.storage().instance().get(&ADMIN_KEY).unwrap_optimized()
}

#[cfg(feature = "admin")]
pub fn set_admin(e: &Env, address: &Address) {
    e.storage().instance().set(&ADMIN_KEY, address);
}

// --- Wrapped Token

#[cfg(feature = "wrapped")]
pub fn get_token(e: &Env) -> Address {
    e.storage().instance().get(&TOKEN_KEY).unwrap_optimized()
}

#[cfg(feature = "wrapped")]
pub fn set_token(e: &Env, address: &Address) {
    e.storage().instance().set(&TOKEN_KEY, address);
}

//********** Persistent **********//

// Total Supply

pub fn get_total_supply(e: &Env) -> u128 {
    get_persistent_default(
        e,
        &TOTAL_SUPPLY_KEY,
        || 0,
        BALANCE_LIFETIME_THRESHOLD,
        BALANCE_BUMP_AMOUNT,
    )
}

pub fn set_total_supply(e: &Env, checkpoint: &u128) {
    e.storage().persistent().set(&TOTAL_SUPPLY_KEY, checkpoint);
    e.storage().persistent().extend_ttl(
        &TOTAL_SUPPLY_KEY,
        BALANCE_LIFETIME_THRESHOLD,
        BALANCE_BUMP_AMOUNT,
    );
}

// Balance

pub fn get_balance(e: &Env, address: &Address) -> i128 {
    get_persistent_default(
        e,
        &DataKey::Balance(address.clone()),
        || 0_i128,
        BALANCE_LIFETIME_THRESHOLD,
        BALANCE_BUMP_AMOUNT,
    )
}

pub fn set_balance(e: &Env, address: &Address, balance: &i128) {
    let key = DataKey::Balance(address.clone());
    e.storage().persistent().set(&key, balance);
    e.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

// Vote Units

pub fn get_voting_units(e: &Env, address: &Address) -> u128 {
    get_persistent_default(
        e,
        &DataKey::Votes(address.clone()),
        || 0,
        BALANCE_LIFETIME_THRESHOLD,
        BALANCE_BUMP_AMOUNT,
    )
}

pub fn set_voting_units(e: &Env, address: &Address, checkpoint: &u128) {
    let key = DataKey::Votes(address.clone());
    e.storage().persistent().set(&key, checkpoint);
    e.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

// Delegate

pub fn get_delegate(e: &Env, address: &Address) -> Address {
    get_persistent_default(
        e,
        &DataKey::Delegate(address.clone()),
        || address.clone(),
        BALANCE_LIFETIME_THRESHOLD,
        BALANCE_BUMP_AMOUNT,
    )
}

pub fn set_delegate(e: &Env, address: &Address, delegatee: &Address) {
    let key = DataKey::Delegate(address.clone());
    e.storage().persistent().set(&key, delegatee);
    e.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

//********** Temporary **********//

// Allowance

pub fn get_allowance(e: &Env, from: &Address, spender: &Address) -> AllowanceValue {
    let key = DataKey::Allowance(AllowanceDataKey {
        from: from.clone(),
        spender: spender.clone(),
    });
    let temp = e.storage().temporary().get(&key);
    temp.unwrap_or_else(|| AllowanceValue {
        amount: 0,
        expiration_ledger: 0,
    })
}

pub fn set_allowance(
    e: &Env,
    from: &Address,
    spender: &Address,
    amount: i128,
    expiration_ledger: u32,
) {
    let key = DataKey::Allowance(AllowanceDataKey {
        from: from.clone(),
        spender: spender.clone(),
    });
    let allowance = AllowanceValue {
        amount,
        expiration_ledger,
    };
    e.storage().temporary().set(&key, &allowance);
    if amount > 0 {
        let ledgers_to_live = expiration_ledger
            .checked_sub(e.ledger().sequence())
            .unwrap_optimized();
        e.storage()
            .temporary()
            .extend_ttl(&key, ledgers_to_live, ledgers_to_live);
    }
}

// Vote Units Checkpoints

pub fn get_vote_ledgers(e: &Env) -> Vec<u32> {
    get_temporary_default(e, &VOTE_LEDGERS_KEY, || Vec::new(&e))
}

pub fn set_vote_ledgers(e: &Env, vote_ledgers: &Vec<u32>) {
    e.storage().temporary().set(&VOTE_LEDGERS_KEY, vote_ledgers);
    // extend for twice the time a vote period can last
    e.storage().temporary().extend_ttl(
        &VOTE_LEDGERS_KEY,
        MAX_VOTE_CHECKPOINT_LEDGERS * 2,
        MAX_VOTE_CHECKPOINT_LEDGERS * 2,
    );
}

// Total Supply Checkpoints

pub fn get_total_supply_checkpoints(e: &Env) -> Vec<u128> {
    get_temporary_default(e, &TOTAL_SUPPLY_CHECK_KEY, || Vec::new(&e))
}

pub fn set_total_supply_checkpoints(e: &Env, balance: &Vec<u128>) {
    e.storage()
        .temporary()
        .set(&TOTAL_SUPPLY_CHECK_KEY, balance);
    // Checkpoints only need to exist for at least 7 days to ensure that correct
    // vote periods can be tracked for the entire max voting period of 7 days.
    // TTL is 8 days of ledgers, providing some wiggle room for fast ledgers.
    e.storage().temporary().extend_ttl(
        &TOTAL_SUPPLY_CHECK_KEY,
        MAX_VOTE_CHECKPOINT_LEDGERS,
        MAX_VOTE_CHECKPOINT_LEDGERS,
    );
}

// Vote Units Checkpoints

pub fn get_voting_units_checkpoints(e: &Env, address: &Address) -> Vec<u128> {
    get_temporary_default(e, &DataKey::VotesCheck(address.clone()), || Vec::new(&e))
}

pub fn set_voting_units_checkpoints(e: &Env, address: &Address, balance: &Vec<u128>) {
    let key = DataKey::VotesCheck(address.clone());
    e.storage().temporary().set(&key, balance);
    // Checkpoints only need to exist for at least 7 days to ensure that correct
    // vote periods can be tracked for the entire max voting period of 7 days.
    // Instance bump amount is 8 days, providing some wiggle room for fast ledgers.
    e.storage().temporary().extend_ttl(
        &key,
        MAX_VOTE_CHECKPOINT_LEDGERS,
        MAX_VOTE_CHECKPOINT_LEDGERS,
    );
}
