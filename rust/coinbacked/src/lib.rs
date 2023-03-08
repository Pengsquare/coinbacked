//! minting tests
#![deny(missing_docs)]
#![forbid(unsafe_code)]

pub use solana_program;

pub mod processor;
mod utils;
mod entrypoint;
mod state;
mod instruction;
mod error;
mod math;

// for development
solana_program::declare_id!("B91LvPYXAo3KVNFbSXkWJWunVtXMV5irzdWCqPxJfMR7");
