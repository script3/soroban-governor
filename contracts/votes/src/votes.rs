use soroban_sdk::{Address, Env};

pub trait Votes {
    /// Setup the votes contract
    ///
    /// ### Arguments
    /// * `token` - The address of the underlying token contract
    /// * `governor`- The address of the Governor contract the votes apply to
    fn initialize(e: Env, token: Address, governor: Address);

    /// Get the total supply of voting tokens
    fn total_supply(e: Env) -> i128;

    /// Set a new sequence number of a future vote. This ensures vote history is maintained
    /// for old votes.
    ///
    /// Requires auth from the governor contract
    ///
    /// ### Arguments
    /// * `sequence` - The sequence number of the vote
    fn set_vote_sequence(e: Env, sequence: u32);

    /// Get the total supply of voting tokens at a specific ledger sequence number
    ///
    /// ### Arguments
    /// * `sequence` - The sequence number to get the total voting token supply at
    fn get_past_total_supply(e: Env, sequence: u32) -> i128;

    /// Get the current voting power of an account
    ///
    /// ### Arguments
    /// * `account` - The address of the account
    fn get_votes(e: Env, account: Address) -> i128;

    /// Get the voting power of an account at a specific ledger sequence number
    ///
    /// ### Arguments
    /// * `account` - The address of the account
    /// * `sequence` - The sequence number to get the voting power at
    fn get_past_votes(e: Env, user: Address, sequence: u32) -> i128;

    /// Get the deletage that account has chosen
    ///
    /// ### Arguments
    /// * `account` - The address of the account
    fn get_delegate(e: Env, account: Address) -> Address;

    /// Delegate the voting power of the account to a delegate
    ///
    /// ### Arguments
    /// * `delegate` - The address of the delegate
    fn delegate(e: Env, account: Address, delegatee: Address);

    /// Deposit underlying tokens into the votes contract and mint the corresponding
    /// amount of voting tokens
    ///
    /// ### Arguments
    /// * `from` - The address of the account to deposit for
    /// * `amount` - The amount of underlying tokens to deposit
    fn deposit_for(e: Env, from: Address, amount: i128);

    /// Burn voting tokens and withdraw the corresponding amount of underlying tokens
    ///
    /// ### Arguments
    /// * `from` - The address of the account to withdraw for
    /// * `amount` - The amount of underlying tokens to withdraw
    fn withdraw_to(e: Env, from: Address, amount: i128);
}
