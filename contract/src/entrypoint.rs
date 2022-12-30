use crate::helpers::TryToBinary;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};
use cw2::set_contract_version;

use crate::{
    contract::{self, Contract, ContractContext},
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::STATE,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:eth2-client";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO remove unwraps
// TODO use cosmwasm Maps, dont deserialize entire mapping for every call
// TODO remove uneeded deps
// TODO try types from substrate implementation - https://github.com/webb-tools/pallet-eth2-light-client
// TODO add logs
// TODO prevent reinstantiation attacks
// TODO remove custom jsonschema implements or make the typesafe so they dont break when types are changed - make macro?
// TODO add comprehensive errors
// TODO REFACTOR

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let contract = contract::Contract::init(
        ContractContext {
            env,
            info: Some(info.clone()),
        },
        msg.0,
    );

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &contract.state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let mut contract = Contract {
        ctx: ContractContext {
            env,
            info: Some(info),
        },
        state: STATE.load(deps.storage)?,
    };

    match msg {
        ExecuteMsg::RegisterSubmitter => contract.register_submitter(),
        ExecuteMsg::UnRegisterSubmitter => contract.unregister_submitter(),
        ExecuteMsg::SubmitBeaconChainLightClientUpdate(light_client_update) => {
            contract.submit_beacon_chain_light_client_update(light_client_update)
        }
        ExecuteMsg::SubmitExecutionHeader(block_header) => {
            contract.submit_execution_header(block_header)
        }
        ExecuteMsg::UpdateTrustedSigner { trusted_signer } => {
            contract.update_trusted_signer(trusted_signer)
        }
    };

    STATE.save(deps.storage, &contract.state)?;

    Ok(Response::new().add_attribute("method", "execute"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let res = try_query(deps, env, msg).map_err(|e| StdError::generic_err(e.to_string()))?;
    Ok(res)
}

pub fn try_query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    let contract = Contract {
        ctx: ContractContext { env, info: None },
        state: STATE.load(deps.storage)?,
    };
    let res = match msg {
        QueryMsg::IsInitialized => true.try_to_binary()?,
        QueryMsg::LastBlockNumber => contract.last_block_number().try_to_binary()?,
        QueryMsg::BlockHashSafe { block_number } => {
            contract.block_hash_safe(block_number).try_to_binary()?
        }
        QueryMsg::IsKnownExecutionHeader { hash } => {
            contract.is_known_execution_header(hash).try_to_binary()?
        }
        QueryMsg::FinalizedBeaconBlockRoot => {
            contract.finalized_beacon_block_root().try_to_binary()?
        }
        QueryMsg::FinalizedBeaconBlockSlot => {
            contract.finalized_beacon_block_slot().try_to_binary()?
        }
        QueryMsg::FinalizedBeaconBlockHeader => {
            contract.finalized_beacon_block_header().try_to_binary()?
        }
        QueryMsg::GetLightClientState => contract.get_light_client_state().try_to_binary()?,
        QueryMsg::IsSubmitterRegistered { addr } => {
            contract.is_submitter_registered(addr).try_to_binary()?
        }
        QueryMsg::GetNumOfSubmittedBlocksByAccount { addr } => contract
            .get_num_of_submitted_blocks_by_account(addr)
            .try_to_binary()?,
        QueryMsg::GetMaxSubmittedBlocksByAccount => contract
            .get_max_submitted_blocks_by_account()
            .try_to_binary()?,
        QueryMsg::GetTrustedSigner => contract.get_trusted_signer().try_to_binary()?,
    };

    Ok(res)
}
