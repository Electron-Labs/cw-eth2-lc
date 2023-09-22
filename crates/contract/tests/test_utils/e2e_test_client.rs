use super::contract_interface::ContractInterface;
use crate::test_utils::accounts;
use cosmwasm_std::{Addr, CosmosMsg, QueryRequest};
use cw_eth2_lc::entrypoint::{execute, instantiate, query};
use cw_eth2_lc::msg::{
    ExecuteMsg, ExecutionStateRootResponse, HeadResponse, HeaderRootResponse, InitInput,
    InstantiateMsg, LightClientUpdate, QueryMsg, SyncCommitteePoseidonHashResponse,
};
use cw_eth2_lc::Result;
use cw_multi_test::{App, ContractWrapper, Executor};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct E2ETestContractImplementation {
    contract_addr: Addr,
    app: App,
}

impl E2ETestContractImplementation {
    pub fn new(args: InitInput) -> Result<Self> {
        let mut app = App::default();
        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));
        println!("{:?}", code_id);
        let contract_addr = app.instantiate_contract(
            code_id,
            accounts(0),
            &InstantiateMsg { init_input: args },
            &[],
            "Contract",
            None,
        )?;
        println!("{:?}", contract_addr);
        Ok(Self { contract_addr, app })
    }
    pub fn query_smart_contract<Q: Serialize, R: DeserializeOwned>(&self, query: Q) -> Result<R> {
        let query_request = QueryRequest::Wasm(cosmwasm_std::WasmQuery::Smart {
            contract_addr: self.contract_addr.to_string(),
            msg: cosmwasm_std::Binary(serde_json::ser::to_vec(&query)?),
        });
        let resp = self.app.wrap().query(&query_request);
        Ok(resp?)
    }
}

impl ContractInterface for E2ETestContractImplementation {
    fn update_light_client(&mut self, light_client_update: LightClientUpdate) -> Result<()> {
        let msg = CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: self.contract_addr.to_string(),
            msg: cosmwasm_std::Binary(serde_json::ser::to_vec(&ExecuteMsg::UpdateLightClient {
                light_client_update,
            })?),
            funds: Vec::new(),
        });
        self.app.execute(accounts(0), msg)?;

        Ok(())
    }

    fn head(&self) -> Result<HeadResponse> {
        self.query_smart_contract(QueryMsg::Head {})
    }

    fn header_root(&self, slot: u64) -> Result<HeaderRootResponse> {
        self.query_smart_contract(QueryMsg::HeaderRoot { slot })
    }

    fn execution_state_root(&self, slot: u64) -> Result<ExecutionStateRootResponse> {
        self.query_smart_contract(QueryMsg::ExecutionStateRoot { slot })
    }

    fn sync_committee_poseidon_hash(
        &self,
        period: u64,
    ) -> Result<SyncCommitteePoseidonHashResponse> {
        self.query_smart_contract(QueryMsg::SyncCommitteePoseidonHash { period })
    }
}
