#[cfg(test)]
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events},
    vec, Address, Env, Error, IntoVal, Symbol, Val,
};
use tests::{
    common::create_stellar_token,
    env::EnvTestUtils,
    votes::{create_bonding_token_votes, create_soroban_token_votes_wasm},
};

#[test]
fn test_delegation() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let frodo = Address::generate(&e);
    let governor = Address::generate(&e);

    let (token_id, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_id, votes_client) = create_bonding_token_votes(&e, &token_id, &governor);

    votes_client.set_vote_sequence(&(e.ledger().sequence() + 100 - 1));

    let initial_balance = 100_000 * 10i128.pow(7);
    token_client.mint(&samwise, &initial_balance);
    token_client.mint(&frodo, &initial_balance);

    let initial_amount_frodo = 50 * 10i128.pow(7);
    votes_client.deposit(&frodo, &initial_amount_frodo);

    let initial_amount_samwise = 100 * 10i128.pow(7);
    votes_client.deposit(&samwise, &initial_amount_samwise);

    e.jump(100);

    votes_client.delegate(&samwise, &frodo);

    // validate auth
    assert_eq!(
        e.auths()[0],
        (
            samwise.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    votes_id.clone(),
                    Symbol::new(&e, "delegate"),
                    vec![&e, samwise.to_val(), frodo.to_val(),]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    // validate chain results
    assert_eq!(votes_client.get_delegate(&samwise), frodo);
    assert_eq!(votes_client.get_delegate(&frodo), frodo);
    assert_eq!(votes_client.balance(&samwise), initial_amount_samwise);
    assert_eq!(votes_client.balance(&frodo), initial_amount_frodo);
    assert_eq!(
        votes_client.total_supply(),
        initial_amount_samwise + initial_amount_frodo
    );
    assert_eq!(votes_client.get_votes(&samwise), 0);
    assert_eq!(
        votes_client.get_votes(&frodo),
        initial_amount_samwise + initial_amount_frodo
    );
    assert_eq!(
        votes_client.get_past_votes(&samwise, &(e.ledger().sequence() - 1)),
        initial_amount_samwise
    );
    assert_eq!(
        votes_client.get_past_votes(&frodo, &(e.ledger().sequence() - 1)),
        initial_amount_frodo
    );
    assert_eq!(
        token_client.balance(&samwise),
        initial_balance - initial_amount_samwise
    );
    assert_eq!(
        token_client.balance(&frodo),
        initial_balance - initial_amount_frodo
    );
    assert_eq!(
        token_client.balance(&votes_id),
        initial_amount_samwise + initial_amount_frodo
    );

    // validate events
    let events = e.events().all();
    let tx_events = events.slice((events.len() - 3)..(events.len()));
    let event_data_0: soroban_sdk::Vec<Val> =
        vec![&e, initial_amount_samwise.into_val(&e), 0i128.into_val(&e)];
    let event_data_1: soroban_sdk::Vec<Val> = vec![
        &e,
        initial_amount_frodo.into_val(&e),
        (initial_amount_frodo + initial_amount_samwise).into_val(&e),
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
                (Symbol::new(&e, "votes_changed"), frodo.clone()).into_val(&e),
                event_data_1.into_val(&e)
            ),
            (
                votes_id.clone(),
                (Symbol::new(&e, "delegate"), samwise.clone(), frodo.clone()).into_val(&e),
                samwise.into_val(&e)
            )
        ]
    );
}

#[test]
fn test_delegation_chain_only_delegates_balance() {
    let e = Env::default();
    e.budget().reset_unlimited();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let frodo = Address::generate(&e);
    let pippin = Address::generate(&e);
    let merry = Address::generate(&e);

    let governor = Address::generate(&e);

    let (_, votes_client) = create_soroban_token_votes_wasm(&e, &bombadil, &governor);

    // setup vote ledgers - do a ledger before each action to verify the actions
    // occuring after the vote starts are recorded properly
    let cur_ledger = e.ledger().sequence();
    votes_client.set_vote_sequence(&(cur_ledger + 99));
    votes_client.set_vote_sequence(&(cur_ledger + 199));
    votes_client.set_vote_sequence(&(cur_ledger + 299));

    let initial_amount_frodo = 50 * 10i128.pow(7);
    votes_client.mint(&frodo, &initial_amount_frodo);

    let initial_amount_samwise = 25 * 10i128.pow(7);
    votes_client.mint(&samwise, &initial_amount_samwise);

    let initial_amount_pippen = 100 * 10i128.pow(7);
    votes_client.mint(&pippin, &initial_amount_pippen);

    e.jump(100);

    // delegate from pippin -> samwise
    votes_client.delegate(&pippin, &samwise);

    assert_eq!(votes_client.get_delegate(&pippin), samwise);
    assert_eq!(votes_client.get_delegate(&samwise), samwise);
    assert_eq!(votes_client.balance(&pippin), initial_amount_pippen);
    assert_eq!(votes_client.balance(&samwise), initial_amount_samwise);
    assert_eq!(votes_client.get_votes(&pippin), 0);
    assert_eq!(
        votes_client.get_votes(&samwise),
        initial_amount_samwise + initial_amount_pippen
    );

    e.jump(100);

    // delegate from samwise -> frodo, verify only balance is delegated
    votes_client.delegate(&samwise, &frodo);

    assert_eq!(votes_client.get_delegate(&samwise), frodo);
    assert_eq!(votes_client.get_delegate(&frodo), frodo);
    assert_eq!(votes_client.balance(&samwise), initial_amount_samwise);
    assert_eq!(votes_client.balance(&frodo), initial_amount_frodo);
    assert_eq!(votes_client.get_votes(&pippin), 0);
    assert_eq!(votes_client.get_votes(&samwise), initial_amount_pippen);
    assert_eq!(
        votes_client.get_votes(&frodo),
        initial_amount_samwise + initial_amount_frodo
    );

    e.jump(100);

    // verify transfers only effect immediate delegate
    let transfer_amount = 10 * 10i128.pow(7);
    votes_client.transfer(&pippin, &merry, &transfer_amount);

    assert_eq!(
        votes_client.balance(&pippin),
        initial_amount_pippen - transfer_amount
    );
    assert_eq!(votes_client.balance(&merry), transfer_amount);
    assert_eq!(votes_client.get_votes(&pippin), 0);
    assert_eq!(
        votes_client.get_votes(&samwise),
        initial_amount_pippen - transfer_amount
    );
    assert_eq!(
        votes_client.get_votes(&frodo),
        initial_amount_samwise + initial_amount_frodo
    );
    assert_eq!(votes_client.get_votes(&merry), transfer_amount);

    // verify checkpoints for pippin
    assert_eq!(
        votes_client.get_past_votes(&pippin, &(e.ledger().sequence() - 201)),
        initial_amount_pippen
    );
    assert_eq!(
        votes_client.get_past_votes(&pippin, &(e.ledger().sequence() - 101)),
        0
    );
    assert_eq!(
        votes_client.get_past_votes(&pippin, &(e.ledger().sequence() - 1)),
        0
    );

    // verify checkpoints for samwise
    assert_eq!(
        votes_client.get_past_votes(&samwise, &(e.ledger().sequence() - 201)),
        initial_amount_samwise
    );
    assert_eq!(
        votes_client.get_past_votes(&samwise, &(e.ledger().sequence() - 101)),
        initial_amount_samwise + initial_amount_pippen
    );
    assert_eq!(
        votes_client.get_past_votes(&samwise, &(e.ledger().sequence() - 1)),
        initial_amount_pippen
    );

    // verify checkpoints for frodo
    assert_eq!(
        votes_client.get_past_votes(&frodo, &(e.ledger().sequence() - 201)),
        initial_amount_frodo
    );
    assert_eq!(
        votes_client.get_past_votes(&frodo, &(e.ledger().sequence() - 101)),
        initial_amount_frodo
    );
    assert_eq!(
        votes_client.get_past_votes(&frodo, &(e.ledger().sequence() - 1)),
        initial_amount_frodo + initial_amount_samwise
    );
}

#[test]
fn test_delegation_to_current_delegate() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let frodo = Address::generate(&e);
    let governor = Address::generate(&e);

    let (token_id, _) = create_stellar_token(&e, &bombadil);
    let (_, votes_client) = create_bonding_token_votes(&e, &token_id, &governor);

    votes_client.delegate(&samwise, &frodo);

    assert_eq!(votes_client.get_delegate(&samwise), frodo);

    let result = votes_client.try_delegate(&samwise, &frodo);
    assert_eq!(result.err(), Some(Ok(Error::from_contract_error(101))));
}
