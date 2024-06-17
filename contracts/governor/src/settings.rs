use soroban_sdk::{panic_with_error, Env};

use crate::{
    constants::{
        BPS_SCALAR, MAX_GRACE_PERIOD, MAX_PROPOSAL_LIFETIME, MAX_VOTE_PERIOD, MIN_GRACE_PERIOD, MIN_VOTE_PERIOD, MIN_VOTE_THRESHOLD
    },
    errors::GovernorError,
    types::GovernorSettings,
};

/// Set the governor settings and validate they fit within the maximums
///
/// ### Arguments
/// * `settings` - The settings for the governor
///
/// ### Panics
/// * If the vote_period is greater than the maximum vote period or less than the minimum vote period
/// * If the vote_delay + vote_period + timelock + grace_period is greater than the maximum proposal lifetime
/// * If the grace_period is less than the minimum grace period
/// * If the proposal_threshold is less than 1
/// * If the counting_type is greater than 0b111
/// * If the quorum or vote threshold is greater than 99% or less than 0.1%
pub fn require_valid_settings(e: &Env, settings: &GovernorSettings) {
    if settings.vote_period > MAX_VOTE_PERIOD
        || settings.vote_period < MIN_VOTE_PERIOD
        || settings.grace_period < MIN_GRACE_PERIOD
        || settings.grace_period > MAX_GRACE_PERIOD
        || settings.vote_delay
            + settings.vote_period
            + settings.timelock
            + settings.grace_period * 2
            > MAX_PROPOSAL_LIFETIME
        || settings.counting_type > 0b111
        || settings.proposal_threshold < MIN_VOTE_THRESHOLD
        || settings.quorum > BPS_SCALAR - 100
        || settings.quorum < 10
        || settings.vote_threshold > BPS_SCALAR - 100
        || settings.vote_threshold < 10
    {
        panic_with_error!(&e, GovernorError::InvalidSettingsError)
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::{ONE_DAY_LEDGERS, ONE_HOUR_LEDGERS};

    use super::*;

    #[test]
    fn test_require_valid_settings_is_valid() {
        let e = Env::default();
        let settings = GovernorSettings {
            proposal_threshold: 1_0000000,
            vote_delay: ONE_DAY_LEDGERS,
            vote_period: ONE_DAY_LEDGERS * 5,
            timelock: ONE_DAY_LEDGERS,
            grace_period: ONE_DAY_LEDGERS * 7,
            quorum: 100,
            counting_type: 2,
            vote_threshold: 5100,
        };

        require_valid_settings(&e, &settings);
        assert!(true);
    }

    #[test]
    fn test_require_valid_settings_is_valid_at_max() {
        let e = Env::default();
        let settings = GovernorSettings {
            proposal_threshold: 1_0000000,
            vote_delay: ONE_DAY_LEDGERS * 3,
            vote_period: ONE_DAY_LEDGERS * 7,
            timelock: ONE_DAY_LEDGERS * 7,
            grace_period: ONE_DAY_LEDGERS * 7,
            quorum: 100,
            counting_type: 2,
            vote_threshold: 5100,
        };

        require_valid_settings(&e, &settings);
        assert!(true);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #200)")]
    fn test_require_valid_settings_invalid_vote_period_max() {
        let e = Env::default();
        let settings = GovernorSettings {
            proposal_threshold: 1_0000000,
            vote_delay: ONE_DAY_LEDGERS,
            vote_period: ONE_DAY_LEDGERS * 7 + 1,
            timelock: ONE_DAY_LEDGERS,
            grace_period: ONE_DAY_LEDGERS * 7,
            quorum: 100,
            counting_type: 2,
            vote_threshold: 5100,
        };

        require_valid_settings(&e, &settings);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #200)")]
    fn test_require_valid_settings_invalid_vote_period_min() {
        let e = Env::default();
        let settings = GovernorSettings {
            proposal_threshold: 1_0000000,
            vote_delay: ONE_DAY_LEDGERS,
            vote_period: ONE_HOUR_LEDGERS - 1,
            timelock: ONE_DAY_LEDGERS,
            grace_period: ONE_DAY_LEDGERS * 7,
            quorum: 100,
            counting_type: 2,
            vote_threshold: 5100,
        };

        require_valid_settings(&e, &settings);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #200)")]
    fn test_require_valid_settings_invalid_grace_period_max() {
        let e = Env::default();
        let settings = GovernorSettings {
            proposal_threshold: 1_0000000,
            vote_delay: ONE_DAY_LEDGERS,
            vote_period: ONE_DAY_LEDGERS * 5,
            timelock: ONE_DAY_LEDGERS,
            grace_period: 7 * ONE_DAY_LEDGERS + 1,
            quorum: 100,
            counting_type: 2,
            vote_threshold: 5100,
        };

        require_valid_settings(&e, &settings);
        assert!(true);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #200)")]
    fn test_require_valid_settings_invalid_grace_period_min() {
        let e = Env::default();
        let settings = GovernorSettings {
            proposal_threshold: 1_0000000,
            vote_delay: ONE_DAY_LEDGERS,
            vote_period: ONE_DAY_LEDGERS * 5,
            timelock: ONE_DAY_LEDGERS,
            grace_period: ONE_DAY_LEDGERS - 1,
            quorum: 100,
            counting_type: 2,
            vote_threshold: 5100,
        };

        require_valid_settings(&e, &settings);
        assert!(true);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #200)")]
    fn test_require_valid_settings_invalid_proposal_lifetime() {
        let e = Env::default();
        let settings = GovernorSettings {
            proposal_threshold: 1_0000000,
            vote_delay: ONE_DAY_LEDGERS * 3 + 1,
            vote_period: ONE_DAY_LEDGERS * 7,
            timelock: ONE_DAY_LEDGERS * 7,
            grace_period: ONE_DAY_LEDGERS * 7,
            quorum: 100,
            counting_type: 2,
            vote_threshold: 5100,
        };

        require_valid_settings(&e, &settings);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #200)")]
    fn test_require_valid_settings_invalid_threshold() {
        let e = Env::default();
        let settings = GovernorSettings {
            proposal_threshold: 0,
            vote_delay: ONE_DAY_LEDGERS,
            vote_period: ONE_DAY_LEDGERS * 5,
            timelock: ONE_DAY_LEDGERS,
            grace_period: ONE_DAY_LEDGERS * 7,
            quorum: 100,
            counting_type: 2,
            vote_threshold: 5100,
        };

        require_valid_settings(&e, &settings);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #200)")]
    fn test_require_valid_settings_invalid_counting_type() {
        let e = Env::default();
        let settings = GovernorSettings {
            proposal_threshold: 1_0000000,
            vote_delay: ONE_DAY_LEDGERS,
            vote_period: ONE_DAY_LEDGERS * 5,
            timelock: ONE_DAY_LEDGERS,
            grace_period: ONE_DAY_LEDGERS * 7,
            quorum: 100,
            counting_type: 7 + 1,
            vote_threshold: 5100,
        };

        require_valid_settings(&e, &settings);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #200)")]
    fn test_require_valid_settings_invalid_quorum_max() {
        let e = Env::default();
        let settings = GovernorSettings {
            proposal_threshold: 1_0000000,
            vote_delay: ONE_DAY_LEDGERS,
            vote_period: ONE_DAY_LEDGERS * 5,
            timelock: ONE_DAY_LEDGERS,
            grace_period: ONE_DAY_LEDGERS * 7,
            quorum: BPS_SCALAR - 99,
            counting_type: 2,
            vote_threshold: 5100,
        };

        require_valid_settings(&e, &settings);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #200)")]
    fn test_require_valid_settings_invalid_quorum_min() {
        let e = Env::default();
        let settings = GovernorSettings {
            proposal_threshold: 1_0000000,
            vote_delay: ONE_DAY_LEDGERS,
            vote_period: ONE_DAY_LEDGERS * 5,
            timelock: ONE_DAY_LEDGERS,
            grace_period: ONE_DAY_LEDGERS * 7,
            quorum: 9,
            counting_type: 2,
            vote_threshold: 5100,
        };

        require_valid_settings(&e, &settings);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #200)")]
    fn test_require_valid_settings_invalid_vote_threshold_max() {
        let e = Env::default();
        let settings = GovernorSettings {
            proposal_threshold: 1_0000000,
            vote_delay: ONE_DAY_LEDGERS,
            vote_period: ONE_DAY_LEDGERS * 5,
            timelock: ONE_DAY_LEDGERS,
            grace_period: ONE_DAY_LEDGERS * 7,
            quorum: 100,
            counting_type: 2,
            vote_threshold: BPS_SCALAR - 99,
        };

        require_valid_settings(&e, &settings);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #200)")]
    fn test_require_valid_settings_invalid_vote_threshold_min() {
        let e = Env::default();
        let settings = GovernorSettings {
            proposal_threshold: 1_0000000,
            vote_delay: ONE_DAY_LEDGERS,
            vote_period: ONE_DAY_LEDGERS * 5,
            timelock: ONE_DAY_LEDGERS,
            grace_period: ONE_DAY_LEDGERS * 7,
            quorum: 100,
            counting_type: 2,
            vote_threshold: 9,
        };

        require_valid_settings(&e, &settings);
    }
}
