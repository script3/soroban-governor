use soroban_sdk::{
    testutils::{Ledger, LedgerInfo},
    Env,
};

pub trait EnvTestUtils {
    /// Jump the ledger time by the given amount of time. Don't advance the sequence number.
    fn jump(&self, time: u64);

    /// Jump the ledger time and sequence number by the given amount of time.
    /// Assumes 5 seconds per ledger.
    fn jump_with_sequence(&self, time: u64);

    /// Set the ledger to the default LedgerInfo
    ///
    /// Time -> 1441065600 (Sept 1st, 2015 12:00:00 AM UTC)
    /// Sequence -> 100
    fn set_default_info(&self);
}

impl EnvTestUtils for Env {
    fn jump(&self, time: u64) {
        self.ledger().set(LedgerInfo {
            timestamp: self.ledger().timestamp().saturating_add(time),
            protocol_version: 20,
            sequence_number: self.ledger().sequence(),
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 999999,
            min_persistent_entry_ttl: 999999,
            max_entry_ttl: 9999999,
        });
    }

    fn jump_with_sequence(&self, time: u64) {
        let blocks = time / 5;
        self.ledger().set(LedgerInfo {
            timestamp: self.ledger().timestamp().saturating_add(time),
            protocol_version: 20,
            sequence_number: self.ledger().sequence().saturating_add(blocks as u32),
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 999999,
            min_persistent_entry_ttl: 999999,
            max_entry_ttl: 9999999,
        });
    }

    fn set_default_info(&self) {
        self.ledger().set(LedgerInfo {
            timestamp: 1441065600, // Sept 1st, 2015 12:00:00 AM UTC
            protocol_version: 20,
            sequence_number: 100,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 999999,
            min_persistent_entry_ttl: 500000,
            max_entry_ttl: 9999999,
        });
    }
}
