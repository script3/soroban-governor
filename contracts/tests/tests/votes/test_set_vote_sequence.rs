#[cfg(test)]
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Env, IntoVal, Symbol,
};
use tests::{common::create_stellar_token, env::EnvTestUtils, votes::create_bonding_token_votes};

#[test]
fn test_set_vote_sequence() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let samwise = Address::generate(&e);
    let governor = Address::generate(&e);

    let (token_id, token_client) = create_stellar_token(&e, &bombadil);
    let (votes_id, votes_client) = create_bonding_token_votes(&e, &token_id, &governor);

    token_client.mint(&samwise, &123);
    votes_client.deposit(&samwise, &123);

    let to_add_sequence = e.ledger().sequence() + 100;
    votes_client.set_vote_sequence(&to_add_sequence);
    // validate auth
    assert_eq!(
        e.auths()[0],
        (
            governor.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    votes_id.clone(),
                    Symbol::new(&e, "set_vote_sequence"),
                    vec![&e, to_add_sequence.into_val(&e)]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    // validate votes are tracked based on the added sequence
    e.jump(101);

    votes_client.withdraw(&samwise, &23);

    assert_eq!(votes_client.get_votes(&samwise), 100);
    assert_eq!(
        votes_client.get_past_votes(&samwise, &(e.ledger().sequence() - 1)),
        123
    );
    assert_eq!(
        votes_client.get_past_votes(&samwise, &(e.ledger().sequence() - 101)),
        123
    );
    assert_eq!(
        votes_client.get_past_votes(&samwise, &(e.ledger().sequence() - 102)),
        0
    );
}
