#[cfg(test)]
use soroban_sdk::{testutils::Address as _, Address, Env};
use tests::{common::create_stellar_token, env::EnvTestUtils, votes::create_token_votes};

#[test]
fn test_get_past() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let frodo = Address::generate(&e);
    let pippin = Address::generate(&e);

    let (token_id, token_client) = create_stellar_token(&e, &bombadil);
    let (_, votes_client) = create_token_votes(&e, &token_id);

    let initial_balance = 100_000 * 10i128.pow(7);
    token_client.mint(&frodo, &initial_balance);
    token_client.mint(&samwise, &initial_balance);
    token_client.mint(&pippin, &initial_balance);

    let deposit_amount_frodo = 1_000 * 10i128.pow(7);
    votes_client.deposit_for(&frodo, &deposit_amount_frodo);

    let deposit_amount_samwise = 250 * 10i128.pow(7);
    votes_client.deposit_for(&samwise, &deposit_amount_samwise);

    assert_eq!(
        votes_client.total_supply(),
        deposit_amount_frodo + deposit_amount_samwise
    );

    e.jump_with_sequence(100);

    // transfer some tokens to verify that the total supply remains constant
    let transfer_amount = 100 * 10i128.pow(7);
    votes_client.transfer(&frodo, &pippin, &transfer_amount);
    votes_client.transfer(&samwise, &pippin, &transfer_amount);

    assert_eq!(
        votes_client.total_supply(),
        deposit_amount_frodo + deposit_amount_samwise
    );
    assert_eq!(
        votes_client.balance(&pippin),
        transfer_amount + transfer_amount
    );
    assert_eq!(token_client.balance(&pippin), initial_balance);

    e.jump_with_sequence(100);

    // withdraw some tokens
    let withdraw_amount = 75 * 10i128.pow(7);

    votes_client.withdraw_to(&pippin, &withdraw_amount);

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

    e.jump_with_sequence(100);

    // deposit tokens
    let deposit_amount_pippin = 50_000 * 10i128.pow(7);

    votes_client.deposit_for(&pippin, &deposit_amount_pippin);

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

    // verify past total supply
    assert_eq!(
        votes_client.get_past_total_supply(&(e.ledger().timestamp() - 201)),
        deposit_amount_frodo + deposit_amount_samwise
    );
    assert_eq!(
        votes_client.get_past_total_supply(&(e.ledger().timestamp() - 101)),
        deposit_amount_frodo + deposit_amount_samwise
    );
    assert_eq!(
        votes_client.get_past_total_supply(&(e.ledger().timestamp() - 1)),
        deposit_amount_frodo + deposit_amount_samwise - withdraw_amount
    );
    assert_eq!(
        votes_client.get_past_total_supply(&e.ledger().timestamp()),
        deposit_amount_frodo + deposit_amount_samwise - withdraw_amount + deposit_amount_pippin
    );

    // verify past votes for pippen
    assert_eq!(
        votes_client.get_past_votes(&pippin, &(e.ledger().timestamp() - 201)),
        0
    );
    assert_eq!(
        votes_client.get_past_votes(&pippin, &(e.ledger().timestamp() - 101)),
        transfer_amount + transfer_amount
    );
    assert_eq!(
        votes_client.get_past_votes(&pippin, &(e.ledger().timestamp() - 1)),
        transfer_amount + transfer_amount - withdraw_amount
    );
    assert_eq!(
        votes_client.get_past_votes(&pippin, &e.ledger().timestamp()),
        transfer_amount + transfer_amount - withdraw_amount + deposit_amount_pippin
    );

    // verify past votes for samwise
    assert_eq!(
        votes_client.get_past_votes(&samwise, &(e.ledger().timestamp() - 201)),
        deposit_amount_samwise
    );
    assert_eq!(
        votes_client.get_past_votes(&samwise, &(e.ledger().timestamp() - 101)),
        deposit_amount_samwise - transfer_amount
    );
    assert_eq!(
        votes_client.get_past_votes(&samwise, &(e.ledger().timestamp() - 1)),
        deposit_amount_samwise - transfer_amount
    );
    assert_eq!(
        votes_client.get_past_votes(&samwise, &e.ledger().timestamp()),
        deposit_amount_samwise - transfer_amount
    );
}
