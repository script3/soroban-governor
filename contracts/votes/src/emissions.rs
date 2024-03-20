#![cfg(feature = "emissions")]

use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{panic_with_error, unwrap::UnwrapOptimized, Address, Env};

use crate::{
    balance,
    constants::SCALAR_7,
    error::TokenVotesError,
    events::TokenVotesEvents,
    storage::{self, EmissionConfig, EmissionData, UserEmissionData},
};

/// Claim emission for a user into their vote token balance
///
/// ### Arguments
/// * `total_supply` - The total supply of the vote token
/// * `user` - The address of the user
/// * `balance` - The balance of the user
///
/// ### Returns
/// The number of tokens claimed
pub fn claim_emissions(e: &Env, total_supply: i128, user: &Address, balance: i128) -> i128 {
    if let Some(emis_config) = storage::get_emission_config(e) {
        let prev_emis_data = storage::get_emission_data(e).unwrap_optimized(); // exists if config exists
        let emis_data = match update_emission_data(e, &prev_emis_data, &emis_config, total_supply) {
            Some(data) => {
                storage::set_emission_data(e, &data);
                data
            }
            None => prev_emis_data,
        };
        let prev_data = storage::get_user_emission_data(e, user);
        let mut user_data = match update_user_emissions(&prev_data, &emis_data, balance) {
            Some(data) => data,
            None => prev_data.unwrap_optimized(),
        };

        let to_claim = user_data.accrued.clone();
        if to_claim > 0 {
            user_data.accrued = 0;
            storage::set_user_emission_data(e, user, &user_data);

            balance::mint_balance(e, &user, to_claim);

            TokenVotesEvents::claim(&e, user.clone(), to_claim);
        } else {
            storage::set_user_emission_data(e, user, &user_data);
        }
        to_claim
    } else {
        0
    }
}

/// Update the emissions for a balance change
///
/// ### Arguments
/// * `total_supply` - The total supply of the vote token
/// * `user` - The address of the user
/// * `balance` - The balance of the user
pub fn update_emissions(e: &Env, total_supply: i128, user: &Address, balance: i128) {
    if let Some(emis_config) = storage::get_emission_config(e) {
        let prev_emis_data = storage::get_emission_data(e).unwrap_optimized(); // exists if config exists
        let emis_data = match update_emission_data(e, &prev_emis_data, &emis_config, total_supply) {
            Some(data) => {
                storage::set_emission_data(e, &data);
                data
            }
            None => prev_emis_data,
        };
        let user_data = storage::get_user_emission_data(e, user);
        if let Some(new_user_data) = update_user_emissions(&user_data, &emis_data, balance) {
            storage::set_user_emission_data(e, user, &new_user_data);
        }
    }
}

/// Update the emissions for a balance change
///
/// ### Arguments
/// * `total_supply` - The total supply of the vote token
/// * `new_tokens` - The address of the user
/// * `balance` - The balance of the user
pub fn set_emissions(e: &Env, total_supply: i128, new_tokens: i128, new_expiration: u64) {
    if new_expiration <= e.ledger().timestamp() || new_tokens <= 0 {
        panic_with_error!(e, TokenVotesError::InvalidEmissionConfigError);
    }
    let mut tokens_left_to_emit = new_tokens;
    if let Some(emis_config) = storage::get_emission_config(e) {
        // data exists - update it with old config
        let prev_emis_data = storage::get_emission_data(e).unwrap_optimized(); // exists if config exists
        let mut emis_data =
            match update_emission_data(e, &prev_emis_data, &emis_config, total_supply) {
                Some(data) => data,
                None => prev_emis_data,
            };
        if emis_data.last_time != e.ledger().timestamp() {
            // force the emission data to be updated to the current timestamp
            emis_data.last_time = e.ledger().timestamp();
        }
        storage::set_emission_data(e, &emis_data);

        // determine the amount of tokens not emitted from the last config
        if emis_config.expiration > e.ledger().timestamp() {
            let time_since_last_emission = emis_config.expiration - e.ledger().timestamp();
            let tokens_since_last_emission = (emis_config.eps * time_since_last_emission) as i128;
            tokens_left_to_emit += tokens_since_last_emission;
        }
    } else {
        // no config or data exists yet - first time this reserve token will get emission
        storage::set_emission_data(
            e,
            &EmissionData {
                index: 0,
                last_time: e.ledger().timestamp(),
            },
        );
    }
    let delta_seconds = (new_expiration - e.ledger().timestamp()) as i128;
    let eps: u64 = (tokens_left_to_emit / delta_seconds)
        .try_into()
        .unwrap_optimized();
    let new_config = EmissionConfig {
        expiration: new_expiration,
        eps,
    };
    storage::set_emission_config(e, &new_config);

    TokenVotesEvents::set_emissions(e, eps, new_expiration);
}

/// Update the backstop emissions index for deposits
///
/// ### Arguments
/// * `total_supply` - The total supply of the vote token
///
/// ### Returns
/// The updated emission data, or None if the data does not need updating
fn update_emission_data(
    e: &Env,
    emis_data: &EmissionData,
    emis_config: &EmissionConfig,
    total_supply: i128,
) -> Option<EmissionData> {
    if emis_data.last_time >= emis_config.expiration
        || e.ledger().timestamp() == emis_data.last_time
        || emis_config.eps == 0
        || total_supply == 0
    {
        // emis_data already updated or expired
        return None;
    }

    let max_timestamp = if e.ledger().timestamp() > emis_config.expiration {
        emis_config.expiration
    } else {
        e.ledger().timestamp()
    };

    let additional_idx = ((max_timestamp - emis_data.last_time) as i128
        * (emis_config.eps as i128))
        .fixed_div_floor(total_supply, SCALAR_7)
        .unwrap_optimized();
    let new_data = EmissionData {
        index: additional_idx + emis_data.index,
        last_time: e.ledger().timestamp(),
    };
    Some(new_data)
}

/// Update the user's emissions
///
/// ### Arguments
/// * `user` - The address of the user
/// * `emis_data` - The emission data
/// * `balance` - The balance of the user
///
/// ### Returns
/// The user's emission data or None if the data does not need updating
fn update_user_emissions(
    user_data_opt: &Option<UserEmissionData>,
    emis_data: &EmissionData,
    balance: i128,
) -> Option<UserEmissionData> {
    if let Some(mut user_data) = user_data_opt.clone() {
        if user_data.index < emis_data.index {
            if balance != 0 {
                let delta_index = emis_data.index - user_data.index;
                let to_accrue = balance
                    .fixed_mul_floor(delta_index, SCALAR_7)
                    .unwrap_optimized();
                user_data.accrued += to_accrue;
            }
            user_data.index = emis_data.index;
            Some(user_data)
        } else {
            None
        }
    } else if balance == 0 {
        // first time the user registered a balance after emissions began
        Some(UserEmissionData {
            index: emis_data.index,
            accrued: 0,
        })
    } else {
        // user had tokens before emissions began, they are due any historical emissions
        let accrued = balance
            .fixed_mul_floor(emis_data.index, SCALAR_7)
            .unwrap_optimized();
        Some(UserEmissionData {
            index: emis_data.index,
            accrued,
        })
    }
}

#[cfg(test)]
mod tests {
    use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};

    use crate::checkpoints::Checkpoint;

    use super::*;

    const ONE_DAY: u64 = 24 * 60 * 60;

    /********** update_emissions **********/

    #[test]
    fn test_update_emissions() {
        let e = Env::default();
        let t_now = 1500000000;
        e.ledger().set(LedgerInfo {
            timestamp: t_now,
            protocol_version: 20,
            sequence_number: 123,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 100000,
            min_persistent_entry_ttl: 100000,
            max_entry_ttl: 100000,
        });

        let user = Address::generate(&e);
        let contract = e.register_contract(None, crate::TokenVotes {});

        let total_supply = 1_123_456 * SCALAR_7;
        let balance = 50_000 * SCALAR_7;
        let emis_config = EmissionConfig {
            expiration: t_now + ONE_DAY * 5,
            eps: 5000000, // 0.5 tokens per second
        };
        let emis_data = EmissionData {
            index: 1234567,
            last_time: t_now - ONE_DAY,
        };
        let user_data = UserEmissionData {
            index: 1000000,
            accrued: 4,
        };

        e.as_contract(&contract, || {
            storage::set_emission_config(&e, &emis_config);
            storage::set_emission_data(&e, &emis_data);
            storage::set_user_emission_data(&e, &user, &user_data);

            update_emissions(&e, total_supply, &user, balance);

            let new_emis_data = storage::get_emission_data(&e).unwrap();
            assert_eq!(new_emis_data.index, 1234567 + 384527);
            assert_eq!(new_emis_data.last_time, t_now);
            let new_user_data = storage::get_user_emission_data(&e, &user).unwrap();
            assert_eq!(new_user_data.index, new_emis_data.index);
            assert_eq!(new_user_data.accrued, 4 + 3095_4700000);
        });
    }

    #[test]
    fn test_update_emissions_no_user_data_with_balance_accrues() {
        let e = Env::default();
        let t_now = 1500000000;
        e.ledger().set(LedgerInfo {
            timestamp: t_now,
            protocol_version: 20,
            sequence_number: 123,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 100000,
            min_persistent_entry_ttl: 100000,
            max_entry_ttl: 100000,
        });

        let user = Address::generate(&e);
        let contract = e.register_contract(None, crate::TokenVotes {});

        let total_supply = 1_123_456 * SCALAR_7;
        let balance = 1000 * SCALAR_7;
        let emis_config = EmissionConfig {
            expiration: t_now + ONE_DAY * 5,
            eps: 5000000, // 0.5 tokens per second
        };
        let emis_data = EmissionData {
            index: 1000,
            last_time: t_now - ONE_DAY,
        };

        e.as_contract(&contract, || {
            storage::set_emission_config(&e, &emis_config);
            storage::set_emission_data(&e, &emis_data);

            update_emissions(&e, total_supply, &user, balance);

            let new_emis_data = storage::get_emission_data(&e).unwrap();
            assert_eq!(new_emis_data.index, 1000 + 384527);
            assert_eq!(new_emis_data.last_time, t_now);
            let new_user_data = storage::get_user_emission_data(&e, &user).unwrap();
            assert_eq!(new_user_data.index, new_emis_data.index);
            assert_eq!(new_user_data.accrued, 38_5527000);
        });
    }

    #[test]
    fn test_update_emissions_no_starting_balance_no_accrual() {
        let e = Env::default();
        let t_now = 1500000000;
        e.ledger().set(LedgerInfo {
            timestamp: t_now,
            protocol_version: 20,
            sequence_number: 123,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 100000,
            min_persistent_entry_ttl: 100000,
            max_entry_ttl: 100000,
        });

        let user = Address::generate(&e);
        let contract = e.register_contract(None, crate::TokenVotes {});

        let total_supply = 1_123_456 * SCALAR_7;
        let balance = 0;
        let emis_config = EmissionConfig {
            expiration: t_now + ONE_DAY * 5,
            eps: 5000000, // 0.5 tokens per second
        };
        let emis_data = EmissionData {
            index: 1000,
            last_time: t_now - ONE_DAY,
        };

        e.as_contract(&contract, || {
            storage::set_emission_config(&e, &emis_config);
            storage::set_emission_data(&e, &emis_data);

            update_emissions(&e, total_supply, &user, balance);

            let new_emis_data = storage::get_emission_data(&e).unwrap();
            assert_eq!(new_emis_data.index, 1000 + 384527);
            assert_eq!(new_emis_data.last_time, t_now);
            let new_user_data = storage::get_user_emission_data(&e, &user).unwrap();
            assert_eq!(new_user_data.index, new_emis_data.index);
            assert_eq!(new_user_data.accrued, 0);
        });
    }

    #[test]
    fn test_update_emissions_no_update_data_still_updates_user() {
        let e = Env::default();
        let t_now = 1500000000;
        e.ledger().set(LedgerInfo {
            timestamp: t_now,
            protocol_version: 20,
            sequence_number: 123,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 100000,
            min_persistent_entry_ttl: 100000,
            max_entry_ttl: 100000,
        });

        let user = Address::generate(&e);
        let contract = e.register_contract(None, crate::TokenVotes {});

        let total_supply = 1_123_456 * SCALAR_7;
        let balance = 50_000 * SCALAR_7;
        let emis_config = EmissionConfig {
            expiration: t_now + ONE_DAY * 5,
            eps: 5000000, // 0.5 tokens per second
        };
        let emis_data = EmissionData {
            index: 1234567,
            last_time: t_now,
        };
        let user_data = UserEmissionData {
            index: 1000000,
            accrued: 0,
        };

        e.as_contract(&contract, || {
            storage::set_emission_config(&e, &emis_config);
            storage::set_emission_data(&e, &emis_data);
            storage::set_user_emission_data(&e, &user, &user_data);

            update_emissions(&e, total_supply, &user, balance);

            let new_emis_data = storage::get_emission_data(&e).unwrap();
            assert_eq!(new_emis_data.index, 1234567);
            assert_eq!(new_emis_data.last_time, t_now);
            let new_user_data = storage::get_user_emission_data(&e, &user).unwrap();
            assert_eq!(new_user_data.index, new_emis_data.index);
            assert_eq!(new_user_data.accrued, 1172_8350000);
        });
    }

    #[test]
    fn test_update_emissions_config_expires() {
        let e = Env::default();
        let t_now = 1500000000;
        e.ledger().set(LedgerInfo {
            timestamp: t_now,
            protocol_version: 20,
            sequence_number: 123,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 100000,
            min_persistent_entry_ttl: 100000,
            max_entry_ttl: 100000,
        });

        let user = Address::generate(&e);
        let contract = e.register_contract(None, crate::TokenVotes {});

        let total_supply = 123_456 * SCALAR_7;
        let balance = 42 * SCALAR_7;
        let emis_config = EmissionConfig {
            expiration: t_now - 1000,
            eps: 5000000, // 0.5 tokens per second
        };
        let emis_data = EmissionData {
            index: 0,
            last_time: t_now - 1100,
        };
        let user_data = UserEmissionData {
            index: 0,
            accrued: 0,
        };

        e.as_contract(&contract, || {
            storage::set_emission_config(&e, &emis_config);
            storage::set_emission_data(&e, &emis_data);
            storage::set_user_emission_data(&e, &user, &user_data);

            update_emissions(&e, total_supply, &user, balance);

            let new_emis_data = storage::get_emission_data(&e).unwrap();
            assert_eq!(new_emis_data.index, 4050);
            assert_eq!(new_emis_data.last_time, t_now);
            let new_user_data = storage::get_user_emission_data(&e, &user).unwrap();
            assert_eq!(new_user_data.index, new_emis_data.index);
            assert_eq!(new_user_data.accrued, 0_0170100);
        });
    }

    #[test]
    fn test_update_emissions_no_config() {
        let e = Env::default();
        let t_now = 1500000000;
        e.ledger().set(LedgerInfo {
            timestamp: t_now,
            protocol_version: 20,
            sequence_number: 123,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 100000,
            min_persistent_entry_ttl: 100000,
            max_entry_ttl: 100000,
        });

        let user = Address::generate(&e);
        let contract = e.register_contract(None, crate::TokenVotes {});

        let total_supply = 100 * SCALAR_7;
        let balance = 5 * SCALAR_7;
        e.as_contract(&contract, || {
            update_emissions(&e, total_supply, &user, balance);

            assert!(storage::get_emission_data(&e).is_none());
            assert!(storage::get_user_emission_data(&e, &user).is_none());
        });
    }

    #[test]
    fn test_update_emissions_no_balance() {
        let e = Env::default();
        let t_now = 1500000000;
        e.ledger().set(LedgerInfo {
            timestamp: t_now,
            protocol_version: 20,
            sequence_number: 123,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 100000,
            min_persistent_entry_ttl: 100000,
            max_entry_ttl: 100000,
        });

        let user = Address::generate(&e);
        let contract = e.register_contract(None, crate::TokenVotes {});

        let total_supply = 23_456 * SCALAR_7;
        let balance = 0;
        let emis_config = EmissionConfig {
            expiration: t_now + ONE_DAY * 5,
            eps: 2000000, // 0.2 tokens per second
        };
        let emis_data = EmissionData {
            index: 1234567,
            last_time: t_now - 12345,
        };
        let user_data = UserEmissionData {
            index: 4567,
            accrued: 0,
        };

        e.as_contract(&contract, || {
            storage::set_emission_config(&e, &emis_config);
            storage::set_emission_data(&e, &emis_data);
            storage::set_user_emission_data(&e, &user, &user_data);

            update_emissions(&e, total_supply, &user, balance);

            let new_emis_data = storage::get_emission_data(&e).unwrap();
            assert_eq!(new_emis_data.index, 1052609 + 1234567);
            assert_eq!(new_emis_data.last_time, t_now);
            let new_user_data = storage::get_user_emission_data(&e, &user).unwrap();
            assert_eq!(new_user_data.index, new_emis_data.index);
            assert_eq!(new_user_data.accrued, 0);
        });
    }

    /********** claim_emissions **********/

    #[test]
    fn test_claim_emissions() {
        let e = Env::default();
        let t_now = 1500000000;
        e.ledger().set(LedgerInfo {
            timestamp: t_now,
            protocol_version: 20,
            sequence_number: 123,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 100000,
            min_persistent_entry_ttl: 100000,
            max_entry_ttl: 100000,
        });

        let user = Address::generate(&e);
        let contract = e.register_contract(None, crate::TokenVotes {});

        let total_supply = 1_123_456 * SCALAR_7;
        let balance = 50_000 * SCALAR_7;
        let emis_config = EmissionConfig {
            expiration: t_now + ONE_DAY * 5,
            eps: 5000000, // 0.5 tokens per second
        };
        let emis_data = EmissionData {
            index: 1234567,
            last_time: t_now - ONE_DAY,
        };
        let user_data = UserEmissionData {
            index: 1000000,
            accrued: 4,
        };

        e.as_contract(&contract, || {
            storage::set_emission_config(&e, &emis_config);
            storage::set_emission_data(&e, &emis_data);
            storage::set_user_emission_data(&e, &user, &user_data);

            let result = claim_emissions(&e, total_supply, &user, balance);

            let new_emis_data = storage::get_emission_data(&e).unwrap();
            assert_eq!(new_emis_data.index, 1234567 + 384527);
            assert_eq!(new_emis_data.last_time, t_now);
            let new_user_data = storage::get_user_emission_data(&e, &user).unwrap();
            assert_eq!(new_user_data.index, new_emis_data.index);
            assert_eq!(new_user_data.accrued, 0);

            assert_eq!(result, 4 + 3095_4700000);
            let balance = storage::get_balance(&e, &user);
            assert_eq!(balance, 4 + 3095_4700000);
            let votes = storage::get_voting_units(&e, &user);
            assert_eq!(votes.to_checkpoint_data(), (123, 4 + 3095_4700000));
        });
    }

    #[test]
    fn test_claim_emissions_already_updated_and_delegated() {
        let e = Env::default();
        let t_now = 1500000000;
        e.ledger().set(LedgerInfo {
            timestamp: t_now,
            protocol_version: 20,
            sequence_number: 123,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 100000,
            min_persistent_entry_ttl: 100000,
            max_entry_ttl: 100000,
        });

        let user = Address::generate(&e);
        let samwise = Address::generate(&e);
        let contract = e.register_contract(None, crate::TokenVotes {});

        let total_supply = 1_123_456 * SCALAR_7;
        let balance = 50_000 * SCALAR_7;
        let emis_config = EmissionConfig {
            expiration: t_now + ONE_DAY * 5,
            eps: 5000000, // 0.5 tokens per second
        };
        let emis_data = EmissionData {
            index: 1234567,
            last_time: t_now,
        };
        let user_data = UserEmissionData {
            index: 1234567,
            accrued: 3095_4700000,
        };

        e.as_contract(&contract, || {
            storage::set_emission_config(&e, &emis_config);
            storage::set_emission_data(&e, &emis_data);
            storage::set_user_emission_data(&e, &user, &user_data);
            storage::set_delegate(&e, &user, &samwise);

            let result = claim_emissions(&e, total_supply, &user, balance);

            let new_emis_data = storage::get_emission_data(&e).unwrap();
            assert_eq!(new_emis_data.index, 1234567);
            assert_eq!(new_emis_data.last_time, t_now);
            let new_user_data = storage::get_user_emission_data(&e, &user).unwrap();
            assert_eq!(new_user_data.index, new_emis_data.index);
            assert_eq!(new_user_data.accrued, 0);

            assert_eq!(result, 3095_4700000);
            let balance = storage::get_balance(&e, &user);
            assert_eq!(balance, 3095_4700000);
            let votes = storage::get_voting_units(&e, &user);
            assert_eq!(votes, 0);
            let samwise_votes = storage::get_voting_units(&e, &samwise);
            assert_eq!(samwise_votes.to_checkpoint_data(), (123, 3095_4700000));
        });
    }

    #[test]
    fn test_claim_no_config() {
        let e = Env::default();
        let t_now = 1500000000;
        e.ledger().set(LedgerInfo {
            timestamp: t_now,
            protocol_version: 20,
            sequence_number: 123,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 100000,
            min_persistent_entry_ttl: 100000,
            max_entry_ttl: 100000,
        });

        let user = Address::generate(&e);
        let contract = e.register_contract(None, crate::TokenVotes {});

        let total_supply = 100 * SCALAR_7;
        let balance = 5 * SCALAR_7;
        e.as_contract(&contract, || {
            let result = claim_emissions(&e, total_supply, &user, balance);

            assert_eq!(result, 0);
            assert!(storage::get_emission_data(&e).is_none());
            assert!(storage::get_user_emission_data(&e, &user).is_none());
        });
    }

    #[test]
    fn test_claim_no_accrual() {
        let e = Env::default();
        let t_now = 1500000000;
        e.ledger().set(LedgerInfo {
            timestamp: t_now,
            protocol_version: 20,
            sequence_number: 123,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 100000,
            min_persistent_entry_ttl: 100000,
            max_entry_ttl: 100000,
        });

        let user = Address::generate(&e);
        let contract = e.register_contract(None, crate::TokenVotes {});

        let total_supply = 23_456 * SCALAR_7;
        let balance = 0;
        let emis_config = EmissionConfig {
            expiration: t_now + ONE_DAY * 5,
            eps: 2000000, // 0.2 tokens per second
        };
        let emis_data = EmissionData {
            index: 1234567,
            last_time: t_now - 12345,
        };
        let user_data = UserEmissionData {
            index: 4567,
            accrued: 0,
        };

        e.as_contract(&contract, || {
            storage::set_emission_config(&e, &emis_config);
            storage::set_emission_data(&e, &emis_data);
            storage::set_user_emission_data(&e, &user, &user_data);

            let result = claim_emissions(&e, total_supply, &user, balance);

            assert_eq!(result, 0);
            let new_emis_data = storage::get_emission_data(&e).unwrap();
            assert_eq!(new_emis_data.index, 1052609 + 1234567);
            assert_eq!(new_emis_data.last_time, t_now);
            let new_user_data = storage::get_user_emission_data(&e, &user).unwrap();
            assert_eq!(new_user_data.index, new_emis_data.index);
            assert_eq!(new_user_data.accrued, 0);

            assert_eq!(result, 0);
            let balance = storage::get_balance(&e, &user);
            assert_eq!(balance, 0);
            let votes = storage::get_voting_units(&e, &user);
            assert_eq!(votes, 0);
        });
    }

    /********** set_emissions **********/

    #[test]
    fn test_set_emissions_init() {
        let e = Env::default();
        let t_now = 1500000000;
        e.ledger().set(LedgerInfo {
            timestamp: t_now,
            protocol_version: 20,
            sequence_number: 123,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 100000,
            min_persistent_entry_ttl: 100000,
            max_entry_ttl: 100000,
        });

        let contract = e.register_contract(None, crate::TokenVotes {});

        let total_supply = 654_321 * SCALAR_7;
        let new_tokens = 14_000 * SCALAR_7;
        let new_expiration = t_now + ONE_DAY * 14;
        e.as_contract(&contract, || {
            set_emissions(&e, total_supply, new_tokens, new_expiration);

            let new_config = storage::get_emission_config(&e).unwrap();
            assert_eq!(new_config.expiration, new_expiration);
            assert_eq!(new_config.eps, 115740);
            let new_emis_data = storage::get_emission_data(&e).unwrap();
            assert_eq!(new_emis_data.index, 0);
            assert_eq!(new_emis_data.last_time, t_now);
        });
    }

    #[test]
    fn test_set_emissions_update_expired() {
        let e = Env::default();
        let t_now = 1500000000;
        e.ledger().set(LedgerInfo {
            timestamp: t_now,
            protocol_version: 20,
            sequence_number: 123,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 100000,
            min_persistent_entry_ttl: 100000,
            max_entry_ttl: 100000,
        });

        let contract = e.register_contract(None, crate::TokenVotes {});

        let total_supply = 654_321 * SCALAR_7;

        let emis_config = EmissionConfig {
            expiration: t_now - 1000,
            eps: 5000000, // 0.5 tokens per second
        };
        let emis_data = EmissionData {
            index: 54321,
            last_time: t_now - 1000,
        };

        let new_tokens = 70_000 * SCALAR_7;
        let new_expiration = t_now + ONE_DAY * 7;
        e.as_contract(&contract, || {
            storage::set_emission_config(&e, &emis_config);
            storage::set_emission_data(&e, &emis_data);

            set_emissions(&e, total_supply, new_tokens, new_expiration);

            let new_config = storage::get_emission_config(&e).unwrap();
            assert_eq!(new_config.expiration, new_expiration);
            assert_eq!(new_config.eps, 1157407);
            let new_emis_data = storage::get_emission_data(&e).unwrap();
            assert_eq!(new_emis_data.index, 54321);
            assert_eq!(new_emis_data.last_time, t_now);
        });
    }

    #[test]
    fn test_set_emissions_update_ongoing() {
        let e = Env::default();
        let t_now = 1500000000;
        e.ledger().set(LedgerInfo {
            timestamp: t_now,
            protocol_version: 20,
            sequence_number: 123,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 100000,
            min_persistent_entry_ttl: 100000,
            max_entry_ttl: 100000,
        });

        let contract = e.register_contract(None, crate::TokenVotes {});

        let total_supply = 654_321 * SCALAR_7;

        let emis_config = EmissionConfig {
            expiration: t_now + 5000,
            eps: 5000000, // 0.5 tokens per second
        };
        let emis_data = EmissionData {
            index: 54321,
            last_time: t_now - 1000,
        };

        let new_tokens = 70_000 * SCALAR_7;
        let new_expiration = t_now + ONE_DAY * 7;
        e.as_contract(&contract, || {
            storage::set_emission_config(&e, &emis_config);
            storage::set_emission_data(&e, &emis_data);

            set_emissions(&e, total_supply, new_tokens, new_expiration);

            let new_config = storage::get_emission_config(&e).unwrap();
            assert_eq!(new_config.expiration, new_expiration);
            assert_eq!(new_config.eps, 1198743);
            let new_emis_data = storage::get_emission_data(&e).unwrap();
            assert_eq!(new_emis_data.index, 54321 + 7641);
            assert_eq!(new_emis_data.last_time, t_now);
        });
    }
}
