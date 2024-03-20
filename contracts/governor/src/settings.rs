use soroban_sdk::{panic_with_error, Env};

use crate::{
    constants::{MAX_PROPOSAL_LIFETIME, MAX_VOTE_PERIOD},
    errors::GovernorError,
    types::GovernorSettings,
};

/// Set the governor settings and validate they fit within the maximums
///
/// ### Arguments
/// * `settings` - The settings for the governor
///
/// ### Panics
/// * If the vote_period is greater than the maximum vote period
/// * If the vote_delay + vote_period + timelock + grace_period is greater than the maximum proposal lifetime
pub fn require_valid_settings(e: &Env, settings: &GovernorSettings) {
    if settings.vote_period > MAX_VOTE_PERIOD {
        panic_with_error!(&e, GovernorError::InvalidSettingsError)
    }
    if settings.vote_delay + settings.vote_period + settings.timelock + settings.grace_period
        > MAX_PROPOSAL_LIFETIME
    {
        panic_with_error!(&e, GovernorError::InvalidSettingsError)
    }
}
