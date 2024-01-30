use soroban_governor::storage::{Proposal, VoteCount};
#[cfg(test)]
use soroban_governor::{
    dependencies::{VotesClient, VOTES_WASM},
    storage::{self, Calldata, ProposalStatus, SubCalldata},
};
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{vec, Address, IntoVal, String};
use soroban_sdk::{Env, Symbol};
use tests::common::create_govenor;

#[test]
fn test_close_proposal_queued() {
    let e = Env::default();
    let (govenor_address, votes_address, _, govenor) = create_govenor(&e);
    e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
    let votes_client = VotesClient::new(&e, &votes_address);
    let creater = Address::generate(&e);

    votes_client.set_votes(&creater, &10_000_000_i128);
    let calldata = Calldata {
        contract_id: Address::generate(&e),
        function: Symbol::new(&e, "test"),
        args: (1, 2, 3).into_val(&e),
    };
    let sub_calldata = vec![
        &e,
        SubCalldata {
            contract_id: Address::generate(&e),
            function: Symbol::new(&e, "test"),
            args: (1, 2, 3).into_val(&e),
            sub_auth: vec![&e],
        },
    ];
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
                sub_calldata,
                vote_start: 0,
                vote_end: 1000,
            },
        );
        // Set proposal votes to ensure quorum is met
        storage::set_proposal_vote_count(
            &e,
            &proposal_id,
            &VoteCount {
                votes_abstained: 3_000_000,
                votes_against: 1_000_000,
                votes_for: 5_000_000,
            },
        );
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

    govenor.close(&proposal_id);
    e.as_contract(&govenor_address, || {
        let proposal_status = storage::get_proposal_status(&e, &proposal_id);
        assert_eq!(proposal_status, ProposalStatus::Queued);
    });
}
#[test]
fn test_close_quorum_not_met() {
    let e = Env::default();
    let (govenor_address, votes_address, _, govenor) = create_govenor(&e);
    e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
    let votes_client = VotesClient::new(&e, &votes_address);
    let creater = Address::generate(&e);

    votes_client.set_votes(&creater, &10_000_000_i128);
    let calldata = Calldata {
        contract_id: Address::generate(&e),
        function: Symbol::new(&e, "test"),
        args: (1, 2, 3).into_val(&e),
    };
    let sub_calldata = vec![
        &e,
        SubCalldata {
            contract_id: Address::generate(&e),
            function: Symbol::new(&e, "test"),
            args: (1, 2, 3).into_val(&e),
            sub_auth: vec![&e],
        },
    ];
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
                sub_calldata,
                vote_start: 0,
                vote_end: 1000,
            },
        );
        // Set proposal votes to ensure quorum is met
        storage::set_proposal_vote_count(
            &e,
            &proposal_id,
            &VoteCount {
                votes_abstained: 2_000_000,
                votes_against: 1_000_000,
                votes_for: 3_000_000,
            },
        );
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

    govenor.close(&proposal_id);
    e.as_contract(&govenor_address, || {
        let proposal_status = storage::get_proposal_status(&e, &proposal_id);
        assert_eq!(proposal_status, ProposalStatus::Defeated);
    });
}

#[test]
fn test_close_quorum_vote_threshold_not_met() {
    let e = Env::default();
    let (govenor_address, votes_address, _, govenor) = create_govenor(&e);
    e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
    let votes_client = VotesClient::new(&e, &votes_address);
    let creater = Address::generate(&e);

    votes_client.set_votes(&creater, &10_000_000_i128);
    let calldata = Calldata {
        contract_id: Address::generate(&e),
        function: Symbol::new(&e, "test"),
        args: (1, 2, 3).into_val(&e),
    };
    let sub_calldata = vec![
        &e,
        SubCalldata {
            contract_id: Address::generate(&e),
            function: Symbol::new(&e, "test"),
            args: (1, 2, 3).into_val(&e),
            sub_auth: vec![&e],
        },
    ];
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
                sub_calldata,
                vote_start: 0,
                vote_end: 1000,
            },
        );
        // Set proposal votes to ensure quorum is met
        storage::set_proposal_vote_count(
            &e,
            &proposal_id,
            &VoteCount {
                votes_abstained: 6_000_000,
                votes_against: 2_000_000,
                votes_for: 2_000_000,
            },
        );
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

    govenor.close(&proposal_id);
    e.as_contract(&govenor_address, || {
        let proposal_status = storage::get_proposal_status(&e, &proposal_id);
        assert_eq!(proposal_status, ProposalStatus::Defeated);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_close_nonexistent_proposal() {
    let e = Env::default();
    let (govenor_address, votes_address, _, govenor) = create_govenor(&e);
    e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
    let votes_client = VotesClient::new(&e, &votes_address);
    let creater = Address::generate(&e);

    votes_client.set_votes(&creater, &10_000_000_i128);
    let proposal_id = e.as_contract(&govenor_address, || {
        return storage::get_proposal_id(&e);
    });
    govenor.close(&proposal_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn test_close_vote_period_unfinished() {
    let e = Env::default();
    let (govenor_address, votes_address, _, govenor) = create_govenor(&e);
    e.register_contract_wasm(&Some(votes_address.clone()), VOTES_WASM);
    let votes_client = VotesClient::new(&e, &votes_address);
    let creater = Address::generate(&e);

    votes_client.set_votes(&creater, &10_000_000_i128);
    let calldata = Calldata {
        contract_id: Address::generate(&e),
        function: Symbol::new(&e, "test"),
        args: (1, 2, 3).into_val(&e),
    };
    let sub_calldata = vec![
        &e,
        SubCalldata {
            contract_id: Address::generate(&e),
            function: Symbol::new(&e, "test"),
            args: (1, 2, 3).into_val(&e),
            sub_auth: vec![&e],
        },
    ];
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
                sub_calldata,
                vote_start: 0,
                vote_end: 1000,
            },
        );
        // Set proposal votes to ensure quorum is met
        storage::set_proposal_vote_count(
            &e,
            &proposal_id,
            &VoteCount {
                votes_abstained: 3_000_000,
                votes_against: 1_000_000,
                votes_for: 5_000_000,
            },
        );
        return proposal_id;
    });
    e.ledger().set(LedgerInfo {
        timestamp: e.ledger().timestamp().saturating_add(999),
        protocol_version: 20,
        sequence_number: e.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 999999,
        min_persistent_entry_ttl: 999999,
        max_entry_ttl: 9999999,
    });

    govenor.close(&proposal_id);
}
