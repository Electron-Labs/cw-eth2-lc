#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;
use eth2_utility::types::InitInput;

use crate::{error::ContractError, msg::GenericQueryResponse, rainbow};
use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::STATE_KEY,
};
use borsh::BorshDeserialize;
use borsh::BorshSerialize;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:eth2-client";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO we dont need to deserialize 50k headers on every call

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let args = InitInput::try_from_slice(msg.borsh.as_slice())?;
    let state = rainbow::Eth2Client::init(args);
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    deps.storage.set(STATE_KEY, state.try_to_vec()?.as_slice());

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let res = try_query(deps, env, msg).map_err(|e| StdError::generic_err(e.to_string()))?;

    Ok(res)
}

pub fn try_query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    let state = rainbow::Eth2Client::try_from_slice(
        deps.storage
            .get(STATE_KEY)
            .ok_or(ContractError::Std(StdError::generic_err(
                "could not find state",
            )))?
            .as_slice(),
    )?;
    let res = match msg {
        QueryMsg::IsInitialized {} => true.try_to_vec()?,
        QueryMsg::LastBlockNumber {} => state.last_block_number().try_to_vec()?,
        QueryMsg::BlockHashSafe { block_number } => {
            state.block_hash_safe(block_number).try_to_vec()?
        }
        QueryMsg::IsKnownExecutionHeader { hash } => state
            .is_known_execution_header(eth_types::H256::try_from_slice(hash.as_slice())?)
            .try_to_vec()?,
        QueryMsg::FinalizedBeaconBlockRoot {} => {
            state.finalized_beacon_block_root().try_to_vec()?
        }
        QueryMsg::FinalizedBeaconBlockSlot {} => {
            state.finalized_beacon_block_slot().try_to_vec()?
        }
        QueryMsg::FinalizedBeaconBlockHeader {} => {
            state.finalized_beacon_block_header().try_to_vec()?
        }
        QueryMsg::MinStorageBalanceForSubmitter {} => {
            state.min_storage_balance_for_submitter().try_to_vec()?
        }
        QueryMsg::GetLightClientState {} => state.get_light_client_state().try_to_vec()?,
        QueryMsg::IsSubmitterRegistered { account_id } => state
            .is_submitter_registered(near_sdk::AccountId::new_unchecked(account_id))
            .try_to_vec()?,
        QueryMsg::GetNumOfSubmittedBlocksByAccount { account_id } => state
            .get_num_of_submitted_blocks_by_account(near_sdk::AccountId::new_unchecked(account_id))
            .try_to_vec()?,
        QueryMsg::GetMaxSubmittedBlocksByAccount {} => {
            state.get_max_submitted_blocks_by_account().try_to_vec()?
        }
        QueryMsg::GetTrustedSigner {} => state.get_trusted_signer().try_to_vec()?,
    };
    Ok(to_binary(&GenericQueryResponse { borsh: res })?)
}
