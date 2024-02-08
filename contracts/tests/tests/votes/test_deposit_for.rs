#[cfg(test)]
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    vec, Address, Env, IntoVal, Symbol, Val,
};
use tests::{common::create_stellar_token, env::EnvTestUtils, votes::create_token_votes};
#[test]
fn test_deposit_for() {
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

    // validate auth
    assert_eq!(
        e.auths()[0],
        (
            samwise.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    votes_id.clone(),
                    Symbol::new(&e, "deposit_for"),
                    vec![&e, samwise.to_val(), deposit_amount.into_val(&e),]
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        token_id.clone(),
                        Symbol::new(&e, "transfer"),
                        vec![
                            &e,
                            samwise.to_val(),
                            votes_id.to_val(),
                            deposit_amount.into_val(&e)
                        ]
                    )),
                    sub_invocations: std::vec![]
                }]
            }
        )
    );

    // validate chain results
    assert_eq!(votes_client.balance(&samwise), deposit_amount);
    assert_eq!(votes_client.total_supply(), deposit_amount);
    assert_eq!(votes_client.get_votes(&samwise), deposit_amount);
    assert_eq!(
        votes_client.get_past_votes(&samwise, &(e.ledger().timestamp())),
        deposit_amount
    );
    assert_eq!(
        token_client.balance(&samwise),
        initial_balance - deposit_amount
    );
    assert_eq!(token_client.balance(&votes_id), deposit_amount);

    // validate events
    let events = e.events().all();
    let tx_events = events.slice((events.len() - 2)..(events.len()));
    let event_data_0: soroban_sdk::Vec<Val> =
        vec![&e, 0i128.into_val(&e), deposit_amount.into_val(&e)];
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
                (Symbol::new(&e, "deposit"), samwise.clone()).into_val(&e),
                deposit_amount.into_val(&e)
            )
        ]
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_deposit_for_negative_amount() {
    let e = Env::default();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);

    let (token_id, token_client) = create_stellar_token(&e, &bombadil);
    let (_, votes_client) = create_token_votes(&e, &token_id);

    let initial_balance = 100_000 * 10i128.pow(7);
    token_client.mint(&samwise, &initial_balance);

    let deposit_amount: i128 = -1;
    votes_client.deposit_for(&samwise, &deposit_amount);
}
