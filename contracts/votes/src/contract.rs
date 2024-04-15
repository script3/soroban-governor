use soroban_sdk::{
    contract, contractimpl, panic_with_error, unwrap::UnwrapOptimized, Address, Env, String,
};

use crate::{
    balance,
    checkpoints::{upper_lookup, Checkpoint},
    constants::MAX_CHECKPOINT_AGE_LEDGERS,
    error::TokenVotesError,
    events::TokenVotesEvents,
    storage::{self, set_delegate, TokenMetadata},
    validation::require_nonnegative_amount,
    votes::Votes,
    voting_units::move_voting_units,
};

// SEP-0041 Feature imports

#[cfg(any(feature = "sep-0041", not(feature = "bonding")))]
use sep_41_token::TokenEvents;

#[cfg(feature = "sep-0041")]
use sep_41_token::Token;

#[cfg(feature = "sep-0041")]
use crate::allowance::{create_allowance, spend_allowance};

// Bonding Feature imports

#[cfg(feature = "bonding")]
use crate::{
    emissions::{claim_emissions, set_emissions},
    votes::Bonding,
};
#[cfg(feature = "bonding")]
use soroban_sdk::token::TokenClient;

// Admin (Bonding not enabled) Feature imports

#[cfg(not(feature = "bonding"))]
use crate::votes::Admin;
#[cfg(not(feature = "bonding"))]
use soroban_sdk::Symbol;

// Token Data Feature imports (SEP-0041 not enabled)

#[cfg(not(feature = "sep-0041"))]
use crate::votes::TokenData;

#[contract]
pub struct TokenVotes;

#[cfg(feature = "sep-0041")]
#[contractimpl]
/// Implementation of the SEP-41 Token trait.
impl Token for TokenVotes {
    fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        let result = storage::get_allowance(&e, &from, &spender);
        result.amount
    }

    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();
        require_nonnegative_amount(&e, amount);
        storage::extend_instance(&e);

        create_allowance(&e, &from, &spender, amount, expiration_ledger);

        TokenEvents::approve(&e, from, spender, amount, expiration_ledger);
    }

    fn balance(e: Env, id: Address) -> i128 {
        storage::extend_instance(&e);
        storage::get_balance(&e, &id)
    }

    fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        require_nonnegative_amount(&e, amount);
        storage::extend_instance(&e);

        balance::transfer_balance(&e, &from, &to, amount);

        TokenEvents::transfer(&e, from, to, amount);
    }

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        require_nonnegative_amount(&e, amount);
        storage::extend_instance(&e);

        spend_allowance(&e, &from, &spender, amount);
        balance::transfer_balance(&e, &from, &to, amount);

        TokenEvents::transfer(&e, from, to, amount);
    }

    fn burn(e: Env, from: Address, amount: i128) {
        from.require_auth();
        require_nonnegative_amount(&e, amount);
        storage::extend_instance(&e);

        balance::burn_balance(&e, &from, amount);

        // burn underlying from the tokens held by this contract
        #[cfg(feature = "bonding")]
        TokenClient::new(&e, &storage::get_token(&e)).burn(&e.current_contract_address(), &amount);

        TokenEvents::burn(&e, from, amount);
    }

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();
        require_nonnegative_amount(&e, amount);
        storage::extend_instance(&e);

        spend_allowance(&e, &from, &spender, amount);
        balance::burn_balance(&e, &from, amount);

        // burn underlying from the tokens held by this contract
        #[cfg(feature = "bonding")]
        TokenClient::new(&e, &storage::get_token(&e)).burn(&e.current_contract_address(), &amount);

        TokenEvents::burn(&e, from, amount);
    }

    fn decimals(e: Env) -> u32 {
        storage::get_metadata(&e).decimal
    }

    fn name(e: Env) -> String {
        storage::get_metadata(&e).name
    }

    fn symbol(e: Env) -> String {
        storage::get_metadata(&e).symbol
    }
}

#[contractimpl]
/// Implementation of the Votes trait to allow for tracking votes
impl Votes for TokenVotes {
    fn total_supply(e: Env) -> i128 {
        storage::extend_instance(&e);
        storage::get_total_supply(&e).to_checkpoint_data().1
    }

    fn set_vote_sequence(e: Env, sequence: u32) {
        storage::get_governor(&e).require_auth();
        storage::extend_instance(&e);

        let mut vote_ledgers = storage::get_vote_ledgers(&e);
        let len = vote_ledgers.len();
        let ledger_cutoff = e
            .ledger()
            .sequence()
            .checked_sub(MAX_CHECKPOINT_AGE_LEDGERS);
        if len > 0 && ledger_cutoff.is_some() {
            // if the `ledger_cutoff` is found or if the index in which it could
            // be inserted is returned, we remove everything before it
            let result = vote_ledgers.binary_search(ledger_cutoff.unwrap_optimized());
            let index = match result {
                Ok(index) => index,
                Err(index) => index,
            };
            // check if there is anything to remove before doing the work
            if index > 0 {
                vote_ledgers = vote_ledgers.slice(index..len);
            }
        }
        vote_ledgers.push_back(sequence);
        storage::set_vote_ledgers(&e, &vote_ledgers);
    }

    fn get_past_total_supply(e: Env, sequence: u32) -> i128 {
        storage::extend_instance(&e);
        if sequence >= e.ledger().sequence() {
            panic_with_error!(e, TokenVotesError::SequenceNotClosedError);
        }
        let cur_supply = storage::get_total_supply(&e);
        let (cur_seq, cur_supply) = cur_supply.to_checkpoint_data();
        if cur_seq <= sequence {
            return cur_supply;
        }
        let supply_checkpoints = storage::get_total_supply_checkpoints(&e);
        upper_lookup(&e, &supply_checkpoints, sequence)
    }

    fn get_votes(e: Env, account: Address) -> i128 {
        storage::extend_instance(&e);
        storage::get_voting_units(&e, &account)
            .to_checkpoint_data()
            .1
    }

    fn get_past_votes(e: Env, user: Address, sequence: u32) -> i128 {
        storage::extend_instance(&e);
        if sequence >= e.ledger().sequence() {
            panic_with_error!(e, TokenVotesError::SequenceNotClosedError);
        }
        let cur_votes = storage::get_voting_units(&e, &user);
        let (cur_seq, cur_amount) = cur_votes.to_checkpoint_data();
        if cur_seq <= sequence {
            return cur_amount;
        }
        let checkpoints = storage::get_voting_units_checkpoints(&e, &user);
        upper_lookup(&e, &checkpoints, sequence)
    }

    fn get_delegate(e: Env, account: Address) -> Address {
        storage::extend_instance(&e);
        storage::get_delegate(&e, &account)
    }

    fn delegate(e: Env, account: Address, delegatee: Address) {
        account.require_auth();
        storage::extend_instance(&e);
        let cur_delegate = storage::get_delegate(&e, &account);
        if cur_delegate == delegatee {
            panic_with_error!(e, TokenVotesError::InvalidDelegateeError);
        }
        let dest_delegate = storage::get_delegate(&e, &delegatee);
        let balance = storage::get_balance(&e, &account);
        let vote_ledgers = storage::get_vote_ledgers(&e);
        if balance > 0 {
            move_voting_units(
                &e,
                &vote_ledgers,
                Some(&cur_delegate),
                Some(&dest_delegate),
                balance,
            );
        }
        set_delegate(&e, &account, &delegatee);

        TokenVotesEvents::delegate(&e, account, delegatee, cur_delegate)
    }
}

#[cfg(feature = "bonding")]
#[contractimpl]
impl Bonding for TokenVotes {
    fn initialize(e: Env, token: Address, governor: Address, name: String, symbol: String) {
        if storage::get_is_init(&e) {
            panic_with_error!(e, TokenVotesError::AlreadyInitializedError);
        }
        storage::extend_instance(&e);

        let underlying_token = TokenClient::new(&e, &token);
        let decimal = underlying_token.decimals();
        let token_metadata = TokenMetadata {
            decimal,
            name,
            symbol,
        };
        storage::set_metadata(&e, &token_metadata);
        storage::set_token(&e, &token);
        storage::set_governor(&e, &governor);
        storage::set_is_init(&e);
    }

    fn deposit(e: Env, from: Address, amount: i128) {
        require_nonnegative_amount(&e, amount);
        from.require_auth();
        storage::extend_instance(&e);

        let token = TokenClient::new(&e, &storage::get_token(&e));
        token.transfer(&from, &e.current_contract_address(), &amount);

        balance::mint_balance(&e, &from, amount);

        TokenVotesEvents::deposit(&e, from, amount);
    }

    fn withdraw(e: Env, from: Address, amount: i128) {
        require_nonnegative_amount(&e, amount);
        from.require_auth();
        storage::extend_instance(&e);

        balance::burn_balance(&e, &from, amount);

        let token = TokenClient::new(&e, &storage::get_token(&e));
        token.transfer(&e.current_contract_address(), &from, &amount);

        TokenVotesEvents::withdraw(&e, from, amount);
    }

    fn claim(e: Env, address: Address) -> i128 {
        address.require_auth();
        let total_supply = storage::get_total_supply(&e).to_checkpoint_data().1;
        let balance = storage::get_balance(&e, &address);
        claim_emissions(&e, total_supply, &address, balance)
    }

    fn set_emis(e: Env, tokens: i128, expiration: u64) {
        let governor = storage::get_governor(&e);
        governor.require_auth();

        let token = TokenClient::new(&e, &storage::get_token(&e));
        token.transfer(&governor, &e.current_contract_address(), &tokens);

        let total_supply = storage::get_total_supply(&e).to_checkpoint_data().1;
        set_emissions(&e, total_supply, tokens, expiration);
    }
}

#[cfg(not(feature = "bonding"))]
#[contractimpl]
impl Admin for TokenVotes {
    fn initialize(
        e: Env,
        admin: Address,
        governor: Address,
        decimal: u32,
        name: String,
        symbol: String,
    ) {
        if storage::get_is_init(&e) {
            panic_with_error!(e, TokenVotesError::AlreadyInitializedError);
        }
        storage::extend_instance(&e);

        storage::set_admin(&e, &admin);
        storage::set_governor(&e, &governor);
        let token_metadata = TokenMetadata {
            decimal,
            name,
            symbol,
        };
        storage::set_metadata(&e, &token_metadata);
        storage::set_is_init(&e);
    }

    fn mint(e: Env, to: Address, amount: i128) {
        require_nonnegative_amount(&e, amount);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        storage::extend_instance(&e);

        balance::mint_balance(&e, &to, amount);

        TokenEvents::mint(&e, admin, to, amount);
    }

    #[cfg(feature = "clawback")]
    fn clawback(e: Env, from: Address, amount: i128) {
        require_nonnegative_amount(&e, amount);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        storage::extend_instance(&e);

        balance::burn_balance(&e, &from, amount);

        let topics = (Symbol::new(&e, "clawback"), from);
        e.events().publish(topics, amount);
    }

    fn set_admin(e: Env, new_admin: Address) {
        let admin = storage::get_admin(&e);
        admin.require_auth();
        storage::extend_instance(&e);

        storage::set_admin(&e, &new_admin);

        TokenVotesEvents::set_admin(&e, admin, new_admin);
    }

    fn admin(e: Env) -> Address {
        storage::get_admin(&e)
    }
}

#[cfg(not(feature = "sep-0041"))]
#[contractimpl]
impl TokenData for TokenVotes {
    fn balance(e: Env, id: Address) -> i128 {
        storage::extend_instance(&e);
        storage::get_balance(&e, &id)
    }

    fn decimals(e: Env) -> u32 {
        storage::get_metadata(&e).decimal
    }

    fn name(e: Env) -> String {
        storage::get_metadata(&e).name
    }

    fn symbol(e: Env) -> String {
        storage::get_metadata(&e).symbol
    }
}
