use crate::contract::Contract;
use crate::error::ContractError;
use crate::helpers::TryToBinary;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};

// TODO optimize after reading eth2 light client spec
// TODO review cargo xtasks
// TODO review deps.api object
// TODO quoted_int could cause errors deserialize_str test data - somethine somewhere is trying to serialize with serde_json we need to find and remove it
// TODO remove uneeded features and deps
// TODO readme makes no sense
// TODO add docs
// TODO add gas estimation to  e2etest
// TODO REFACTOR
// TODO refer existing cosmwasm contracts
// TODO improve with rust tooling
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
    contract.init(deps, msg.init_input.clone());

    Ok(contract
        .response_with_logs(
            Response::new()
                .add_attribute("method", "instantiate")
                .add_attribute("caller", info.sender),
        )
        .add_attribute(
            "instantiate_options",
            msg.init_input.try_to_binary()?.to_string(),
        ))
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
        ExecuteMsg::UpdateLightClient {
            light_client_update,
        } => {
            resp = resp.add_attribute("execute_method", "update_light_client");
            contract.update_light_client(deps, light_client_update)
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
        QueryMsg::Head {} => contract.head(deps).try_to_binary()?,
        QueryMsg::HeaderRoot { slot } => contract.header_root(deps, slot).try_to_binary()?,
        QueryMsg::ExecutionStateRoot { slot } => {
            contract.execution_state_root(deps, slot).try_to_binary()?
        }
        QueryMsg::SyncCommitteePoseidonHash { period } => contract
            .sync_committee_poseidon_hash(deps, period)
            .try_to_binary()?,
        QueryMsg::VerifyLogEntry {
            verify_log_entry_request,
        } => contract
            .verify_log_entry(deps, verify_log_entry_request)
            .try_to_binary()?,
    };

    Ok(res)
}
