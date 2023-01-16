use cosmwasm_std::{Addr, Deps};

use types::{
    eth2::{ExtendedBeaconBlockHeader, LightClientState},
    H256,
};

use super::Contract;

impl Contract<'_> {
    /// Returns finalized execution block number
    pub fn last_block_number(&self, deps: Deps) -> u64 {
        let non_mapped_state = self.state.non_mapped.load(deps.storage).unwrap();

        non_mapped_state
            .finalized_execution_header
            .unwrap()
            .block_number
    }

    /// Returns finalized execution block hash
    pub fn block_hash_safe(&self, deps: Deps, block_number: u64) -> Option<H256> {
        self.state
            .mapped
            .finalized_execution_blocks
            .load(deps.storage, block_number)
            .ok()
    }

    /// Checks if the execution header is already submitted.
    pub fn is_known_execution_header(&self, deps: Deps, hash: H256) -> bool {
        self.state
            .mapped
            .unfinalized_headers
            .has(deps.storage, hash.to_string())
    }

    /// Get finalized beacon block root
    pub fn finalized_beacon_block_root(&self, deps: Deps) -> H256 {
        let non_mapped_state = self.state.non_mapped.load(deps.storage).unwrap();

        non_mapped_state.finalized_beacon_header.beacon_block_root
    }

    /// Returns finalized beacon block slot
    pub fn finalized_beacon_block_slot(&self, deps: Deps) -> u64 {
        let non_mapped_state = self.state.non_mapped.load(deps.storage).unwrap();

        non_mapped_state.finalized_beacon_header.header.slot
    }

    /// Returns finalized beacon block header
    pub fn finalized_beacon_block_header(&self, deps: Deps) -> ExtendedBeaconBlockHeader {
        let non_mapped_state = self.state.non_mapped.load(deps.storage).unwrap();

        non_mapped_state.finalized_beacon_header
    }

    /// Get the current light client state
    pub fn get_light_client_state(&self, deps: Deps) -> LightClientState {
        let non_mapped_state = self.state.non_mapped.load(deps.storage).unwrap();

        LightClientState {
            finalized_beacon_header: non_mapped_state.finalized_beacon_header.clone(),
            current_sync_committee: non_mapped_state.current_sync_committee.clone().unwrap(),
            next_sync_committee: non_mapped_state.next_sync_committee.unwrap(),
        }
    }

    pub fn get_trusted_signer(&self, deps: Deps) -> Option<Addr> {
        let non_mapped_state = self.state.non_mapped.load(deps.storage).unwrap();

        non_mapped_state.trusted_signer
    }
}
