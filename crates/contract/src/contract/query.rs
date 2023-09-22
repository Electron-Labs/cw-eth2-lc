use super::Contract;
use crate::msg::{
    ExecutionStateRootResponse, HeadResponse, HeaderRootResponse, SyncCommitteePoseidonHashResponse,
};
use cosmwasm_std::Deps;

impl Contract<'_> {
    pub fn head(&self, deps: Deps) -> HeadResponse {
        let non_mapped_state = self.state.non_mapped.load(deps.storage).unwrap();
        HeadResponse {
            head: non_mapped_state.head_slot,
        }
    }

    pub fn header_root(&self, deps: Deps, slot: u64) -> HeaderRootResponse {
        HeaderRootResponse {
            header_root: self.state.mapped.header_roots.load(deps.storage, slot).ok(),
        }
    }

    pub fn execution_state_root(&self, deps: Deps, slot: u64) -> ExecutionStateRootResponse {
        ExecutionStateRootResponse {
            execution_state_root: self
                .state
                .mapped
                .execution_state_roots
                .load(deps.storage, slot)
                .ok(),
        }
    }

    pub fn sync_committee_poseidon_hash(
        &self,
        deps: Deps,
        period: u64,
    ) -> SyncCommitteePoseidonHashResponse {
        SyncCommitteePoseidonHashResponse {
            sync_committee_poseidon_hash: self
                .state
                .mapped
                .sync_committee_poseidon_hashes
                .load(deps.storage, period)
                .ok(),
        }
    }
}
