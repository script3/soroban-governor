#[cfg(test)]
use sep_41_token::testutils::MockTokenClient;
use soroban_governor::types::{ProposalAction, ProposalStatus};
use soroban_governor::GovernorContractClient;
use soroban_sdk::testutils::{Ledger as _, LedgerInfo};
use soroban_sdk::{testutils::Address as _, Address, Env};

use tests::governor::create_governor_wasm;
use tests::{
    env::EnvTestUtils,
    governor::{default_governor_settings, default_proposal_data},
};
use tests::{votes::BondingVotesClient, ONE_DAY_LEDGERS};

#[test]
fn test_checkpoint_safely_tracked_for_proposals() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    e.budget().reset_unlimited();

    let bombadil = Address::generate(&e);
    let prop_user_1 = Address::generate(&e);
    let prop_user_2 = Address::generate(&e);
    let prop_user_3 = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);

    let settings = default_governor_settings();
    let (governor_address, underlying_address, votes_address) =
        create_governor_wasm(&e, &bombadil, &bombadil, &settings);
    let votes_client = BondingVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);
    let token_client = MockTokenClient::new(&e, &underlying_address);

    let gov_balance: i128 = 123 * 10i128.pow(7);
    token_client.mint(&governor_address, &gov_balance);

    // set intial votes
    let frodo_votes: i128 = 10_000 * 10i128.pow(7);
    token_client.mint(&frodo, &(frodo_votes + 10i128.pow(7)));
    votes_client.deposit(&frodo, &frodo_votes);

    let samwise_votes = 5_000 * 10i128.pow(7);
    token_client.mint(&samwise, &samwise_votes);
    votes_client.deposit(&samwise, &samwise_votes);

    let pippin_votes = 2_000 * 10i128.pow(7);
    token_client.mint(&pippin, &(pippin_votes * 2));
    votes_client.deposit(&pippin, &pippin_votes);

    token_client.mint(&prop_user_1, &10i128.pow(7));
    votes_client.deposit(&prop_user_1, &10i128.pow(7));
    token_client.mint(&prop_user_2, &10i128.pow(7));
    votes_client.deposit(&prop_user_2, &10i128.pow(7));
    token_client.mint(&prop_user_3, &10i128.pow(7));
    votes_client.deposit(&prop_user_3, &10i128.pow(7));

    // pippin delegate to frodo
    votes_client.delegate(&pippin, &frodo);

    // create a proposal w/ a vote delay (1 day ledgers)
    let (title, description, action) = default_proposal_data(&e);
    let real_proposal_id = governor_client.propose(&prop_user_1, &title, &description, &action);

    e.jump(ONE_DAY_LEDGERS / 2);

    // create a snapshot proposal (no vote delay) to cause out of order vote ledger
    let action = ProposalAction::Snapshot;
    let snapshot_proposal_id = governor_client.propose(&prop_user_2, &title, &description, &action);

    e.jump(5);

    // pippin forces update to Frodo's votes
    votes_client.withdraw(&pippin, &10i128.pow(7));

    e.jump(1);

    // vote on snapshot proposal
    governor_client.vote(&frodo, &snapshot_proposal_id, &0);
    governor_client.vote(&samwise, &snapshot_proposal_id, &1);

    let votes_0 = governor_client
        .get_proposal_votes(&snapshot_proposal_id)
        .unwrap();

    assert_eq!(votes_0.against, frodo_votes + pippin_votes);
    assert_eq!(votes_0._for, samwise_votes);
    assert_eq!(votes_0.abstain, 0);

    e.jump(ONE_DAY_LEDGERS / 2);

    // Frodo and Samwise update their votes
    votes_client.deposit(&frodo, &10i128.pow(7));
    votes_client.withdraw(&samwise, &10i128.pow(7));

    e.jump(1);

    governor_client.vote(&frodo, &real_proposal_id, &0);
    governor_client.vote(&samwise, &real_proposal_id, &1);

    let votes_1 = governor_client
        .get_proposal_votes(&real_proposal_id)
        .unwrap();

    assert_eq!(votes_1.against, frodo_votes + pippin_votes - 10i128.pow(7));
    assert_eq!(votes_1._for, samwise_votes);
    assert_eq!(votes_1.abstain, 0);
}

#[test]
fn test_checkpoints_retained_long_enough() {
    let e = Env::default();
    e.set_default_info();
    e.ledger().set(LedgerInfo {
        timestamp: 1441065600, // Sept 1st, 2015 12:00:00 AM UTC
        protocol_version: 20,
        sequence_number: 100,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: ONE_DAY_LEDGERS,
        // required to ensure token balances don't expire
        min_persistent_entry_ttl: 100 * ONE_DAY_LEDGERS,
        max_entry_ttl: 365 * ONE_DAY_LEDGERS,
    });
    e.mock_all_auths();
    e.budget().reset_unlimited();

    let bombadil = Address::generate(&e);
    let prop_user_1 = Address::generate(&e);
    let prop_user_2 = Address::generate(&e);
    let prop_user_3 = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);

    let mut settings = default_governor_settings();
    // set settings to max grace period and vote period while ensuring
    // maximum proposal lifetime is reached
    settings.vote_period = 7 * ONE_DAY_LEDGERS;
    settings.grace_period = 7 * ONE_DAY_LEDGERS;
    settings.vote_delay = 7 * ONE_DAY_LEDGERS;
    settings.timelock = 0;
    settings.quorum = 8000; // force close to fail if supply checkpoint is lost
    let (governor_address, underlying_address, votes_address) =
        create_governor_wasm(&e, &bombadil, &bombadil, &settings);
    let votes_client = BondingVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);
    let token_client = MockTokenClient::new(&e, &underlying_address);

    let gov_balance: i128 = 123 * 10i128.pow(7);
    token_client.mint(&governor_address, &gov_balance);

    // set intial votes
    let frodo_votes: i128 = 10_000 * 10i128.pow(7);
    token_client.mint(&frodo, &(frodo_votes * 3));
    votes_client.deposit(&frodo, &frodo_votes);

    let samwise_votes = 2_000 * 10i128.pow(7);
    token_client.mint(&samwise, &(frodo_votes * 2));
    votes_client.deposit(&samwise, &samwise_votes);

    token_client.mint(&prop_user_1, &10i128.pow(7));
    votes_client.deposit(&prop_user_1, &10i128.pow(7));
    token_client.mint(&prop_user_2, &10i128.pow(7));
    votes_client.deposit(&prop_user_2, &10i128.pow(7));
    token_client.mint(&prop_user_3, &10i128.pow(7));
    votes_client.deposit(&prop_user_3, &10i128.pow(7));

    // allow time to pass before any proposals are created
    e.jump(31 * ONE_DAY_LEDGERS);

    // create an executable proposal
    let (title, description, _) = default_proposal_data(&e);
    let action = ProposalAction::Council(frodo.clone());
    let proposal_id = governor_client.propose(&prop_user_1, &title, &description, &action);
    let executable_vote_ledger = e.ledger().sequence() + 7 * ONE_DAY_LEDGERS;

    e.jump(1);

    // create a snapshot proposal (no vote delay) to create checkpoints
    let snapshot = ProposalAction::Snapshot;
    let snapshot_proposal_id_1 =
        governor_client.propose(&prop_user_2, &title, &description, &snapshot);

    e.jump(1);

    // force checkpoint for supply and frodo
    votes_client.deposit(&frodo, &10i128.pow(7));

    // Jump to the end of the snapshot proposal voting period
    e.jump(7 * ONE_DAY_LEDGERS - 2);

    governor_client.vote(&frodo, &snapshot_proposal_id_1, &1);
    governor_client.vote(&samwise, &snapshot_proposal_id_1, &0);

    let votes_0 = governor_client
        .get_proposal_votes(&snapshot_proposal_id_1)
        .unwrap();
    assert_eq!(votes_0.against, samwise_votes);
    assert_eq!(votes_0._for, frodo_votes);

    // Jump one ledger past the vote start of the executable proposal
    e.jump(1);

    // force checkpoint for supply and frodo
    votes_client.deposit(&frodo, &frodo_votes);

    // Jump to the end of the voting period for the executable proposal
    e.jump(7 * ONE_DAY_LEDGERS - 1);

    governor_client.vote(&frodo, &proposal_id, &1);
    governor_client.vote(&samwise, &proposal_id, &0);

    let votes_1 = governor_client.get_proposal_votes(&proposal_id).unwrap();
    assert_eq!(votes_1.against, samwise_votes);
    assert_eq!(votes_1._for, frodo_votes + 10i128.pow(7));

    // Jump to one ledger before the end of the grace period for the executable proposal
    e.jump(7 * ONE_DAY_LEDGERS - 1);

    // create another snapshot proposal
    let _ = governor_client.propose(&prop_user_3, &title, &description, &snapshot);

    e.jump(1);

    // force checkpoint for supply and samwise
    votes_client.deposit(&frodo, &10i128.pow(7));

    // try and close proposal
    governor_client.close(&proposal_id);

    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Successful);
    assert_eq!(proposal.data.eta, e.ledger().sequence() + settings.timelock);

    let past_supply = votes_client.get_past_total_supply(&executable_vote_ledger);
    assert_eq!(
        past_supply,
        frodo_votes + samwise_votes + 3 * 10i128.pow(7) + 10i128.pow(7)
    );

    let past_votes = votes_client.get_past_votes(&frodo, &executable_vote_ledger);
    assert_eq!(past_votes, frodo_votes + 10i128.pow(7));
}
