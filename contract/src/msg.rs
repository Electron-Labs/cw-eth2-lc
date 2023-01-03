use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use types::{
    eth2::{ExtendedBeaconBlockHeader, LightClientState, LightClientUpdate},
    BlockHeader, H256,
};
use utility::types::InitInput;

#[cw_serde]
pub struct InstantiateMsg(pub InitInput);

#[cw_serde]
pub enum ExecuteMsg {
    RegisterSubmitter,
    UnRegisterSubmitter,
    SubmitBeaconChainLightClientUpdate(LightClientUpdate),
    SubmitExecutionHeader(BlockHeader),
    UpdateTrustedSigner { trusted_signer: Option<Addr> },
    Reset(Box<InitInput>),
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
    IsKnownExecutionHeader { hash: H256 },
    #[returns(H256)]
    FinalizedBeaconBlockRoot,
    #[returns(u64)]
    FinalizedBeaconBlockSlot,
    #[returns(ExtendedBeaconBlockHeader)]
    FinalizedBeaconBlockHeader,
    #[returns(LightClientState)]
    GetLightClientState,
    #[returns(bool)]
    IsSubmitterRegistered { addr: Addr },
    #[returns(u32)]
    GetNumOfSubmittedBlocksByAccount { addr: Addr },
    #[returns(u32)]
    GetMaxSubmittedBlocksByAccount,
    #[returns(Option<Addr>)]
    GetTrustedSigner,
}
