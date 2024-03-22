#[cfg(test)]
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, IntoVal, String, Symbol,
};
use tests::{
    common::create_stellar_token,
    env::EnvTestUtils,
    votes::{create_soroban_token_votes_wasm, create_staking_token_votes},
};

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn initialize_already_initialized() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let governor = Address::generate(&e);

    let (token_id, _) = create_stellar_token(&e, &bombadil);
    let (_, votes_client) = create_staking_token_votes(&e, &token_id, &governor);

    votes_client.initialize(
        &Address::generate(&e),
        &Address::generate(&e),
        &String::from_str(&e, "1"),
        &String::from_str(&e, "2"),
    );
}

// Soroban token

#[test]
fn test_token_actions_soroban() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();
    e.budget().reset_unlimited();

    let bombadil = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let user3 = Address::generate(&e);
    let governor = Address::generate(&e);

    let (_, votes_client) = create_soroban_token_votes_wasm(&e, &bombadil, &governor);

    let deposit_amount = 1000;
    votes_client.mint(&user1, &deposit_amount);
    // validate auth
    assert_eq!(
        e.auths(),
        std::vec![(
            bombadil.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    votes_client.address.clone(),
                    symbol_short!("mint"),
                    (&user1, deposit_amount).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    votes_client.approve(&user2, &user3, &500, &200);
    assert_eq!(
        e.auths(),
        std::vec![(
            user2.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    votes_client.address.clone(),
                    symbol_short!("approve"),
                    (&user2, &user3, 500_i128, 200_u32).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(votes_client.allowance(&user2, &user3), 500);

    votes_client.transfer(&user1, &user2, &600);
    assert_eq!(
        e.auths(),
        std::vec![(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    votes_client.address.clone(),
                    symbol_short!("transfer"),
                    (&user1, &user2, 600_i128).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(votes_client.balance(&user1), 400);
    assert_eq!(votes_client.get_votes(&user1), 400);
    assert_eq!(votes_client.balance(&user2), 600);
    assert_eq!(votes_client.get_votes(&user2), 600);

    votes_client.transfer_from(&user3, &user2, &user1, &400);
    assert_eq!(
        e.auths(),
        std::vec![(
            user3.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    votes_client.address.clone(),
                    Symbol::new(&e, "transfer_from"),
                    (&user3, &user2, &user1, 400_i128).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(votes_client.balance(&user1), 800);
    assert_eq!(votes_client.get_votes(&user1), 800);
    assert_eq!(votes_client.balance(&user2), 200);
    assert_eq!(votes_client.get_votes(&user2), 200);

    votes_client.transfer(&user1, &user3, &300);
    assert_eq!(votes_client.balance(&user1), 500);
    assert_eq!(votes_client.get_votes(&user1), 500);
    assert_eq!(votes_client.balance(&user3), 300);
    assert_eq!(votes_client.get_votes(&user3), 300);

    // Increase to 500
    votes_client.approve(&user2, &user3, &500, &200);
    assert_eq!(votes_client.allowance(&user2, &user3), 500);
    votes_client.approve(&user2, &user3, &0, &200);
    assert_eq!(
        e.auths(),
        std::vec![(
            user2.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    votes_client.address.clone(),
                    symbol_short!("approve"),
                    (&user2, &user3, 0_i128, 200_u32).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(votes_client.allowance(&user2, &user3), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn transfer_insufficient_balance_soroban() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let governor = Address::generate(&e);

    let (_, votes_client) = create_soroban_token_votes_wasm(&e, &bombadil, &governor);

    let deposit_amount = 1000;
    votes_client.mint(&user1, &deposit_amount);

    votes_client.transfer(&user1, &user2, &1001);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn transfer_from_insufficient_allowance_soroban() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let user3 = Address::generate(&e);
    let governor = Address::generate(&e);

    let (_, votes_client) = create_soroban_token_votes_wasm(&e, &bombadil, &governor);

    let deposit_amount = 1000;
    votes_client.mint(&user1, &deposit_amount);

    votes_client.approve(&user1, &user3, &100, &200);
    assert_eq!(votes_client.allowance(&user1, &user3), 100);

    votes_client.transfer_from(&user3, &user1, &user2, &101);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn initialize_already_initialized_soroban() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();

    let bombadil = Address::generate(&e);
    let governor = Address::generate(&e);

    let (token_id, _) = create_stellar_token(&e, &bombadil);
    let (_, votes_client) = create_soroban_token_votes_wasm(&e, &token_id, &governor);

    votes_client.initialize(
        &Address::generate(&e),
        &Address::generate(&e),
        &1,
        &String::from_str(&e, "1"),
        &String::from_str(&e, "2"),
    );
}
