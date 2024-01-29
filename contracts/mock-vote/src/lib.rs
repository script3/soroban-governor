#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;
mod contract;
mod errors;
mod storage;
