use sep_41_token::{Token, TokenClient, TokenEvents};
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env, String};

use crate::{
    allowance::{create_allowance, spend_allowance},
    balance::{receive_balance, spend_balance},
    checkpoints::upper_lookup,
    error::TokenVotesError,
    events::VoterTokenEvents,
    storage::{self, set_delegate, TokenMetadata, VotingUnits},
    validation::require_nonnegative_amount,
    votes::Votes,
    voting_units::{
        burn_voting_units, mint_voting_units, move_voting_units, transfer_voting_units,
    },
};

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
        let token = TokenClient::new(&e, &storage::get_token(&e));
        token.burn(&e.current_contract_address(), &amount);

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
        let token = TokenClient::new(&e, &storage::get_token(&e));
        token.burn(&e.current_contract_address(), &amount);

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
    fn initialize(e: Env, token: Address) {
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
        storage::set_total_supply(
            &e,
            &VotingUnits {
                amount: 0,
                timestamp: e.ledger().timestamp(),
            },
        );
        storage::set_is_init(&e);
    }

    fn total_supply(e: Env) -> i128 {
        storage::extend_instance(&e);
        storage::get_total_supply(&e).amount
    }

    fn get_past_total_supply(e: Env, timestamp: u64) -> i128 {
        storage::extend_instance(&e);
        let cur_supply = storage::get_total_supply(&e);
        if cur_supply.timestamp <= timestamp {
            return cur_supply.amount;
        }
        let supply_checkpoints = storage::get_total_supply_checkpoints(&e);
        let past_voting_units = upper_lookup(&supply_checkpoints, timestamp);
        match past_voting_units {
            Some(voting_units) => voting_units.amount,
            None => 0,
        }
    }

    fn get_votes(e: Env, account: Address) -> i128 {
        storage::extend_instance(&e);
        storage::get_voting_units(&e, &account).amount
    }

    fn get_past_votes(e: Env, user: Address, timestamp: u64) -> i128 {
        storage::extend_instance(&e);
        let cur_votes = storage::get_voting_units(&e, &user);
        if cur_votes.timestamp <= timestamp {
            return cur_votes.amount;
        }
        let checkpoints = storage::get_voting_units_checkpoints(&e, &user);
        let past_voting_units = upper_lookup(&checkpoints, timestamp);
        match past_voting_units {
            Some(voting_units) => voting_units.amount,
            None => 0,
        }
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
        if balance > 0 {
            move_voting_units(&e, Some(&cur_delegate), Some(&dest_delegate), balance);
        }
        set_delegate(&e, &account, &delegatee);

        VoterTokenEvents::delegate(&e, account, delegatee, cur_delegate)
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
