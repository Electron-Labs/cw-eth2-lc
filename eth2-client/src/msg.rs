use cosmwasm_schema::{cw_serde, QueryResponses};
use eth2_utility::types::InitInput;

#[cw_serde]
pub struct InstantiateMsg {
    pub borsh: Vec<u8>,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterSubmitter {},
    UnRegisterSubmitter {},
    SubmitBeaconChainLightClientUpdate { borsh: Vec<u8> },
    SubmitExecutionHeader { borsh: Vec<u8> },
    UpdateTrustedSigner { trusted_signer: Option<String> },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // // GetCount returns the current count as a json-encoded number
    // GetCount {},
    #[returns(GenericQueryResponse)]
    IsInitialized {},
    #[returns(GenericQueryResponse)]
    LastBlockNumber {},
    #[returns(GenericQueryResponse)]
    BlockHashSafe { block_number: u64 },
    #[returns(GenericQueryResponse)]
    IsKnownExecutionHeader { hash: Vec<u8> },
    #[returns(GenericQueryResponse)]
    FinalizedBeaconBlockRoot {},
    #[returns(GenericQueryResponse)]
    FinalizedBeaconBlockSlot {},
    #[returns(GenericQueryResponse)]
    FinalizedBeaconBlockHeader {},
    #[returns(GenericQueryResponse)]
    MinStorageBalanceForSubmitter {},
    #[returns(GenericQueryResponse)]
    GetLightClientState {},
    #[returns(GenericQueryResponse)]
    IsSubmitterRegistered { account_id: String },
    #[returns(GenericQueryResponse)]
    GetNumOfSubmittedBlocksByAccount { account_id: String },
    #[returns(GenericQueryResponse)]
    GetMaxSubmittedBlocksByAccount {},
    #[returns(GenericQueryResponse)]
    GetTrustedSigner {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GenericQueryResponse {
    pub borsh: Vec<u8>,
}
