use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct NextSyncCommittee {
    pub sync_committee_ssz: Vec<u8>,
    pub sync_committee_poseidon_hash: Vec<u8>,
    pub sc_update_proof: String,
}

#[cw_serde]
pub struct LightClientUpdate {
    pub attested_slot: u64,
    pub finalized_slot: u64,
    pub participation: u64,
    pub finalized_header_root: Vec<u8>,
    pub execution_state_root: Vec<u8>,
    pub lc_update_proof: String,
    pub next_sync_committee: Option<NextSyncCommittee>,
}

#[cw_serde]
pub struct InitInput {
    pub admin: Addr,
    pub network: String,
    pub head_slot: u64,
    pub header_root: Vec<u8>,
    pub execution_state_root: Vec<u8>,
    pub sync_committee_poseidon_hash: Vec<u8>,
    pub vkey_lc_update_string: String,
    pub vkey_sc_update_string: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub init_input: InitInput,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateLightClient {
        light_client_update: LightClientUpdate,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(HeadResponse)]
    Head {},
    #[returns(HeaderRootResponse)]
    HeaderRoot { slot: u64 },
    #[returns(ExecutionStateRootResponse)]
    ExecutionStateRoot { slot: u64 },
    #[returns(SyncCommitteePoseidonHashResponse)]
    SyncCommitteePoseidonHash { period: u64 },
    #[returns(VerifyLogEntryResponse)]
    VerifyLogEntry {
        verify_log_entry_request: VerifyLogEntryRequest,
    },
}
#[cw_serde]
pub struct VerifyLogEntryRequest {
    pub log_index: u64,
    pub log_entry_data: Vec<u8>,
    pub receipt_index: u64,
    pub receipt_data: Vec<u8>,
    pub proof: Vec<Vec<u8>>,
    pub src_slot: u64,
    pub tx_slot: u64,
    pub receipts_root: Vec<u8>,
    pub receipts_root_proof: Vec<Vec<u8>>,
    pub skip_bridge_call: bool,
}

#[cw_serde]
pub struct HeadResponse {
    pub head: u64,
}

#[cw_serde]
pub struct HeaderRootResponse {
    pub header_root: Option<Vec<u8>>,
}

#[cw_serde]
pub struct ExecutionStateRootResponse {
    pub execution_state_root: Option<Vec<u8>>,
}

#[cw_serde]
pub struct SyncCommitteePoseidonHashResponse {
    pub sync_committee_poseidon_hash: Option<Vec<u8>>,
}

#[cw_serde]
pub struct VerifyLogEntryResponse {
    pub verified: bool,
}
