use cosmwasm_schema::{cw_serde, QueryResponses};
use eth2_utility::types::InitInput;
use eth_types::{
    eth2::{ExtendedBeaconBlockHeader, LightClientState, LightClientUpdate},
    BlockHeader, H256,
};
use near_sdk::AccountId;

#[cw_serde]
pub struct InstantiateMsg(pub InitInput);

#[cw_serde]
pub enum ExecuteMsg {
    RegisterSubmitter,
    UnRegisterSubmitter,
    SubmitBeaconChainLightClientUpdate(LightClientUpdate),
    SubmitExecutionHeader(BlockHeader),
    UpdateTrustedSigner { trusted_signer: Option<String> },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(bool)]
    IsInitialized,
    #[returns(u64)]
    LastBlockNumber,
    #[returns(Option<H256>)]
    BlockHashSafe { block_number: u64 },
    #[returns(bool)]
    IsKnownExecutionHeader { hash: Vec<u8> },
    #[returns(H256)]
    FinalizedBeaconBlockRoot,
    #[returns(u64)]
    FinalizedBeaconBlockSlot,
    #[returns(ExtendedBeaconBlockHeader)]
    FinalizedBeaconBlockHeader,
    #[returns(LightClientState)]
    GetLightClientState,
    #[returns(bool)]
    IsSubmitterRegistered { account_id: String },
    #[returns(u32)]
    GetNumOfSubmittedBlocksByAccount { account_id: String },
    #[returns(u32)]
    GetMaxSubmittedBlocksByAccount,
    #[returns(Option<AccountId>)]
    GetTrustedSigner,
}
