use soroban_sdk::{symbol_short, unwrap::UnwrapOptimized, Address, Env, Symbol};

const ONE_DAY_LEDGERS: u32 = 17280; // assumes 5 seconds per ledger on average
const LEDGER_THRESHOLD_SHARED: u32 = 14 * ONE_DAY_LEDGERS;
const LEDGER_BUMP_SHARED: u32 = 15 * ONE_DAY_LEDGERS;

//********** Storage Keys **********//

const IS_INIT_KEY: Symbol = symbol_short!("init");
const TOKEN_KEY: Symbol = symbol_short!("token");
const GOV_KEY: Symbol = symbol_short!("gov");

//********** Storage Utils **********//

/// Bump the instance lifetime by the defined amount
pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
}

/********** Instance **********/

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

pub fn set_token(e: &Env, token: &Address) {
    e.storage()
        .instance()
        .set::<Symbol, Address>(&TOKEN_KEY, &token);
}

pub fn get_token(e: &Env) -> Address {
    e.storage()
        .instance()
        .get::<Symbol, Address>(&TOKEN_KEY)
        .unwrap_optimized()
}

pub fn set_governor(e: &Env, governor: &Address) {
    e.storage()
        .instance()
        .set::<Symbol, Address>(&GOV_KEY, &governor);
}

pub fn get_governor(e: &Env) -> Address {
    e.storage()
        .instance()
        .get::<Symbol, Address>(&GOV_KEY)
        .unwrap_optimized()
}
