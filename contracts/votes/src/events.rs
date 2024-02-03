use soroban_sdk::{symbol_short, Address, Env, Symbol};

pub struct VoterTokenEvents {}

impl VoterTokenEvents {
    /// Emitted when a voter delegates their votes to an address
    ///
    /// - topics - `["delegate", delegator: Address, delegatee: Address]`
    /// - data - `[old_delegatee: Address]`
    pub fn delegate(env: &Env, delegator: Address, delegatee: Address, old_delegatee: Address) {
        let topics = (symbol_short!("delegate"), delegator, delegatee);
        env.events().publish(topics, old_delegatee);
    }

    /// Emitted when a delagate's votes are changed
    ///
    /// This event is emitted for the delegated account's votes if a transfer, deposit, or withdraw occurs
    ///
    /// - topics - `["votes_changed", delegate: Address]`
    /// - data - `[old_votes: i128, new_votes: i128]`
    pub fn votes_changed(e: &Env, delegate: Address, old_votes: i128, new_votes: i128) {
        let topics = (Symbol::new(e, "votes_changed"), delegate);
        e.events().publish(topics, (old_votes, new_votes));
    }

    /// Emitted when an account deposits tokens into the votes contract
    ///
    /// - topics - `["deposit", account: Address]`
    /// - data - `[amount: i128]`
    pub fn deposit(e: &Env, account: Address, amount: i128) {
        let topics = (Symbol::new(e, "deposit"), account);
        e.events().publish(topics, amount);
    }

    /// Emitted when an account withdraws tokens from the votes contract
    ///
    /// - topics - `["withdraw", account: Address]`
    /// - data - `[amount: i128]`
    pub fn withdraw(e: &Env, account: Address, amount: i128) {
        let topics = (Symbol::new(e, "withdraw"), account);
        e.events().publish(topics, amount);
    }
}
