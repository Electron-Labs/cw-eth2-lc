use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use types::{
    eth2::{ExtendedBeaconBlockHeader, LightClientState, LightClientUpdate},
    BlockHeader, H256,
};
use utility::types::InitInput;

use crate::contract::admin_controlled::Mask;

#[cw_serde]
pub struct InstantiateMsg(pub InitInput);

#[cw_serde]
pub enum ExecuteMsg {
    SubmitBeaconChainLightClientUpdate(LightClientUpdate),
    SubmitExecutionHeader(BlockHeader),
    UpdateTrustedSigner { trusted_signer: Option<Addr> },
    SetPaused(Mask),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(u64)]
    LastBlockNumber,
    #[returns(Option<H256>)]
    BlockHashSafe { block_number: u64 },
    #[returns(bool)]
    IsKnownExecutionHeader { hash: H256 },
    #[returns(H256)]
    FinalizedBeaconBlockRoot,
    #[returns(u64)]
    FinalizedBeaconBlockSlot,
    #[returns(ExtendedBeaconBlockHeader)]
    FinalizedBeaconBlockHeader,
    #[returns(LightClientState)]
    GetLightClientState,
    #[returns(Option<Addr>)]
    GetTrustedSigner,
    #[returns(bool)]
    VerifyLogEntry(VerifyLogEntryRequest),
    #[returns(Mask)]
    GetPaused,
}
#[cw_serde]
pub struct VerifyLogEntryRequest {
    pub log_index: u64,
    pub log_entry_data: Vec<u8>,
    pub receipt_index: u64,
    pub receipt_data: Vec<u8>,
    pub header_data: Vec<u8>,
    pub proof: Vec<Vec<u8>>,
    pub skip_bridge_call: bool,
}
