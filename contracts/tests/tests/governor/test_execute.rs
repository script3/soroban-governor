#[cfg(test)]
use soroban_governor::{
    dependencies::{VotesClient, VOTES_WASM},
    storage::{self, Calldata, Proposal, ProposalStatus},
};
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    vec, Address, Env, IntoVal, String, Symbol,
};
use tests::common::{create_govenor, create_token};

#[test]
fn test_execute() {
    let e = Env::default();
    e.mock_all_auths();
    let (govenor_address, votes_address, settings, govenor) = create_govenor(&e);
    e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
    let votes_client = VotesClient::new(&e, &votes_address);
    let token_admin = Address::generate(&e);
    let (token_address, token_client) = create_token(&e, &token_admin, 7, "test");
    let creater = Address::generate(&e);
    let transfer_recipient = Address::generate(&e);

    token_client.mint(&govenor_address, &1_000_000);
    votes_client.set_votes(&creater, &10_000_000_i128);
    let calldata = Calldata {
        contract_id: token_address,
        function: Symbol::new(&e, "transfer"),
        args: (
            govenor_address.clone(),
            transfer_recipient.clone(),
            1_000_000_i128,
        )
            .into_val(&e),
    };
    let title = String::from_str(&e, "Test Title");
    let description = String::from_str(&e, "Test Description");
    let proposal_id = e.as_contract(&govenor_address, || {
        let proposal_id: u32 = storage::get_proposal_id(&e);
        storage::set_proposal(
            &e,
            &proposal_id,
            &Proposal {
                id: proposal_id,
                title,
                description,
                proposer: creater,
                calldata,
                sub_calldata: vec![&e],
                vote_start: 0,
                vote_end: 1000,
            },
        );

        storage::set_proposal_status(&e, &proposal_id, &ProposalStatus::Queued);
        return proposal_id;
    });
    e.ledger().set(LedgerInfo {
        timestamp: e
            .ledger()
            .timestamp()
            .saturating_add(1000 + settings.timelock),
        protocol_version: 20,
        sequence_number: e.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 999999,
        min_persistent_entry_ttl: 999999,
        max_entry_ttl: 9999999,
    });

    e.set_auths(&[]);
    govenor.execute(&proposal_id);
    e.as_contract(&govenor_address, || {
        let proposal_status = storage::get_proposal_status(&e, &proposal_id);
        assert_eq!(proposal_status, ProposalStatus::Succeeded);
        assert_eq!(token_client.balance(&transfer_recipient), 1_000_000_i128);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn test_execute_nonexistent_proposal() {
    let e = Env::default();
    let (govenor_address, votes_address, _, govenor) = create_govenor(&e);
    e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
    let votes_client = VotesClient::new(&e, &votes_address);
    let creater = Address::generate(&e);

    votes_client.set_votes(&creater, &10_000_000_i128);
    let proposal_id = e.as_contract(&govenor_address, || {
        return storage::get_proposal_id(&e);
    });
    govenor.execute(&proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #205)")]
fn test_execute_proposal_not_queued() {
    let e = Env::default();
    e.mock_all_auths();
    let (govenor_address, votes_address, settings, govenor) = create_govenor(&e);
    e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
    let token_admin = Address::generate(&e);
    let (token_address, _) = create_token(&e, &token_admin, 7, "test");
    let creater = Address::generate(&e);
    let transfer_recipient = Address::generate(&e);

    let calldata = Calldata {
        contract_id: token_address,
        function: Symbol::new(&e, "transfer"),
        args: (
            govenor_address.clone(),
            transfer_recipient.clone(),
            1_000_000_i128,
        )
            .into_val(&e),
    };
    let title = String::from_str(&e, "Test Title");
    let description = String::from_str(&e, "Test Description");
    let proposal_id = e.as_contract(&govenor_address, || {
        let proposal_id: u32 = storage::get_proposal_id(&e);
        storage::set_proposal(
            &e,
            &proposal_id,
            &Proposal {
                id: proposal_id,
                title,
                description,
                proposer: creater,
                calldata,
                sub_calldata: vec![&e],
                vote_start: 0,
                vote_end: 1000,
            },
        );
        storage::set_proposal_status(&e, &proposal_id, &ProposalStatus::Active);
        return proposal_id;
    });

    e.ledger().set(LedgerInfo {
        timestamp: e
            .ledger()
            .timestamp()
            .saturating_add(1000 + settings.timelock),
        protocol_version: 20,
        sequence_number: e.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 999999,
        min_persistent_entry_ttl: 999999,
        max_entry_ttl: 9999999,
    });
    e.set_auths(&[]);
    govenor.execute(&proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #206)")]
fn test_execute_timelock_not_met() {
    let e = Env::default();
    e.mock_all_auths();
    let (govenor_address, votes_address, _, govenor) = create_govenor(&e);
    e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
    let token_admin = Address::generate(&e);
    let (token_address, _) = create_token(&e, &token_admin, 7, "test");
    let creater = Address::generate(&e);
    let transfer_recipient = Address::generate(&e);

    let calldata = Calldata {
        contract_id: token_address,
        function: Symbol::new(&e, "transfer"),
        args: (
            govenor_address.clone(),
            transfer_recipient.clone(),
            1_000_000_i128,
        )
            .into_val(&e),
    };
    let title = String::from_str(&e, "Test Title");
    let description = String::from_str(&e, "Test Description");
    let proposal_id = e.as_contract(&govenor_address, || {
        let proposal_id: u32 = storage::get_proposal_id(&e);
        storage::set_proposal(
            &e,
            &proposal_id,
            &Proposal {
                id: proposal_id,
                title,
                description,
                proposer: creater,
                calldata,
                sub_calldata: vec![&e],
                vote_start: 0,
                vote_end: 1000,
            },
        );
        storage::set_proposal_status(&e, &proposal_id, &ProposalStatus::Queued);
        return proposal_id;
    });

    e.ledger().set(LedgerInfo {
        timestamp: e.ledger().timestamp().saturating_add(1000),
        protocol_version: 20,
        sequence_number: e.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 999999,
        min_persistent_entry_ttl: 999999,
        max_entry_ttl: 9999999,
    });
    e.set_auths(&[]);
    govenor.execute(&proposal_id);
}
