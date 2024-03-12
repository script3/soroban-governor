#[cfg(test)]
use sep_41_token::testutils::MockTokenClient;
use soroban_governor::types::{Calldata, ProposalAction, ProposalStatus};
use soroban_governor::GovernorContractClient;
use soroban_sdk::{testutils::Address as _, vec, Address, Env, IntoVal, Symbol};
use soroban_votes::TokenVotesClient;
use tests::common::create_stellar_token;
use tests::governor::create_soroban_governor_wasm;
use tests::{
    env::EnvTestUtils,
    governor::{create_governor_wasm, default_governor_settings, default_proposal_data},
    mocks::create_mock_subcall_contract_wasm,
};

const ONE_HOUR: u32 = 60 * 60 / 5;

#[test]
fn test_wasm_happy_path() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    e.budget().reset_unlimited();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);
    let merry = Address::generate(&e);

    let settings = default_governor_settings(&e);
    let (governor_address, token_address, votes_address) =
        create_governor_wasm(&e, &bombadil, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let votes_client = TokenVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);
    let (subcall_address, _) =
        create_mock_subcall_contract_wasm(&e, &token_address, &governor_address);

    let gov_balance: i128 = 123 * 10i128.pow(7);
    token_client.mint(&governor_address, &gov_balance);

    // set intial votes
    let mut frodo_votes: i128 = 10_000 * 10i128.pow(7);
    token_client.mint(&frodo, &frodo_votes);
    votes_client.deposit_for(&frodo, &frodo_votes);

    let mut samwise_votes = 5_000 * 10i128.pow(7);
    token_client.mint(&samwise, &samwise_votes);
    votes_client.deposit_for(&samwise, &samwise_votes);

    let pippin_votes = 9_000 * 10i128.pow(7);
    token_client.mint(&pippin, &pippin_votes);
    votes_client.deposit_for(&pippin, &pippin_votes);

    let mut total_votes = frodo_votes + samwise_votes + pippin_votes;

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

    // pass some time - samwise delegates to frodo then transfers votes to merry. Merry mints more votes.
    e.jump(settings.vote_delay - ONE_HOUR);

    votes_client.delegate(&samwise, &frodo);
    frodo_votes += samwise_votes;
    samwise_votes = 0;

    votes_client.transfer(&samwise, &merry, &(1000 * 10i128.pow(7)));
    frodo_votes -= 1000 * 10i128.pow(7);
    let mut merry_votes = 1000 * 10i128.pow(7);

    merry_votes += 2_000 * 10i128.pow(7);
    token_client.mint(&merry, &(2_000 * 10i128.pow(7)));
    votes_client.deposit_for(&merry, &(2_000 * 10i128.pow(7)));
    total_votes += 2_000 * 10i128.pow(7);

    assert_eq!(votes_client.get_votes(&frodo), frodo_votes);
    assert_eq!(votes_client.get_votes(&samwise), samwise_votes);
    assert_eq!(votes_client.get_votes(&merry), merry_votes);
    assert_eq!(votes_client.get_votes(&pippin), pippin_votes);
    assert_eq!(votes_client.total_supply(), total_votes);

    // start the vote period
    e.jump(ONE_HOUR + 1);

    // merry tries to delegate votes to pippin after the delay and mint more votes
    // @dev: don't update vote trackers with changes after the vote period starts
    votes_client.delegate(&merry, &pippin);
    let late_mint_votes = 500 * 10i128.pow(7);
    token_client.mint(&merry, &late_mint_votes);
    votes_client.deposit_for(&merry, &late_mint_votes);
    assert_eq!(votes_client.get_votes(&merry), 0);
    assert_eq!(
        votes_client.get_votes(&pippin),
        pippin_votes + merry_votes + late_mint_votes
    );
    assert_eq!(votes_client.total_supply(), total_votes + late_mint_votes);

    // frodo votes for, pippin votes against, merry abstains
    governor_client.vote(&frodo, &proposal_id, &1);
    e.jump(ONE_HOUR);
    governor_client.vote(&pippin, &proposal_id, &0);
    e.jump(ONE_HOUR);
    governor_client.vote(&merry, &proposal_id, &2);
    e.jump(ONE_HOUR);

    let proposal_votes = governor_client.get_proposal_votes(&proposal_id);
    assert_eq!(proposal_votes.against, pippin_votes);
    assert_eq!(proposal_votes._for, frodo_votes);
    assert_eq!(proposal_votes.abstain, merry_votes);

    // close the proposal
    e.jump(settings.vote_period - 3 * ONE_HOUR);
    governor_client.close(&proposal_id);

    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Successful);

    // execute the proposal
    e.jump(settings.timelock);
    assert_eq!(token_client.balance(&governor_address), gov_balance);
    assert_eq!(token_client.balance(&subcall_address), 0);

    governor_client.execute(&proposal_id);

    assert_eq!(
        token_client.balance(&governor_address),
        gov_balance - call_amount
    );
    assert_eq!(token_client.balance(&subcall_address), call_amount);
}

#[test]
fn test_wasm_happy_path_soroban_token() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    e.budget().reset_unlimited();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let samwise = Address::generate(&e);
    let pippin = Address::generate(&e);
    let merry = Address::generate(&e);

    let settings = default_governor_settings(&e);
    let (token_address, token_client) = create_stellar_token(&e, &bombadil);
    let (governor_address, votes_address) = create_soroban_governor_wasm(&e, &bombadil, &settings);
    let votes_client = TokenVotesClient::new(&e, &votes_address);
    let governor_client = GovernorContractClient::new(&e, &governor_address);
    let (subcall_address, _) =
        create_mock_subcall_contract_wasm(&e, &token_address, &governor_address);

    let gov_balance: i128 = 123 * 10i128.pow(7);
    token_client.mint(&governor_address, &gov_balance);

    // set intial votes
    let mut frodo_votes: i128 = 10_000 * 10i128.pow(7);
    votes_client.mint(&frodo, &frodo_votes);

    let mut samwise_votes = 5_000 * 10i128.pow(7);
    votes_client.mint(&samwise, &samwise_votes);

    let pippin_votes = 9_000 * 10i128.pow(7);
    votes_client.mint(&pippin, &pippin_votes);

    let mut total_votes = frodo_votes + samwise_votes + pippin_votes;

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

    // pass some time - samwise delegates to frodo then transfers votes to merry. Merry mints more votes.
    e.jump(settings.vote_delay - ONE_HOUR);

    votes_client.delegate(&samwise, &frodo);
    frodo_votes += samwise_votes;
    samwise_votes = 0;

    votes_client.transfer(&samwise, &merry, &(1000 * 10i128.pow(7)));
    frodo_votes -= 1000 * 10i128.pow(7);
    let mut merry_votes = 1000 * 10i128.pow(7);

    merry_votes += 2_000 * 10i128.pow(7);
    votes_client.mint(&merry, &(2_000 * 10i128.pow(7)));
    total_votes += 2_000 * 10i128.pow(7);

    assert_eq!(votes_client.get_votes(&frodo), frodo_votes);
    assert_eq!(votes_client.get_votes(&samwise), samwise_votes);
    assert_eq!(votes_client.get_votes(&merry), merry_votes);
    assert_eq!(votes_client.get_votes(&pippin), pippin_votes);
    assert_eq!(votes_client.total_supply(), total_votes);

    // start the vote period
    e.jump(ONE_HOUR + 1);

    // merry tries to delegate votes to pippin after the delay and mint more votes
    // @dev: don't update vote trackers with changes after the vote period starts
    votes_client.delegate(&merry, &pippin);
    let late_mint_votes = 500 * 10i128.pow(7);
    votes_client.mint(&merry, &late_mint_votes);
    assert_eq!(votes_client.get_votes(&merry), 0);
    assert_eq!(
        votes_client.get_votes(&pippin),
        pippin_votes + merry_votes + late_mint_votes
    );
    assert_eq!(votes_client.total_supply(), total_votes + late_mint_votes);

    // frodo votes for, pippin votes against, merry abstains
    governor_client.vote(&frodo, &proposal_id, &1);
    e.jump(ONE_HOUR);
    governor_client.vote(&pippin, &proposal_id, &0);
    e.jump(ONE_HOUR);
    governor_client.vote(&merry, &proposal_id, &2);
    e.jump(ONE_HOUR);

    let proposal_votes = governor_client.get_proposal_votes(&proposal_id);
    assert_eq!(proposal_votes.against, pippin_votes);
    assert_eq!(proposal_votes._for, frodo_votes);
    assert_eq!(proposal_votes.abstain, merry_votes);

    // close the proposal
    e.jump(settings.vote_period - 3 * ONE_HOUR);
    governor_client.close(&proposal_id);

    let proposal = governor_client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.data.status, ProposalStatus::Successful);

    // execute the proposal
    e.jump(settings.timelock);
    assert_eq!(token_client.balance(&governor_address), gov_balance);
    assert_eq!(token_client.balance(&subcall_address), 0);

    governor_client.execute(&proposal_id);

    assert_eq!(
        token_client.balance(&governor_address),
        gov_balance - call_amount
    );
    assert_eq!(token_client.balance(&subcall_address), call_amount);
}
