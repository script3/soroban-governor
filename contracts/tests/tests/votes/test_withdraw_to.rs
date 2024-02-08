#[cfg(test)]
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    vec, Address, Env, IntoVal, Symbol, Val,
};
use tests::{common::create_stellar_token, env::EnvTestUtils, votes::create_token_votes};

#[test]
fn test_withdraw_to() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);

    let (token_id, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_id, votes_client) = create_token_votes(&e, &token_id);

    let initial_balance = 100_000 * 10i128.pow(7);
    token_client.mint(&samwise, &initial_balance);

    let deposit_amount = 123_7654321;
    votes_client.deposit_for(&samwise, &deposit_amount);

    e.jump_with_sequence(1000);

    let withdraw_amount = 100 * 10i128.pow(7);
    votes_client.withdraw_to(&samwise, &withdraw_amount);

    // validate auth
    assert_eq!(
        e.auths()[0],
        (
            samwise.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    votes_id.clone(),
                    Symbol::new(&e, "withdraw_to"),
                    vec![&e, samwise.to_val(), withdraw_amount.into_val(&e),]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    // validate chain results
    assert_eq!(
        votes_client.balance(&samwise),
        deposit_amount - withdraw_amount
    );
    assert_eq!(
        votes_client.total_supply(),
        deposit_amount - withdraw_amount
    );
    assert_eq!(
        votes_client.get_votes(&samwise),
        deposit_amount - withdraw_amount
    );
    assert_eq!(
        votes_client.get_past_votes(&samwise, &(e.ledger().timestamp())),
        deposit_amount - withdraw_amount
    );
    assert_eq!(
        votes_client.get_past_votes(&samwise, &(e.ledger().timestamp() - 1)),
        deposit_amount
    );
    assert_eq!(
        token_client.balance(&samwise),
        initial_balance - deposit_amount + withdraw_amount
    );
    assert_eq!(
        token_client.balance(&votes_id),
        deposit_amount - withdraw_amount
    );

    // validate events
    let events = e.events().all();
    let tx_events = vec![
        &e,
        events.get_unchecked(events.len() - 3),
        events.last().unwrap(),
    ];
    let event_data_0: soroban_sdk::Vec<Val> = vec![
        &e,
        deposit_amount.into_val(&e),
        (deposit_amount - withdraw_amount).into_val(&e),
    ];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                votes_id.clone(),
                (Symbol::new(&e, "votes_changed"), samwise.clone()).into_val(&e),
                event_data_0.into_val(&e)
            ),
            (
                votes_id.clone(),
                (Symbol::new(&e, "withdraw"), samwise.clone()).into_val(&e),
                withdraw_amount.into_val(&e)
            )
        ]
    );
}

#[test]
fn test_withdraw_to_full_balance() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);

    let (token_id, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_id, votes_client) = create_token_votes(&e, &token_id);

    let initial_balance = 100_000 * 10i128.pow(7);
    token_client.mint(&samwise, &initial_balance);

    let deposit_amount = 123_7654321;
    votes_client.deposit_for(&samwise, &deposit_amount);

    e.jump_with_sequence(1000);

    votes_client.withdraw_to(&samwise, &deposit_amount);

    assert_eq!(votes_client.balance(&samwise), 0);
    assert_eq!(votes_client.total_supply(), 0);
    assert_eq!(votes_client.get_votes(&samwise), 0);
    assert_eq!(token_client.balance(&samwise), initial_balance);
    assert_eq!(token_client.balance(&votes_id), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_withdraw_to_negative_amount() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);

    let (token_id, token_client) = create_stellar_token(&e, &bombadil);
    let (_, votes_client) = create_token_votes(&e, &token_id);

    let initial_balance = 100_000 * 10i128.pow(7);
    token_client.mint(&samwise, &initial_balance);

    let deposit_amount = 123_7654321;
    votes_client.deposit_for(&samwise, &deposit_amount);

    e.jump_with_sequence(1000);

    let withdraw_amount = -1 * 10i128.pow(7);
    votes_client.withdraw_to(&samwise, &withdraw_amount);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_withdraw_to_more_than_balance() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);

    let (token_id, token_client) = create_stellar_token(&e, &bombadil);
    let (_, votes_client) = create_token_votes(&e, &token_id);

    let initial_balance = 100_000 * 10i128.pow(7);
    token_client.mint(&samwise, &initial_balance);

    let deposit_amount = 123_7654321;
    votes_client.deposit_for(&samwise, &deposit_amount);

    e.jump_with_sequence(1000);

    let withdraw_amount = deposit_amount + 1;
    votes_client.withdraw_to(&samwise, &withdraw_amount);
}
