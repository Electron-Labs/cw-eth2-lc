use cosmwasm_std::testing::{mock_dependencies, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{Empty, OwnedDeps};
use cw_eth2_lc::contract::Contract;
use cw_eth2_lc::msg::{
    ExecutionStateRootResponse, HeadResponse, HeaderRootResponse, SyncCommitteePoseidonHashResponse,
};
use cw_eth2_lc::Result;

use super::contract_interface::ContractInterface;
pub struct UnitTestContractImplementation<'a> {
    pub inner: Contract<'a>,
    pub deps: OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
}

impl<'a> UnitTestContractImplementation<'a> {
    pub fn new(contract: Contract<'a>) -> UnitTestContractImplementation {
        Self {
            inner: contract,
            deps: mock_dependencies(),
        }
    }
}

impl ContractInterface for UnitTestContractImplementation<'_> {
    fn update_light_client(
        &mut self,
        light_client_update: cw_eth2_lc::msg::LightClientUpdate,
    ) -> Result<()> {
        self.inner
            .update_light_client(self.deps.as_mut(), light_client_update);
        Ok(())
    }

    fn head(&self) -> Result<HeadResponse> {
        Ok(self.inner.head(self.deps.as_ref()))
    }

    fn header_root(&self, slot: u64) -> Result<HeaderRootResponse> {
        Ok(self.inner.header_root(self.deps.as_ref(), slot))
    }

    fn execution_state_root(&self, slot: u64) -> Result<ExecutionStateRootResponse> {
        Ok(self.inner.execution_state_root(self.deps.as_ref(), slot))
    }

    fn sync_committee_poseidon_hash(
        &self,
        period: u64,
    ) -> Result<SyncCommitteePoseidonHashResponse> {
        Ok(self
            .inner
            .sync_committee_poseidon_hash(self.deps.as_ref(), period))
    }
}
