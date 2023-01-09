use crate::{helpers::TryToBinary, state::ContractState};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};
use cw2::set_contract_version;

use crate::{
    contract::{Contract, ContractContext},
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::STATE,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:eth2-client";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO test on testnet, try no std crates from this if things dont workout - https://github.com/webb-tools/pallet-eth2-light-client
// TODO prevent reinstantiation attacks
// TODO optimize after reading cosmwasm docs and eth2 light client spec
// TODO use cosmwasm Maps, dont deserialize entire mapping for every call
// TODO do we want to pause contract?
// TODO add logs
// TODO remove unwraps
// TODO implement prover contract
// TODO conditionally compile reset endpoint
// TODO quoted_int could cause errors deserialize_str test data
// TODO remove all panics

// TODO readme makes no sense
// TODO add docs
// TODO add gas to test
// TODO make test e2e
// TODO remove custom jsonschema implements or make them typesafe so they dont break when types are changed - make macro?
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
    let mut contract = Contract::new(
        ContractContext {
            env,
            info: Some(info.clone()),
        },
        ContractState::default(),
    );

    contract.init(msg.args);

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
    let mut contract = Contract::new(
        ContractContext {
            env,
            info: Some(info),
        },
        STATE.load(deps.storage)?,
    );

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
    let contract = Contract::new(
        ContractContext { env, info: None },
        STATE.load(deps.storage)?,
    );
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
