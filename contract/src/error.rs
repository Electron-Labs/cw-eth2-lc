use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}

impl From<std::io::Error> for ContractError {
    fn from(value: std::io::Error) -> Self {
        ContractError::Std(StdError::generic_err(value.to_string()))
    }
}
