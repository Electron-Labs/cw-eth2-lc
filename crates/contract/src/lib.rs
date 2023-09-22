pub mod contract;
pub mod entrypoint;
pub mod error;
pub mod eth_utility;
pub mod helpers;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
