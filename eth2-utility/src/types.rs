
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use eth_types::eth2::*;
use eth_types::H256;

/// Minimal information about a header.
#[cw_serde]
pub struct ExecutionHeaderInfo {
    pub parent_hash: H256,
    pub block_number: u64,
    pub submitter: Addr,
}

#[cw_serde]
pub struct InitInput {
    pub network: String,
    pub finalized_execution_header: eth_types::BlockHeader,
    pub finalized_beacon_header: ExtendedBeaconBlockHeader,
    pub current_sync_committee: SyncCommittee,
    pub next_sync_committee: SyncCommittee,
    pub validate_updates: bool,
    pub verify_bls_signatures: bool,
    pub hashes_gc_threshold: u64,
    pub max_submitted_blocks_by_account: u32,
    pub trusted_signer: Option<Addr>,
}
