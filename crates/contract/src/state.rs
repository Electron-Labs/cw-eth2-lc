use crate::contract::admin_controlled::Mask;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};
use types::{
    eth2::{ExtendedBeaconBlockHeader, SyncCommittee},
    H256,
};
use utility::{consensus::Network, types::ExecutionHeaderInfo};

const NON_MAPPED_STATE_KEY: &str = "non_mapped";
const FINALIZED_EXECUTION_BLOCKS_STATE_KEY: &str = "finalized_execution_blocks";
const UNFINALIZED_HEADERS_STATE_KEY: &str = "unfinalized_headers";

pub struct ContractState<'a> {
    // state that is store in maps
    pub mapped: MappedState<'a>,

    // state that is not stored in maps
    pub non_mapped: Item<'a, NonMappedState>,
}

#[derive(Serialize, Deserialize)]
pub struct NonMappedState {
    pub admin: Addr,
    pub paused: Mask,
    /// If set, only light client updates by the trusted signer will be accepted
    pub trusted_signer: Option<Addr>,
    /// Whether the client validates the updates.
    /// Should only be set to `false` for debugging, testing, and diagnostic purposes
    pub validate_updates: bool,
    /// Whether the client verifies BLS signatures.
    pub verify_bls_signatures: bool,
    /// We store the hashes of the blocks for the past `hashes_gc_threshold` headers.
    /// Events that happen past this threshold cannot be verified by the client.
    /// It is desirable that this number is larger than 7 days' worth of headers, which is roughly
    /// 51k Ethereum blocks. So this number should be 51k in production.
    pub hashes_gc_threshold: u64,
    /// Network. e.g. mainnet, kiln
    pub network: Network,
    /// Light client state
    pub finalized_beacon_header: ExtendedBeaconBlockHeader,
    pub finalized_execution_header: Option<ExecutionHeaderInfo>,
    pub current_sync_committee: Option<SyncCommittee>,
    pub next_sync_committee: Option<SyncCommittee>,
}

pub struct MappedState<'a> {
    pub finalized_execution_blocks: Map<'a, u64, H256>,
    /// All unfinalized execution blocks' headers hashes mapped to their `HeaderInfo`.
    /// Execution block hash -> ExecutionHeaderInfo object
    pub unfinalized_headers: Map<'a, String, ExecutionHeaderInfo>,
}

impl ContractState<'_> {
    pub fn new() -> Self {
        Self {
            non_mapped: Item::new(NON_MAPPED_STATE_KEY),
            mapped: MappedState {
                finalized_execution_blocks: Map::new(FINALIZED_EXECUTION_BLOCKS_STATE_KEY),
                unfinalized_headers: Map::new(UNFINALIZED_HEADERS_STATE_KEY),
            },
        }
    }
}
