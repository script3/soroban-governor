#[cfg(test)]
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    vec, Address, Env, IntoVal, Symbol, Val,
};
use tests::{env::EnvTestUtils, votes::create_soroban_token_votes_wasm};

#[test]
fn test_mint_soroban() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let governor = Address::generate(&e);

    let (votes_id, votes_client) = create_soroban_token_votes_wasm(&e, &bombadil, &governor);

    let deposit_amount = 123_7654321;
    votes_client.mint(&samwise, &deposit_amount);

    // validate auth
    assert_eq!(
        e.auths()[0],
        (
            bombadil.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    votes_id.clone(),
                    Symbol::new(&e, "mint"),
                    vec![&e, samwise.to_val(), deposit_amount.into_val(&e),]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    // validate chain results
    assert_eq!(votes_client.balance(&samwise), deposit_amount);
    assert_eq!(votes_client.total_supply(), deposit_amount);
    assert_eq!(votes_client.get_votes(&samwise), deposit_amount);

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
                (Symbol::new(&e, "mint"), bombadil.clone(), samwise.clone()).into_val(&e),
                deposit_amount.into_val(&e)
            )
        ]
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_mint_negative_amount_soroban() {
    let e = Env::default();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let governor = Address::generate(&e);

    let (_, votes_client) = create_soroban_token_votes_wasm(&e, &bombadil, &governor);

    let deposit_amount: i128 = -1;
    votes_client.mint(&samwise, &deposit_amount);
}

#[test]
fn test_set_admin() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let governor = Address::generate(&e);

    let (votes_id, votes_client) = create_soroban_token_votes_wasm(&e, &bombadil, &governor);

    votes_client.set_admin(&samwise);

    // validate auth
    assert_eq!(
        e.auths()[0],
        (
            bombadil.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    votes_id.clone(),
                    Symbol::new(&e, "set_admin"),
                    vec![&e, samwise.to_val(),]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    // validate chain results
    assert_eq!(votes_client.admin(), samwise);

    // validate events
    let events = e.events().all();
    let tx_events = vec![&e, events.last().unwrap()];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                votes_id.clone(),
                (Symbol::new(&e, "set_admin"), bombadil.clone()).into_val(&e),
                samwise.into_val(&e)
            ),
        ]
    );
}
