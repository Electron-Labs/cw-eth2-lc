use cosmwasm_std::Addr;

use types::{
    eth2::{ExtendedBeaconBlockHeader, LightClientState},
    H256,
};

use super::Contract;

impl Contract<'_> {
    /// Returns finalized execution block number
    pub fn last_block_number(&self) -> u64 {
        let deps = self.ctx.get_deps().unwrap();
        let non_mapped_state = self.state.non_mapped.load(deps.storage).unwrap();

        non_mapped_state
            .finalized_execution_header
            .unwrap()
            .block_number
    }

    /// Returns finalized execution block hash
    pub fn block_hash_safe(&self, block_number: u64) -> Option<H256> {
        let deps = self.ctx.get_deps().unwrap();

        Some(
            self.state
                .mapped
                .finalized_execution_blocks
                .load(deps.storage, block_number)
                .unwrap(),
        )
    }

    /// Checks if the execution header is already submitted.
    pub fn is_known_execution_header(&self, hash: H256) -> bool {
        let deps = self.ctx.get_deps().unwrap();
        let _non_mapped_state = self.state.non_mapped.load(deps.storage).unwrap();

        self.state
            .mapped
            .unfinalized_headers
            .has(deps.storage, hash.to_string())
    }

    /// Get finalized beacon block root
    pub fn finalized_beacon_block_root(&self) -> H256 {
        let deps = self.ctx.get_deps().unwrap();
        let non_mapped_state = self.state.non_mapped.load(deps.storage).unwrap();

        non_mapped_state.finalized_beacon_header.beacon_block_root
    }

    /// Returns finalized beacon block slot
    pub fn finalized_beacon_block_slot(&self) -> u64 {
        let deps = self.ctx.get_deps().unwrap();
        let non_mapped_state = self.state.non_mapped.load(deps.storage).unwrap();

        non_mapped_state.finalized_beacon_header.header.slot
    }

    /// Returns finalized beacon block header
    pub fn finalized_beacon_block_header(&self) -> ExtendedBeaconBlockHeader {
        let deps = self.ctx.get_deps().unwrap();
        let non_mapped_state = self.state.non_mapped.load(deps.storage).unwrap();

        non_mapped_state.finalized_beacon_header
    }

    /// Get the current light client state
    pub fn get_light_client_state(&self) -> LightClientState {
        let deps = self.ctx.get_deps().unwrap();
        let non_mapped_state = self.state.non_mapped.load(deps.storage).unwrap();

        LightClientState {
            finalized_beacon_header: non_mapped_state.finalized_beacon_header.clone(),
            current_sync_committee: non_mapped_state.current_sync_committee.clone().unwrap(),
            next_sync_committee: non_mapped_state.next_sync_committee.unwrap(),
        }
    }

    pub fn is_submitter_registered(&self, addr: Addr) -> bool {
        let deps = self.ctx.get_deps().unwrap();
        self.state.mapped.submitters.has(deps.storage, addr)
    }

    pub fn get_num_of_submitted_blocks_by_account(&self, addr: Addr) -> u32 {
        let deps = self.ctx.get_deps().unwrap();

        self.state
            .mapped
            .submitters
            .load(deps.storage, addr)
            .unwrap_or_else(|_| panic!("{}", "The account is not registered"))
    }

    pub fn get_max_submitted_blocks_by_account(&self) -> u32 {
        let deps = self.ctx.get_deps().unwrap();
        let non_mapped_state = self.state.non_mapped.load(deps.storage).unwrap();

        non_mapped_state.max_submitted_blocks_by_account
    }

    pub fn get_trusted_signer(&self) -> Option<Addr> {
        let deps = self.ctx.get_deps().unwrap();
        let non_mapped_state = self.state.non_mapped.load(deps.storage).unwrap();

        non_mapped_state.trusted_signer
    }
}
