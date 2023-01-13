use crate::helpers::TryToBinary;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};


use crate::{
    contract::Contract,
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
};

// TODO optimize after reading eth2 light client spec
// TODO do we want to pause contract?
// TODO add logs
// TODO implement prover contract
// TODO remove all panics
// TODO test with all features enabled
// TODO use indexed map
// TODO optimised test speed

// TODO add proper responses
// TODO quoted_int could cause errors deserialize_str test data - somethine somewhere is trying to serialize with serde_json we need to find and remove it
// TODO remove uneeded features and deps
// TODO readme makes no sense
// TODO add docs
// TODO add gas to test
// TODO make test e2e
// TODO add comprehensive errors
// TODO add tests
// TODO refactor tests
// TODO REFACTOR
// TODO optimize rustfmt.toml
// TODO use standardized directory structure
// TODO add cicd
// TODO move todos to github issues
// TODO improve with awesome-cosmwasm
// TODO refer existing cosmwasm contracts
// TODO improve with rust tooling
// TODO add cosmwasm check to CI
// TODO use docker for integration test instead of testnet
// TODO add specific error to tests

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let mut contract = Contract::new(env, Some(info.clone()));
    contract.init(deps, msg.args);
    
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
    let contract = Contract::new(env, Some(info));

    match msg {
        ExecuteMsg::RegisterSubmitter => contract.register_submitter(deps),
        ExecuteMsg::UnRegisterSubmitter => contract.unregister_submitter(deps),
        ExecuteMsg::SubmitBeaconChainLightClientUpdate(light_client_update) => {
            contract.submit_beacon_chain_light_client_update(deps, light_client_update)
        }
        ExecuteMsg::SubmitExecutionHeader(block_header) => {
            contract.submit_execution_header(deps, block_header)
        }
        ExecuteMsg::UpdateTrustedSigner { trusted_signer } => {
            contract.update_trusted_signer(deps, trusted_signer)
        }
    };

    Ok(Response::new().add_attribute("method", "execute"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let res = try_query(deps, env, msg).map_err(|e| StdError::generic_err(e.to_string()))?;
    Ok(res)
}

pub fn try_query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    let contract = Contract::new(env, None);

    let res = match msg {
        QueryMsg::IsInitialized => true.try_to_binary()?,
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
        QueryMsg::IsSubmitterRegistered { addr } => contract
            .is_submitter_registered(deps, addr)
            .try_to_binary()?,
        QueryMsg::GetNumOfSubmittedBlocksByAccount { addr } => contract
            .get_num_of_submitted_blocks_by_account(deps, addr)
            .try_to_binary()?,
        QueryMsg::GetMaxSubmittedBlocksByAccount => contract
            .get_max_submitted_blocks_by_account(deps)
            .try_to_binary()?,
        QueryMsg::GetTrustedSigner => contract.get_trusted_signer(deps).try_to_binary()?,
    };

    Ok(res)
}
