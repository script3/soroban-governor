mod votes;
pub use votes::Client as VotesClient;

#[cfg(any(test, feature = "testutils"))]
pub use votes::WASM as VOTES_WASM;
