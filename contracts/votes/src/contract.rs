use sep_41_token::{Token, TokenEvents};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, unwrap::UnwrapOptimized, Address, Env, String,
};

use crate::{
    allowance::{create_allowance, spend_allowance},
    balance::{receive_balance, spend_balance},
    checkpoints::{upper_lookup, Checkpoint},
    constants::MAX_CHECKPOINT_AGE_LEDGERS,
    error::TokenVotesError,
    events::VoterTokenEvents,
    storage::{self, set_delegate, TokenMetadata},
    validation::require_nonnegative_amount,
    votes::Votes,
    voting_units::{
        burn_voting_units, mint_voting_units, move_voting_units, transfer_voting_units,
    },
};

#[cfg(feature = "admin")]
use crate::votes::Admin;

#[cfg(feature = "wrapped")]
use crate::votes::WrappedToken;
#[cfg(feature = "wrapped")]
use soroban_sdk::token::TokenClient;

#[cfg(all(feature = "admin", not(feature = "wrapped")))]
use crate::votes::SorobanOnly;

#[contract]
pub struct TokenVotes;

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

        spend_balance(&e, &from, amount);
        receive_balance(&e, &to, amount);
        transfer_voting_units(&e, &from, &to, amount);

        TokenEvents::transfer(&e, from, to, amount);
    }

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        require_nonnegative_amount(&e, amount);
        storage::extend_instance(&e);

        spend_allowance(&e, &from, &spender, amount);
        spend_balance(&e, &from, amount);
        receive_balance(&e, &to, amount);
        transfer_voting_units(&e, &from, &to, amount);

        TokenEvents::transfer(&e, from, to, amount);
    }

    // TODO: Consider making these functions a no-op?
    fn burn(e: Env, from: Address, amount: i128) {
        from.require_auth();
        require_nonnegative_amount(&e, amount);
        storage::extend_instance(&e);

        spend_balance(&e, &from, amount);
        burn_voting_units(&e, &from, amount);

        // burn underlying from the tokens held by this contract
        #[cfg(feature = "wrapped")]
        TokenClient::new(&e, &storage::get_token(&e)).burn(&e.current_contract_address(), &amount);

        TokenEvents::burn(&e, from, amount);
    }

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();
        require_nonnegative_amount(&e, amount);
        storage::extend_instance(&e);

        spend_allowance(&e, &from, &spender, amount);
        spend_balance(&e, &from, amount);
        burn_voting_units(&e, &from, amount);

        // burn underlying from the tokens held by this contract
        #[cfg(feature = "wrapped")]
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

        VoterTokenEvents::delegate(&e, account, delegatee, cur_delegate)
    }
}

#[cfg(feature = "admin")]
#[contractimpl]
impl Admin for TokenVotes {
    fn mint(e: Env, to: Address, amount: i128) {
        require_nonnegative_amount(&e, amount);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        storage::extend_instance(&e);

        receive_balance(&e, &to, amount);
        mint_voting_units(&e, &to, amount);

        TokenEvents::mint(&e, admin, to, amount);
    }

    fn set_admin(e: Env, new_admin: Address) {
        let admin = storage::get_admin(&e);
        admin.require_auth();
        storage::extend_instance(&e);

        storage::set_admin(&e, &new_admin);

        VoterTokenEvents::set_admin(&e, admin, new_admin);
    }

    fn admin(e: Env) -> Address {
        storage::get_admin(&e)
    }
}

#[cfg(feature = "wrapped")]
#[contractimpl]
impl WrappedToken for TokenVotes {
    fn initialize(e: Env, token: Address, governor: Address) {
        if storage::get_is_init(&e) {
            panic_with_error!(e, TokenVotesError::AlreadyInitializedError);
        }
        storage::extend_instance(&e);

        let underlying_token = TokenClient::new(&e, &token);
        let decimal = underlying_token.decimals();
        let symbol = underlying_token.symbol();
        let name = underlying_token.name();
        // TODO: Come up with custom symbol and name for the token
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

    fn deposit_for(e: Env, from: Address, amount: i128) {
        from.require_auth();
        storage::extend_instance(&e);

        let token = TokenClient::new(&e, &storage::get_token(&e));
        token.transfer(&from, &e.current_contract_address(), &amount);

        receive_balance(&e, &from, amount);
        mint_voting_units(&e, &from, amount);

        VoterTokenEvents::deposit(&e, from, amount);
    }

    fn withdraw_to(e: Env, from: Address, amount: i128) {
        from.require_auth();
        storage::extend_instance(&e);

        spend_balance(&e, &from, amount);
        burn_voting_units(&e, &from, amount);

        let token = TokenClient::new(&e, &storage::get_token(&e));
        token.transfer(&e.current_contract_address(), &from, &amount);

        VoterTokenEvents::withdraw(&e, from, amount);
    }
}

#[cfg(all(feature = "admin", not(feature = "wrapped")))]
#[contractimpl]
impl SorobanOnly for TokenVotes {
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
}
