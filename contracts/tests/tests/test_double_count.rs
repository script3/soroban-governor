#[cfg(test)]
use soroban_governor::types::{Calldata, ProposalAction};
use soroban_governor::GovernorContractClient;
use soroban_sdk::{testutils::Address as _, vec, Address, Env, Error, IntoVal, Symbol};
use tests::common::create_stellar_token;
use tests::governor::create_soroban_governor_wasm;
use tests::{
    env::EnvTestUtils,
    governor::{default_governor_settings, default_proposal_data},
    mocks::create_mock_subcall_contract_wasm,
    votes::SorobanVotesClient,
};

/// @dev
/// This test explicitly checks that votes are not double counted. However, this also
/// prevents any potential flash loan attack, where a user could borrow tokens, vote,
/// and return them interest free.
#[test]
fn test_double_count() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    e.budget().reset_unlimited();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);

    let settings = default_governor_settings();
    let (governor_address, votes_address) = create_soroban_governor_wasm(&e, &bombadil, &settings);
    let votes_client = SorobanVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);

    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (subcall_address, _) =
        create_mock_subcall_contract_wasm(&e, &token_address, &governor_address);

    let gov_balance: i128 = 123 * 10i128.pow(7);
    token_client.mint(&governor_address, &gov_balance);

    // set intial votes
    let frodo_votes: i128 = 10_000 * 10i128.pow(7);
    votes_client.mint(&frodo, &frodo_votes);

    let samwise_votes = 5_000 * 10i128.pow(7);
    votes_client.mint(&samwise, &samwise_votes);

    let pippin_votes = 9_000 * 10i128.pow(7);
    votes_client.mint(&pippin, &pippin_votes);

    let total_votes = frodo_votes + samwise_votes + pippin_votes;

    assert_eq!(votes_client.get_votes(&frodo), frodo_votes);
    assert_eq!(votes_client.get_votes(&samwise), samwise_votes);
    assert_eq!(votes_client.get_votes(&pippin), pippin_votes);
    assert_eq!(votes_client.total_supply(), total_votes);

    // create a proposal
    let (title, description, _) = default_proposal_data(&e);
    let call_amount: i128 = 100 * 10i128.pow(7);
    let action = ProposalAction::Calldata(Calldata {
        contract_id: subcall_address.clone(),
        function: Symbol::new(&e, "subcall"),
        args: (call_amount.clone(),).into_val(&e),
        auths: vec![
            &e,
            Calldata {
                contract_id: token_address,
                function: Symbol::new(&e, "transfer"),
                args: (
                    governor_address.clone(),
                    subcall_address.clone(),
                    call_amount.clone(),
                )
                    .into_val(&e),
                auths: vec![&e],
            },
        ],
    });
    let proposal_id = governor_client.propose(&frodo, &title, &description, &action);

    // pass time to one ledger before vote start
    e.jump(settings.vote_delay - 1);

    // pippin mints more tokens
    votes_client.mint(&pippin, &pippin_votes);

    // pass time to the same ledger voting starts
    e.jump(1);

    // frodo will attempt to perform a double vote with samwise
    // frodo mints more tokens, votes, sends them to samwise, and then
    // samwise votes with them, to pass the proposal

    // frodo mints more tokens
    let double_vote_amount = 9 * 10i128.pow(7);
    votes_client.mint(&frodo, &double_vote_amount);

    // frodo votes and fails
    let result = governor_client.try_vote(&frodo, &proposal_id, &2);
    assert_eq!(result.err(), Some(Ok(Error::from_contract_error(103))));

    // frodo sends tokens to samwise
    votes_client.transfer(&frodo, &samwise, &double_vote_amount);

    // samwise votes and fails
    let result_2 = governor_client.try_vote(&samwise, &proposal_id, &2);
    assert_eq!(result_2.err(), Some(Ok(Error::from_contract_error(103))));

    e.jump(1);

    // everyone can vote and things that occured on the block of the vote start are tracked

    votes_client.transfer(&frodo, &pippin, &1);
    governor_client.vote(&frodo, &proposal_id, &1);
    governor_client.vote(&samwise, &proposal_id, &1);
    governor_client.vote(&pippin, &proposal_id, &0);

    // verify proposal votes
    let proposal_votes = governor_client.get_proposal_votes(&proposal_id).unwrap();
    assert_eq!(proposal_votes.against, pippin_votes * 2);
    assert_eq!(
        proposal_votes._for,
        frodo_votes + samwise_votes + double_vote_amount
    );
    assert_eq!(proposal_votes.abstain, 0);
}
