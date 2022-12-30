use cosmwasm_std::Addr;

use types::{
    eth2::{ExtendedBeaconBlockHeader, LightClientState},
    H256,
};

use super::Contract;

impl Contract {
    /// Returns finalized execution block number
    pub fn last_block_number(&self) -> u64 {
        self.state
            .finalized_execution_header
            .clone()
            .unwrap()
            .block_number
    }

    /// Returns finalized execution block hash
    pub fn block_hash_safe(&self, block_number: u64) -> Option<H256> {
        self.state
            .finalized_execution_blocks
            .get(&block_number)
            .copied()
    }

    /// Checks if the execution header is already submitted.
    pub fn is_known_execution_header(&self, hash: H256) -> bool {
        self.state
            .unfinalized_headers
            .get(&hash.to_string())
            .is_some()
    }

    /// Get finalized beacon block root
    pub fn finalized_beacon_block_root(&self) -> H256 {
        self.state.finalized_beacon_header.beacon_block_root
    }

    /// Returns finalized beacon block slot
    pub fn finalized_beacon_block_slot(&self) -> u64 {
        self.state.finalized_beacon_header.header.slot
    }

    /// Returns finalized beacon block header
    pub fn finalized_beacon_block_header(&self) -> ExtendedBeaconBlockHeader {
        self.state.finalized_beacon_header.clone()
    }

    /// Get the current light client state
    pub fn get_light_client_state(&self) -> LightClientState {
        LightClientState {
            finalized_beacon_header: self.state.finalized_beacon_header.clone(),
            current_sync_committee: self.state.current_sync_committee.clone().unwrap(),
            next_sync_committee: self.state.next_sync_committee.clone().unwrap(),
        }
    }

    pub fn is_submitter_registered(&self, addr: Addr) -> bool {
        self.state.submitters.contains_key(&addr)
    }

    pub fn get_num_of_submitted_blocks_by_account(&self, addr: Addr) -> u32 {
        *self
            .state
            .submitters
            .get(&addr)
            .unwrap_or_else(|| panic!("{}", "The account is not registered"))
    }

    pub fn get_max_submitted_blocks_by_account(&self) -> u32 {
        self.state.max_submitted_blocks_by_account
    }

    pub fn get_trusted_signer(&self) -> Option<Addr> {
        self.state.trusted_signer.clone()
    }
}
