use cw_eth2_lc::msg::{
    ExecutionStateRootResponse, HeadResponse, HeaderRootResponse, LightClientUpdate,
    SyncCommitteePoseidonHashResponse,
};
use cw_eth2_lc::Result;

pub trait ContractInterface {
    // Execute
    fn update_light_client(&mut self, light_client_update: LightClientUpdate) -> Result<()>;

    // Query
    fn head(&self) -> Result<HeadResponse>;
    fn header_root(&self, slot: u64) -> Result<HeaderRootResponse>;
    fn execution_state_root(&self, slot: u64) -> Result<ExecutionStateRootResponse>;
    fn sync_committee_poseidon_hash(
        &self,
        period: u64,
    ) -> Result<SyncCommitteePoseidonHashResponse>;
}
