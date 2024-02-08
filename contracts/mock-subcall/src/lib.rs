#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

pub mod contract;
pub mod storage;

pub use contract::*;
