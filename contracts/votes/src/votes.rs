use soroban_sdk::{Address, Env};

#[cfg(all(feature = "admin", not(feature = "wrapped")))]
use soroban_sdk::String;

pub trait Votes {
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

    /// Get the total supply of voting tokens at a specific ledger sequence number.
    /// The ledger must be finalized before the sequence number can be used.
    ///
    /// ### Arguments
    /// * `sequence` - The sequence number to get the total voting token supply at
    ///
    /// ### Panics
    /// Panics if the sequence number is greater than or equal to the current ledger sequence.
    fn get_past_total_supply(e: Env, sequence: u32) -> i128;

    /// Get the current voting power of an account
    ///
    /// ### Arguments
    /// * `account` - The address of the account
    fn get_votes(e: Env, account: Address) -> i128;

    /// Get the voting power of an account at a specific ledger sequence number.
    /// The ledger must be finalized before the sequence number can be used.
    ///
    /// ### Arguments
    /// * `account` - The address of the account
    /// * `sequence` - The sequence number to get the voting power at
    ///
    /// ### Panics
    /// Panics if the sequence number is greater than or equal to the current ledger sequence.
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
}

#[cfg(feature = "wrapped")]
pub trait WrappedToken {
    /// Setup the votes contract
    ///
    /// ### Arguments
    /// * `token` - The address of the underlying token contract
    /// * `governor`- The address of the Governor contract the votes apply to
    fn initialize(e: Env, token: Address, governor: Address);

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

#[cfg(feature = "admin")]
pub trait Admin {
    /// (Admin only) Mint tokens to an address
    ///
    /// ### Arguments
    /// * `from` - The address of the account to deposit for
    /// * `amount` - The amount of underlying tokens to deposit
    fn mint(e: Env, to: Address, amount: i128);

    /// (Admin only) Set the admin of the token to a new address
    ///
    /// ### Arguments
    /// * `new_admin` - The address of the new admin
    fn set_admin(e: Env, new_admin: Address);

    /// Get the admin of the token
    fn admin(e: Env) -> Address;
}

#[cfg(all(feature = "admin", not(feature = "wrapped")))]
pub trait SorobanOnly {
    /// Setup the votes contract
    ///
    /// ### Arguments
    /// * `admin` - The address of the admin of the votes contract
    /// * `governor`- The address of the Governor contract the votes apply to
    /// * `decimal` - The number of decimal places for the voting token
    /// * `name` - The name of the voting token
    /// * `symbol` - The symbol of the voting token
    fn initialize(
        e: Env,
        admin: Address,
        governor: Address,
        decimal: u32,
        name: String,
        symbol: String,
    );
}

#[cfg(feature = "wrapped")]
pub trait Emissions {
    /// Claim emissions for a user into their vote token balance
    ///
    /// Returns the number of tokens claimed
    ///
    /// ### Arguments
    /// * `address` - The address to claim tokens for
    fn claim(e: Env, address: Address) -> i128;

    /// (Governor only) Set the emissions configuration for the vote token. Emits the tokens
    /// evenly over the duration of the emissions period.
    ///
    /// ### Arguments
    /// * `tokens` - The number of new tokens to emit
    /// * `expiration` - When to stop emitting tokens
    fn set_emis(e: Env, tokens: i128, expiration: u64);
}
