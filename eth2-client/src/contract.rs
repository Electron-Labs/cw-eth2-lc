#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;
use eth2_utility::types::InitInput;
use eth_types::{eth2::LightClientUpdate, BlockHeader};

use crate::{
    error::ContractError,
    msg::GenericQueryResponse,
    rainbow::{self, Context, Eth2Client},
};
use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::STATE,
};
use borsh::BorshDeserialize;
use borsh::BorshSerialize;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:eth2-client";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO remove accountId
// TODO remove all borsh from contract interface
// TODO remove near from crate

// TODO remove unwraps
// TODO use cosmwasm Maps, dont deserialize entire mapping for every call
// TODO rename crates - directory structure
// TODO remove uneeded deps
// TODO try types from substrate implementation - https://github.com/webb-tools/pallet-eth2-light-client
// TODO add logs

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let args = InitInput::try_from_slice(msg.borsh.as_slice())?;
    let client = rainbow::Eth2Client::init(
        Context {
            env,
            info: Some(info.clone()),
        },
        args,
    );

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    deps.storage
        .set(STATE, client.state.try_to_vec()?.as_slice());

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
    let state = rainbow::Eth2ClientState::try_from_slice(
        deps.storage
            .get(STATE)
            .ok_or(ContractError::Std(StdError::generic_err(
                "could not find state",
            )))?
            .as_slice(),
    )?;

    let mut client = Eth2Client {
        ctx: Context {
            env,
            info: Some(info),
        },
        state,
    };

    match msg {
        ExecuteMsg::RegisterSubmitter {} => client.register_submitter(),
        ExecuteMsg::UnRegisterSubmitter {} => client.unregister_submitter(),
        ExecuteMsg::SubmitBeaconChainLightClientUpdate { borsh } => client
            .submit_beacon_chain_light_client_update(LightClientUpdate::try_from_slice(
                borsh.as_slice(),
            )?),
        ExecuteMsg::SubmitExecutionHeader { borsh } => {
            client.submit_execution_header(BlockHeader::try_from_slice(borsh.as_slice())?)
        }
        ExecuteMsg::UpdateTrustedSigner { trusted_signer } => {
            client.update_trusted_signer(trusted_signer.map(near_sdk::AccountId::new_unchecked))
        }
    };

    deps.storage
        .set(STATE, client.state.try_to_vec()?.as_slice());

    Ok(Response::new().add_attribute("method", "execute"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let res = try_query(deps, env, msg).map_err(|e| StdError::generic_err(e.to_string()))?;
    Ok(res)
}

pub fn try_query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    let state = rainbow::Eth2ClientState::try_from_slice(
        deps.storage
            .get(STATE)
            .ok_or(ContractError::Std(StdError::generic_err(
                "could not find state",
            )))?
            .as_slice(),
    )?;

    let client = Eth2Client {
        ctx: Context { env, info: None },
        state,
    };
    let res = match msg {
        QueryMsg::IsInitialized {} => true.try_to_vec()?,
        QueryMsg::LastBlockNumber {} => client.last_block_number().try_to_vec()?,
        QueryMsg::BlockHashSafe { block_number } => {
            client.block_hash_safe(block_number).try_to_vec()?
        }
        QueryMsg::IsKnownExecutionHeader { hash } => client
            .is_known_execution_header(eth_types::H256::try_from_slice(hash.as_slice())?)
            .try_to_vec()?,
        QueryMsg::FinalizedBeaconBlockRoot {} => {
            client.finalized_beacon_block_root().try_to_vec()?
        }
        QueryMsg::FinalizedBeaconBlockSlot {} => {
            client.finalized_beacon_block_slot().try_to_vec()?
        }
        QueryMsg::FinalizedBeaconBlockHeader {} => {
            client.finalized_beacon_block_header().try_to_vec()?
        }
        QueryMsg::GetLightClientState {} => client.get_light_client_state().try_to_vec()?,
        QueryMsg::IsSubmitterRegistered { account_id } => client
            .is_submitter_registered(near_sdk::AccountId::new_unchecked(account_id))
            .try_to_vec()?,
        QueryMsg::GetNumOfSubmittedBlocksByAccount { account_id } => client
            .get_num_of_submitted_blocks_by_account(near_sdk::AccountId::new_unchecked(account_id))
            .try_to_vec()?,
        QueryMsg::GetMaxSubmittedBlocksByAccount {} => {
            client.get_max_submitted_blocks_by_account().try_to_vec()?
        }
        QueryMsg::GetTrustedSigner {} => client.get_trusted_signer().try_to_vec()?,
    };
    Ok(to_binary(&GenericQueryResponse { borsh: res })?)
}
