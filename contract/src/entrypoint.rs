use crate::helpers::TryToBinary;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};

use crate::{
    contract::Contract,
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
};

// TODO do we want to pause contract?
// TODO use indexed map
// TODO add verify log entry tests
// TODO use standardized directory structure

// TODO optimize after reading eth2 light client spec
// TODO remove all panics
// TODO optimised test speed for integration tests

// TODO uncomment tests
// TODO quoted_int could cause errors deserialize_str test data - somethine somewhere is trying to serialize with serde_json we need to find and remove it
// TODO remove uneeded features and deps
// TODO readme makes no sense
// TODO add docs
// TODO add gas to test
// TODO add comprehensive errors
// TODO refactor tests
// TODO REFACTOR
// TODO add cicd
// TODO move todos to github issues
// TODO refer existing cosmwasm contracts
// TODO improve with rust tooling
// TODO add cosmwasm check to CI
// TODO add specific error to tests
// TODO look for improvements with awesome-cosmwasm
// TODO optimize rustfmt.toml

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let mut contract = Contract::new(env, Some(info.clone()));
    contract.init(deps, msg.0.clone());

    Ok(contract
        .response_with_logs(
            Response::new()
                .add_attribute("method", "instantiate")
                .add_attribute("instantiater", info.sender),
        )
        .add_attribute("instantiate_options", msg.0.try_to_binary()?.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let contract = Contract::new(env, Some(info.clone()));
    let mut resp = Response::new().add_attribute("method", "execute");

    match msg {
        ExecuteMsg::SubmitBeaconChainLightClientUpdate(light_client_update) => {
            resp = resp.add_attribute("execute_method", "submit_beacon_chain_light_client_update");
            contract.submit_beacon_chain_light_client_update(deps, light_client_update)
        }
        ExecuteMsg::SubmitExecutionHeader(block_header) => {
            resp = resp.add_attribute("execute_method", "submit_execution_header");
            contract.submit_execution_header(deps, block_header)
        }
        ExecuteMsg::UpdateTrustedSigner { trusted_signer } => {
            resp = resp.add_attribute("execute_method", "update_trusted_signer");
            contract.update_trusted_signer(deps, trusted_signer)
        }
    };

    Ok(contract.response_with_logs(resp.add_attribute("caller", info.sender)))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let res = try_query(deps, env, msg).map_err(|e| StdError::generic_err(e.to_string()))?;
    Ok(res)
}

pub fn try_query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    let contract = Contract::new(env, None);

    let res = match msg {
        QueryMsg::LastBlockNumber => contract.last_block_number(deps).try_to_binary()?,
        QueryMsg::BlockHashSafe { block_number } => contract
            .block_hash_safe(deps, block_number)
            .try_to_binary()?,
        QueryMsg::IsKnownExecutionHeader { hash } => contract
            .is_known_execution_header(deps, hash)
            .try_to_binary()?,
        QueryMsg::FinalizedBeaconBlockRoot => {
            contract.finalized_beacon_block_root(deps).try_to_binary()?
        }
        QueryMsg::FinalizedBeaconBlockSlot => {
            contract.finalized_beacon_block_slot(deps).try_to_binary()?
        }
        QueryMsg::FinalizedBeaconBlockHeader => contract
            .finalized_beacon_block_header(deps)
            .try_to_binary()?,
        QueryMsg::GetLightClientState => contract.get_light_client_state(deps).try_to_binary()?,
        QueryMsg::GetTrustedSigner => contract.get_trusted_signer(deps).try_to_binary()?,
        QueryMsg::VerifyLogEntry(req) => contract.verify_log_entry(deps, req).try_to_binary()?,
    };

    Ok(res)
}
