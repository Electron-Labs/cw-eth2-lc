use cw_storage_plus::{Item, Map};

use utility::{consensus::Network, types::ExecutionHeaderInfo};

use cosmwasm_std::Addr;
use serde::{Deserialize, Serialize};
use types::{
    eth2::{ExtendedBeaconBlockHeader, SyncCommittee},
    H256,
};

pub struct ContractState<'a> {
    pub non_mapped: Item<'a, NonMappedState>,
    pub mapped: MappedState<'a>,
}

#[derive(Serialize, Deserialize)]
pub struct NonMappedState {
    pub initialized: bool,

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
    /// Hashes of the finalized execution blocks mapped to their numbers. Stores up to `hashes_gc_threshold` entries.
    /// Execution block number -> execution block hash
    /// Max number of unfinalized blocks allowed to be stored by one submitter account
    /// This value should be at least 32 blocks (1 epoch), but the recommended value is 1024 (32 epochs)
    pub max_submitted_blocks_by_account: u32,
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
    /// `Addr`s mapped to their number of submitted headers.
    /// Submitter account -> Num of submitted headers
    pub submitters: Map<'a, Addr, u32>,
}

impl ContractState<'_> {
    pub fn new() -> Self {
        Self {
            non_mapped: Item::new("non_mapped"),
            mapped: MappedState {
                // TODO better keys
                finalized_execution_blocks: Map::new("1"),
                unfinalized_headers: Map::new("2"),
                submitters: Map::new("3"),
            },
        }
    }
}
