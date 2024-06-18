#[cfg(test)]
use soroban_sdk::{testutils::Address as _, Address, Env, Error};
use tests::{
    common::create_stellar_token,
    env::EnvTestUtils,
    votes::{create_bonding_token_votes, create_soroban_token_votes_wasm},
    ONE_DAY_LEDGERS,
};

#[test]
fn test_get_past() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let frodo = Address::generate(&e);
    let pippin = Address::generate(&e);
    let governor = Address::generate(&e);

    let (token_id, token_client) = create_stellar_token(&e, &bombadil);
    let (_, votes_client) = create_bonding_token_votes(&e, &token_id, &governor);

    // setup vote ledgers - do a ledger before each action to verify the actions
    // occuring after the vote starts are recorded properly
    let cur_ledger = e.ledger().sequence();
    votes_client.set_vote_sequence(&(cur_ledger + 99));
    votes_client.set_vote_sequence(&(cur_ledger + 199));
    votes_client.set_vote_sequence(&(cur_ledger + 299));

    let initial_balance = 100_000 * 10i128.pow(7);
    token_client.mint(&frodo, &initial_balance);
    token_client.mint(&samwise, &initial_balance);
    token_client.mint(&pippin, &initial_balance);

    let deposit_amount_frodo = 1_000 * 10i128.pow(7);
    votes_client.deposit(&frodo, &deposit_amount_frodo);

    let deposit_amount_samwise = 250 * 10i128.pow(7);
    votes_client.deposit(&samwise, &deposit_amount_samwise);

    assert_eq!(
        votes_client.total_supply(),
        deposit_amount_frodo + deposit_amount_samwise
    );

    e.jump(100);

    // transfer some tokens to verify that the total supply remains constant
    let transfer_amount = 100 * 10i128.pow(7);
    votes_client.withdraw(&frodo, &transfer_amount);
    token_client.transfer(&frodo, &pippin, &transfer_amount);
    votes_client.deposit(&pippin, &transfer_amount);

    votes_client.withdraw(&samwise, &transfer_amount);
    token_client.transfer(&samwise, &pippin, &transfer_amount);
    votes_client.deposit(&pippin, &transfer_amount);

    assert_eq!(
        votes_client.total_supply(),
        deposit_amount_frodo + deposit_amount_samwise
    );
    assert_eq!(
        votes_client.balance(&pippin),
        transfer_amount + transfer_amount
    );
    assert_eq!(token_client.balance(&pippin), initial_balance);

    e.jump(100);

    // withdraw some tokens
    let withdraw_amount = 75 * 10i128.pow(7);

    votes_client.withdraw(&pippin, &withdraw_amount);

    assert_eq!(
        votes_client.total_supply(),
        deposit_amount_frodo + deposit_amount_samwise - withdraw_amount
    );
    assert_eq!(
        votes_client.balance(&pippin),
        transfer_amount + transfer_amount - withdraw_amount
    );
    assert_eq!(
        token_client.balance(&pippin),
        initial_balance + withdraw_amount
    );

    e.jump(100);

    // deposit tokens
    let deposit_amount_pippin = 50_000 * 10i128.pow(7);

    votes_client.deposit(&pippin, &deposit_amount_pippin);

    // verify current values
    assert_eq!(
        votes_client.total_supply(),
        deposit_amount_frodo + deposit_amount_samwise - withdraw_amount + deposit_amount_pippin
    );
    assert_eq!(
        votes_client.balance(&pippin),
        transfer_amount + transfer_amount - withdraw_amount + deposit_amount_pippin
    );
    assert_eq!(
        token_client.balance(&pippin),
        initial_balance + withdraw_amount - deposit_amount_pippin
    );
    assert_eq!(
        votes_client.balance(&samwise),
        deposit_amount_samwise - transfer_amount
    );
    assert_eq!(
        votes_client.balance(&frodo),
        deposit_amount_frodo - transfer_amount
    );

    // verify past total supply
    assert_eq!(
        votes_client.get_past_total_supply(&(e.ledger().sequence() - 201)),
        deposit_amount_frodo + deposit_amount_samwise
    );
    assert_eq!(
        votes_client.get_past_total_supply(&(e.ledger().sequence() - 101)),
        deposit_amount_frodo + deposit_amount_samwise
    );
    assert_eq!(
        votes_client.get_past_total_supply(&(e.ledger().sequence() - 1)),
        deposit_amount_frodo + deposit_amount_samwise - withdraw_amount
    );

    // verify past votes for pippen
    assert_eq!(
        votes_client.get_past_votes(&pippin, &(e.ledger().sequence() - 201)),
        0
    );
    assert_eq!(
        votes_client.get_past_votes(&pippin, &(e.ledger().sequence() - 101)),
        transfer_amount + transfer_amount
    );
    assert_eq!(
        votes_client.get_past_votes(&pippin, &(e.ledger().sequence() - 1)),
        transfer_amount + transfer_amount - withdraw_amount
    );

    // verify past votes for samwise
    assert_eq!(
        votes_client.get_past_votes(&samwise, &(e.ledger().sequence() - 201)),
        deposit_amount_samwise
    );
    assert_eq!(
        votes_client.get_past_votes(&samwise, &(e.ledger().sequence() - 101)),
        deposit_amount_samwise - transfer_amount
    );
    assert_eq!(
        votes_client.get_past_votes(&samwise, &(e.ledger().sequence() - 1)),
        deposit_amount_samwise - transfer_amount
    );
}

#[test]
fn test_get_past_same_sequence_as_ledger() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let frodo = Address::generate(&e);
    let pippin = Address::generate(&e);
    let governor = Address::generate(&e);

    let (token_id, token_client) = create_stellar_token(&e, &bombadil);
    let (_, votes_client) = create_bonding_token_votes(&e, &token_id, &governor);

    let cur_ledger = e.ledger().sequence();
    votes_client.set_vote_sequence(&(cur_ledger + 99));

    let initial_balance = 100_000 * 10i128.pow(7);
    token_client.mint(&frodo, &initial_balance);
    token_client.mint(&samwise, &initial_balance);
    token_client.mint(&pippin, &initial_balance);

    let deposit_amount_frodo = 1_000 * 10i128.pow(7);
    votes_client.deposit(&frodo, &deposit_amount_frodo);

    let deposit_amount_samwise = 250 * 10i128.pow(7);
    votes_client.deposit(&samwise, &deposit_amount_samwise);

    e.jump(10);

    assert_eq!(
        votes_client.total_supply(),
        deposit_amount_frodo + deposit_amount_samwise
    );
    assert_eq!(votes_client.get_votes(&frodo), deposit_amount_frodo);
    assert_eq!(votes_client.get_votes(&samwise), deposit_amount_samwise);

    assert_eq!(
        votes_client
            .try_get_past_total_supply(&e.ledger().sequence())
            .err(),
        Some(Ok(Error::from_contract_error(103)))
    );
    assert_eq!(
        votes_client
            .try_get_past_votes(&frodo, &e.ledger().sequence())
            .err(),
        Some(Ok(Error::from_contract_error(103)))
    );
    assert_eq!(
        votes_client
            .try_get_past_votes(&samwise, &e.ledger().sequence())
            .err(),
        Some(Ok(Error::from_contract_error(103)))
    );
}

#[test]
fn test_past_checkpoints_get_pruned() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();
    e.budget().reset_unlimited();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let frodo = Address::generate(&e);
    let pippin = Address::generate(&e);
    let governor = Address::generate(&e);

    let (_, votes_client) = create_soroban_token_votes_wasm(&e, &bombadil, &governor);

    // setup vote ledgers - do a ledger before each action to verify the actions
    // occuring after the vote starts are recorded properly
    let cur_ledger = e.ledger().sequence();
    let start_vote_0 = cur_ledger + ONE_DAY_LEDGERS - 1;
    let start_vote_1 = cur_ledger + 3 * ONE_DAY_LEDGERS - 1;
    let start_vote_2 = cur_ledger + 15 * ONE_DAY_LEDGERS - 1;
    let start_vote_3 = cur_ledger + 16 * ONE_DAY_LEDGERS - 1;
    votes_client.set_vote_sequence(&start_vote_0);
    votes_client.set_vote_sequence(&start_vote_1);
    votes_client.set_vote_sequence(&start_vote_2);
    votes_client.set_vote_sequence(&start_vote_3);

    // Time = 10 days ago
    // Time = 16 days ago
    let deposit_amount_frodo = 1_000 * 10i128.pow(7);
    votes_client.mint(&frodo, &deposit_amount_frodo);

    e.jump(ONE_DAY_LEDGERS);
    // Time = 9 days ago (vote 0 passed by 1 ledger)
    // Time = 15 days ago (vote 0 passed by 1 ledger)

    let deposit_amount_samwise = 250 * 10i128.pow(7);
    votes_client.mint(&samwise, &deposit_amount_samwise);

    let transfer_1_amount = 100 * 10i128.pow(7);
    votes_client.transfer(&samwise, &frodo, &transfer_1_amount);

    e.jump(2 * ONE_DAY_LEDGERS);
    // Time = 6 days ago (vote 1 passed by 1 ledger)
    // Time = 13 days ago (vote 1 passed by 1 ledger)

    let deposit_amount_pippin = 5_000 * 10i128.pow(7);
    votes_client.mint(&pippin, &deposit_amount_pippin);

    e.jump(12 * ONE_DAY_LEDGERS);
    // Time = 2 days ago (vote 2 passed by 1 ledger)
    // Time = 1 days ago (vote 2 passed by 1 ledger)

    let transfer_2_amount = 125 * 10i128.pow(7);
    votes_client.transfer(&pippin, &frodo, &transfer_2_amount);

    e.jump(ONE_DAY_LEDGERS);
    // Time = now (vote 3 passed by 1 ledger)
    // set a vote ledger to cause the
    votes_client.set_vote_sequence(&(e.ledger().sequence() + 2 * ONE_DAY_LEDGERS));

    // verify to be pruned values
    assert_eq!(
        votes_client.get_past_total_supply(&start_vote_0),
        deposit_amount_frodo
    );
    assert_eq!(
        votes_client.get_past_votes(&frodo, &start_vote_0),
        deposit_amount_frodo
    );
    assert_eq!(votes_client.get_past_votes(&samwise, &start_vote_0), 0);
    assert_eq!(votes_client.get_past_votes(&pippin, &start_vote_0), 0);

    // -> results in pruning
    let transfer_3_amount = 75 * 10i128.pow(7);
    votes_client.transfer(&pippin, &frodo, &transfer_3_amount);

    let deposit_2_amount_samwise = 50 * 10i128.pow(7);
    votes_client.mint(&samwise, &deposit_2_amount_samwise);

    let max_vote_period_check = e.ledger().sequence() - 14 * ONE_DAY_LEDGERS;

    // verify current values
    assert_eq!(
        votes_client.total_supply(),
        deposit_amount_frodo
            + deposit_amount_samwise
            + deposit_amount_pippin
            + deposit_2_amount_samwise
    );
    assert_eq!(
        votes_client.get_votes(&frodo),
        deposit_amount_frodo + transfer_1_amount + transfer_2_amount + transfer_3_amount
    );
    assert_eq!(
        votes_client.get_votes(&samwise),
        deposit_amount_samwise - transfer_1_amount + deposit_2_amount_samwise
    );
    assert_eq!(
        votes_client.get_votes(&pippin),
        deposit_amount_pippin - transfer_2_amount - transfer_3_amount
    );

    // verify past total supply
    assert_eq!(votes_client.get_past_total_supply(&start_vote_0), 0); // pruned
    assert_eq!(
        votes_client.get_past_total_supply(&max_vote_period_check),
        deposit_amount_frodo + deposit_amount_samwise
    );
    assert_eq!(
        votes_client.get_past_total_supply(&start_vote_1),
        deposit_amount_frodo + deposit_amount_samwise
    );
    assert_eq!(
        votes_client.get_past_total_supply(&start_vote_2),
        deposit_amount_frodo + deposit_amount_samwise + deposit_amount_pippin
    );
    assert_eq!(
        votes_client.get_past_total_supply(&start_vote_3),
        deposit_amount_frodo + deposit_amount_samwise + deposit_amount_pippin
    );

    // verify past frodo votes
    assert_eq!(votes_client.get_past_votes(&frodo, &start_vote_0), 0); // pruned
    assert_eq!(
        votes_client.get_past_votes(&frodo, &max_vote_period_check),
        deposit_amount_frodo + transfer_1_amount
    );
    assert_eq!(
        votes_client.get_past_votes(&frodo, &start_vote_1),
        deposit_amount_frodo + transfer_1_amount
    );
    assert_eq!(
        votes_client.get_past_votes(&frodo, &start_vote_2),
        deposit_amount_frodo + transfer_1_amount
    );
    assert_eq!(
        votes_client.get_past_votes(&frodo, &start_vote_3),
        deposit_amount_frodo + transfer_1_amount + transfer_2_amount
    );

    // verify past samwise votes
    assert_eq!(votes_client.get_past_votes(&samwise, &start_vote_0), 0); // pruned
    assert_eq!(
        votes_client.get_past_votes(&samwise, &max_vote_period_check),
        deposit_amount_samwise - transfer_1_amount
    );
    assert_eq!(
        votes_client.get_past_votes(&samwise, &start_vote_1),
        deposit_amount_samwise - transfer_1_amount
    );
    assert_eq!(
        votes_client.get_past_votes(&samwise, &start_vote_2),
        deposit_amount_samwise - transfer_1_amount
    );
    assert_eq!(
        votes_client.get_past_votes(&samwise, &start_vote_3),
        deposit_amount_samwise - transfer_1_amount
    );

    // verify past pippin votes
    assert_eq!(votes_client.get_past_votes(&pippin, &start_vote_0), 0);
    assert_eq!(
        votes_client.get_past_votes(&pippin, &max_vote_period_check),
        0
    );
    assert_eq!(votes_client.get_past_votes(&pippin, &start_vote_1), 0);
    assert_eq!(
        votes_client.get_past_votes(&pippin, &start_vote_2),
        deposit_amount_pippin
    );
    assert_eq!(
        votes_client.get_past_votes(&pippin, &start_vote_3),
        deposit_amount_pippin - transfer_2_amount
    );
}

#[test]
fn test_get_past_transfer_same_user() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();
    e.budget().reset_unlimited();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let frodo = Address::generate(&e);
    let pippin = Address::generate(&e);
    let merry = Address::generate(&e);
    let governor = Address::generate(&e);

    let (_, token_client) = create_soroban_token_votes_wasm(&e, &bombadil, &governor);

    let vote_ledger = e.ledger().sequence() + 99;
    token_client.set_vote_sequence(&vote_ledger);

    let initial_balance = 100 * 10i128.pow(7);
    token_client.mint(&frodo, &initial_balance);
    token_client.mint(&samwise, &initial_balance);
    token_client.mint(&pippin, &initial_balance);
    token_client.mint(&merry, &initial_balance);

    token_client.delegate(&pippin, &samwise);

    assert_eq!(token_client.get_votes(&frodo), initial_balance);
    assert_eq!(token_client.get_votes(&samwise), initial_balance * 2);
    assert_eq!(token_client.get_votes(&pippin), 0);
    assert_eq!(token_client.get_votes(&merry), initial_balance);

    e.jump(10);

    // no checkpoint recorded without actual token changes
    token_client.transfer(&frodo, &frodo, &100);
    token_client.transfer(&pippin, &pippin, &100);
    token_client.transfer(&merry, &bombadil, &0);

    assert_eq!(token_client.get_votes(&frodo), initial_balance);
    assert_eq!(token_client.get_votes(&samwise), initial_balance * 2);
    assert_eq!(token_client.get_votes(&pippin), 0);
    assert_eq!(token_client.get_votes(&merry), initial_balance);

    // verify vote balances were not written
    assert_eq!(
        token_client.get_past_votes(&frodo, &(e.ledger().sequence() - 1)),
        initial_balance
    );
    assert_eq!(
        token_client.get_past_votes(&samwise, &(e.ledger().sequence() - 1)),
        initial_balance * 2
    );
    assert_eq!(
        token_client.get_past_votes(&pippin, &(e.ledger().sequence() - 1)),
        0
    );
    assert_eq!(
        token_client.get_past_votes(&merry, &(e.ledger().sequence() - 1)),
        initial_balance
    );
}
