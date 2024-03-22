#[cfg(test)]
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    vec, Address, Env, IntoVal, Symbol,
};
use tests::{
    common::create_stellar_token, env::EnvTestUtils, votes::create_staking_token_votes_wasm,
};

#[test]
fn test_emissions() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let frodo = Address::generate(&e);
    let governor = Address::generate(&e);

    let (token_id, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_id, votes_client) = create_staking_token_votes_wasm(&e, &token_id, &governor);

    let mut balance_samwise = 0;
    let mut balance_frodo = 0;
    let initial_balance = 1 * 10i128.pow(7);
    token_client.mint(&samwise, &initial_balance);
    votes_client.deposit(&samwise, &initial_balance);
    balance_samwise += initial_balance;

    let t_start = e.ledger().timestamp();
    let t_emission_end = t_start + 100_000;
    let tokens_to_emit = 100_000 * 10i128.pow(7);

    token_client.mint(&governor, &tokens_to_emit);

    // start emissions - emit 100k tokens over 100k seconds
    votes_client.set_emis(&tokens_to_emit, &t_emission_end);

    // validate auth
    assert_eq!(
        e.auths()[0],
        (
            governor.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    votes_id.clone(),
                    Symbol::new(&e, "set_emis"),
                    vec![&e, tokens_to_emit.into_val(&e), t_emission_end.into_val(&e),]
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        token_id.clone(),
                        Symbol::new(&e, "transfer"),
                        vec![
                            &e,
                            governor.to_val(),
                            votes_id.to_val(),
                            tokens_to_emit.into_val(&e)
                        ]
                    )),
                    sub_invocations: std::vec![]
                }]
            }
        )
    );

    // validate events
    let events = e.events().all();
    let tx_events = vec![&e, events.last().unwrap()];
    // let event_data_0: soroban_sdk::Vec<Val> =
    //     vec![&e, 1_0000000i128.into_val(&e), tokens_to_emit.into_val(&e)];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                votes_id.clone(),
                (
                    Symbol::new(&e, "set_emissions"),
                    1_0000000u64,
                    t_emission_end
                )
                    .into_val(&e),
                ().into_val(&e)
            ),
        ]
    );

    // skip 25k seconds (1/4). All emissions will go to samwise
    e.jump(25000 / 5);
    balance_samwise += 25_000 * 10i128.pow(7);

    // frodo deposits an equal amount as samwise
    token_client.mint(&frodo, &initial_balance);
    votes_client.deposit(&frodo, &initial_balance);
    balance_frodo += initial_balance;

    // skip 25k seconds (2/4). Emissions will be split between samwise and frodo
    e.jump(25000 / 5);
    balance_samwise += 12_500 * 10i128.pow(7);
    balance_frodo += 12_500 * 10i128.pow(7);

    // samwise withdraws half of his balance, and transfers it to frodo
    // and frodo deposits it
    votes_client.withdraw(&samwise, &(initial_balance / 2));
    token_client.transfer(&samwise, &frodo, &(initial_balance / 2));
    votes_client.deposit(&frodo, &(initial_balance / 2));
    balance_samwise -= initial_balance / 2;
    balance_frodo += initial_balance / 2;

    // skip 25k seconds (3/4). Emissions will be split at 25% sawise and 75% frodo
    e.jump(25000 / 5);
    balance_samwise += 6_250 * 10i128.pow(7);
    balance_frodo += 18_750 * 10i128.pow(7);

    // samwise claims his emissions and withdraws his balance
    let claimed = votes_client.claim(&samwise);

    // validate auth
    assert_eq!(
        e.auths()[0],
        (
            samwise.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    votes_id.clone(),
                    Symbol::new(&e, "claim"),
                    vec![&e, samwise.into_val(&e),]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    // validate events
    let events = e.events().all();
    let tx_events = vec![&e, events.last().unwrap()];
    assert_eq!(
        tx_events,
        vec![
            &e,
            (
                votes_id.clone(),
                (Symbol::new(&e, "claim"), samwise.clone()).into_val(&e),
                claimed.into_val(&e)
            ),
        ]
    );

    // withdraw and validate chain results
    assert_eq!(claimed, balance_samwise - initial_balance / 2); // still has half of initial deposit
    votes_client.withdraw(&samwise, &balance_samwise);
    assert_eq!(token_client.balance(&samwise), balance_samwise);

    // skip 30k seconds (4/4 + some). Last of emissions will be split at 0% sawise and 100% frodo
    e.jump(30000 / 5);
    balance_frodo += 25_000 * 10i128.pow(7);

    // frodo claim and validate chain results
    let claimed = votes_client.claim(&frodo);
    // got half of samwise and all of his initial deposit
    // this gets floored during final claim
    assert_eq!(
        claimed,
        balance_frodo - initial_balance - initial_balance / 2 - 1
    );
    assert_eq!(votes_client.balance(&frodo), balance_frodo - 1);
}
