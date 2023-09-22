use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::Addr;
use cw_eth2_lc::contract::Contract;
use cw_eth2_lc::eth_utility::compute_sync_committee_period;
use cw_eth2_lc::msg::{
    ExecutionStateRootResponse, HeadResponse, HeaderRootResponse, InitInput, LightClientUpdate,
    SyncCommitteePoseidonHashResponse,
};

use super::contract_interface::ContractInterface;
use super::get_test_data;

#[cfg(feature = "e2e")]
use crate::test_utils::e2e_test_client::E2ETestContractImplementation;
#[cfg(not(feature = "e2e"))]
use crate::test_utils::unit_test_client::UnitTestContractImplementation;

pub struct TestContext<'a, 'b> {
    pub contract: Box<dyn ContractInterface + 'b>,
    pub lc_updates: &'a Vec<LightClientUpdate>,
    pub sc_updates: &'a Vec<LightClientUpdate>,
}

pub fn get_test_context<'a>(contract_caller: Addr) -> TestContext<'static, 'a> {
    let (init_input, lc_updates, sc_updates) = get_test_data();
    let contract = get_test_contract(contract_caller, init_input.clone());

    println!("{:?}", contract.head().unwrap());
    assert_eq!(
        contract.head().unwrap(),
        HeadResponse {
            head: init_input.head_slot
        }
    );
    assert_eq!(
        contract.header_root(init_input.head_slot).unwrap(),
        HeaderRootResponse {
            header_root: Some(init_input.header_root)
        }
    );
    assert_eq!(
        contract.execution_state_root(init_input.head_slot).unwrap(),
        ExecutionStateRootResponse {
            execution_state_root: Some(init_input.execution_state_root)
        }
    );
    assert_eq!(
        contract
            .sync_committee_poseidon_hash(compute_sync_committee_period(init_input.head_slot))
            .unwrap(),
        SyncCommitteePoseidonHashResponse {
            sync_committee_poseidon_hash: Some(init_input.sync_committee_poseidon_hash)
        }
    );

    TestContext {
        contract,
        lc_updates,
        sc_updates,
    }
}

#[cfg(not(feature = "e2e"))]
fn get_test_contract<'a>(
    contract_caller: Addr,
    init_input: InitInput,
) -> Box<dyn ContractInterface + 'a> {
    let contract = Contract::new(
        mock_env(),
        Some(mock_info(contract_caller.to_string().as_str(), &[])),
    );
    let mut contract = UnitTestContractImplementation {
        inner: contract,
        deps: mock_dependencies(),
    };
    contract.inner.init(contract.deps.as_mut(), init_input);
    Box::new(contract) as Box<dyn ContractInterface + 'a>
}

#[cfg(feature = "e2e")]
fn get_test_contract<'a>(
    _contract_caller: Addr,
    init_input: InitInput,
) -> Box<dyn ContractInterface + 'a> {
    let contract = Box::new(E2ETestContractImplementation::new(init_input).unwrap())
        as Box<dyn ContractInterface + 'a>;
    contract
}
