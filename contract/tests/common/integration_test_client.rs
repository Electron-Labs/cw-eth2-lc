use super::contract_interface::ContractInterface;
use cosmrs::{
    abci::MsgData,
    bip32::{DerivationPath, Mnemonic},
    cosmwasm::{
        MsgExecuteContract, MsgInstantiateContract, MsgInstantiateContractResponse, MsgStoreCode,
        MsgStoreCodeResponse,
    },
    crypto::secp256k1::SigningKey,
    proto::{
        cosmos::{
            auth::v1beta1::{query_client::QueryClient, BaseAccount, QueryAccountRequest},
            base::{
                abci::v1beta1::{TxMsgData, TxResponse},
                tendermint::v1beta1::{
                    service_client::ServiceClient as TendermintClient, GetLatestBlockRequest,
                },
            },
            tx::v1beta1::{service_client::ServiceClient as TxClient, BroadcastTxRequest},
        },
        cosmwasm::wasm::v1::QuerySmartContractStateRequest,
    },
    tx::{self, Fee, Msg, SignDoc, SignerInfo},
    AccountId, Coin,
};

use cw_eth2_lc::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    Result,
};
use prost::Message;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs::{self, File},
    io::Read,
    str::FromStr,
};

use utility::types::InitInput;

pub const COSMOS_DP: &str = "m/44'/118'/0'/0/0";
pub const MNEMONIC: &str = "come fury another excite blue obtain throw rhythm enjoy pulse olive damage tomato mention patrol farm robot diesel doll execute vapor more theme flee";
pub const ENDPOINT: &str = "http://localhost:9090/";
pub const ADDR_PREFIX: &str = "wasm";

pub struct IntegrationTestContractImplementation {
    contract_addr: AccountId,
    client: CustomCosmosClient,
}

pub struct CustomCosmosClient {
    rt: tokio::runtime::Runtime,
    caller_priv_key: SigningKey,
    caller_address: AccountId,
    auth_query_client: QueryClient<tonic::transport::Channel>,
    tx_client: TxClient<tonic::transport::Channel>,
    tendermint_client: TendermintClient<tonic::transport::Channel>,
    sc_query_client:
        cosmrs::proto::cosmwasm::wasm::v1::query_client::QueryClient<tonic::transport::Channel>,
}

impl CustomCosmosClient {
    pub fn new() -> Result<Self> {
        let rt = tokio::runtime::Runtime::new()?;
        let seed = Mnemonic::new(MNEMONIC, Default::default())?.to_seed("");
        let caller_priv_key =
            SigningKey::derive_from_path(seed, &DerivationPath::from_str(COSMOS_DP)?)?;
        let caller_pub_key = caller_priv_key.public_key();
        let caller_address = caller_pub_key.account_id(ADDR_PREFIX)?;
        let auth_query_client = QueryClient::connect(ENDPOINT).wait(&rt)?;
        let tx_client = TxClient::connect(ENDPOINT).wait(&rt)?;
        let tendermint_client = TendermintClient::connect(ENDPOINT).wait(&rt)?;
        let sc_query_client =
            cosmrs::proto::cosmwasm::wasm::v1::query_client::QueryClient::connect(ENDPOINT)
                .wait(&rt)?;
        Ok(Self {
            rt,
            caller_priv_key,
            caller_address,
            auth_query_client,
            tx_client,
            tendermint_client,
            sc_query_client,
        })
    }

    fn broadcast_tx<M: Msg>(&mut self, msg: M) -> Result<TxResponse> {
        let acc_resp = self
            .auth_query_client
            .account(QueryAccountRequest {
                address: self.caller_address.to_string(),
            })
            .wait(&self.rt)?;

        let account_data =
            BaseAccount::decode(acc_resp.get_ref().account.as_ref().unwrap().value.as_ref())?;
        let latest_block_height = self
            .tendermint_client
            .get_latest_block(GetLatestBlockRequest {})
            .wait(&self.rt)?
            .get_ref()
            .block
            .as_ref()
            .unwrap()
            .header
            .as_ref()
            .unwrap()
            .height;

        // TODO add gas estimation
        let chain_id = "testing".parse()?;
        let account_number = account_data.account_number;
        let sequence_number = account_data.sequence;
        let gas = 10000000000_u64;
        let timeout_height = latest_block_height as u16 + 20;
        let memo = "cw-eth2-lc test";

        let tx_body = tx::Body::new(vec![msg.to_any()?], memo, timeout_height);

        let signer_info =
            SignerInfo::single_direct(Some(self.caller_priv_key.public_key()), sequence_number);
        let auth_info =
            signer_info.auth_info(Fee::from_amount_and_gas(Coin::new(0, "ucosm")?, gas));
        let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, account_number)?;
        let tx_signed = sign_doc.sign(&self.caller_priv_key)?;
        let tx_bytes = tx_signed.to_bytes()?;

        let res = self
            .tx_client
            .broadcast_tx(BroadcastTxRequest { tx_bytes, mode: 1 })
            .wait(&self.rt)?
            .get_ref()
            .clone()
            .tx_response
            .unwrap();

        let type_url = msg.to_any().unwrap().type_url;

        if res.code != 0 {
            return Err(
                format!("transaction unsuccessful - {} - {}", type_url, res.raw_log).into(),
            );
        }

        println!("transaction success - {type_url}");

        Ok(res)
    }

    fn broadcast_tx_with_resp<M: Msg, R: Msg>(&mut self, msg: M) -> Result<R> {
        let res: R = MsgData::try_from(
            TxMsgData::decode(hex::decode(self.broadcast_tx(msg)?.data.as_str())?.as_slice())?
                .data
                .first()
                .unwrap()
                .clone(),
        )?
        .try_decode_as()?;

        Ok(res)
    }
}

impl IntegrationTestContractImplementation {
    pub fn new(args: InitInput) -> Result<Self> {
        let mut client = CustomCosmosClient::new()?;

        let msg = MsgStoreCode {
            sender: client.caller_address.clone(),
            wasm_byte_code: get_file_as_byte_vec("../artifacts/cw_eth2_lc.wasm"),
            instantiate_permission: None,
        };
        let res: MsgStoreCodeResponse = client.broadcast_tx_with_resp(msg)?;

        let msg = MsgInstantiateContract {
            sender: client.caller_address.clone(),
            admin: None,
            code_id: res.code_id,
            label: Some("test label".to_string()),
            msg: serde_json::ser::to_vec(&InstantiateMsg { args })?,
            funds: Vec::new(),
        };
        let res: MsgInstantiateContractResponse = client.broadcast_tx_with_resp(msg)?;

        Ok(Self {
            contract_addr: res.address,
            client,
        })
    }
    pub fn query_smart_contract<Q: Serialize, R: DeserializeOwned>(&self, query: Q) -> Result<R> {
        let mut sc_query_client = self.client.sc_query_client.clone();
        let data = sc_query_client
            .smart_contract_state(QuerySmartContractStateRequest {
                address: self.contract_addr.to_string(),
                query_data: serde_json::ser::to_vec(&query)?,
            })
            .wait(&self.client.rt)?
            .get_ref()
            .data
            .clone();
        let res = serde_json::de::from_slice(&data)?;
        Ok(res)
    }
}

trait Block {
    fn wait(self, rt: &tokio::runtime::Runtime) -> <Self as futures::Future>::Output
    where
        Self: Sized,
        Self: futures::Future,
    {
        rt.block_on(self)
    }
}

impl<F, T> Block for F where F: futures::Future<Output = T> {}

fn get_file_as_byte_vec(filename: &str) -> Vec<u8> {
    let mut f = File::open(filename).expect("no file found");
    let metadata = fs::metadata(filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}

impl ContractInterface for IntegrationTestContractImplementation {
    fn register_submitter(&mut self) -> Result<()> {
        let msg = MsgExecuteContract {
            sender: self.client.caller_address.clone(),
            contract: self.contract_addr.clone(),
            msg: serde_json::ser::to_vec(&ExecuteMsg::RegisterSubmitter)?,
            funds: Vec::new(),
        };
        self.client.broadcast_tx(msg)?;

        Ok(())
    }

    fn unregister_submitter(&mut self) -> Result<()> {
        let msg = MsgExecuteContract {
            sender: self.client.caller_address.clone(),
            contract: self.contract_addr.clone(),
            msg: serde_json::ser::to_vec(&ExecuteMsg::UnRegisterSubmitter)?,
            funds: Vec::new(),
        };
        self.client.broadcast_tx(msg)?;

        Ok(())
    }

    fn submit_beacon_chain_light_client_update(
        &mut self,
        update: types::eth2::LightClientUpdate,
    ) -> Result<()> {
        let msg = MsgExecuteContract {
            sender: self.client.caller_address.clone(),
            contract: self.contract_addr.clone(),
            msg: serde_json::ser::to_vec(&ExecuteMsg::SubmitBeaconChainLightClientUpdate(update))?,
            funds: Vec::new(),
        };
        self.client.broadcast_tx(msg)?;

        Ok(())
    }

    fn submit_execution_header(&mut self, block_header: types::BlockHeader) -> Result<()> {
        let msg = MsgExecuteContract {
            sender: self.client.caller_address.clone(),
            contract: self.contract_addr.clone(),
            msg: serde_json::ser::to_vec(&ExecuteMsg::SubmitExecutionHeader(block_header))?,
            funds: Vec::new(),
        };
        self.client.broadcast_tx(msg)?;

        Ok(())
    }

    fn update_trusted_signer(&mut self, trusted_signer: Option<cosmwasm_std::Addr>) -> Result<()> {
        let msg = MsgExecuteContract {
            sender: self.client.caller_address.clone(),
            contract: self.contract_addr.clone(),
            msg: serde_json::ser::to_vec(&ExecuteMsg::UpdateTrustedSigner { trusted_signer })?,
            funds: Vec::new(),
        };
        self.client.broadcast_tx(msg)?;

        Ok(())
    }

    fn last_block_number(&self) -> Result<u64> {
        self.query_smart_contract(QueryMsg::LastBlockNumber)
    }

    fn block_hash_safe(&self, block_number: u64) -> Result<Option<types::H256>> {
        self.query_smart_contract(QueryMsg::BlockHashSafe { block_number })
    }

    fn is_known_execution_header(&self, hash: types::H256) -> Result<bool> {
        self.query_smart_contract(QueryMsg::IsKnownExecutionHeader { hash })
    }

    fn finalized_beacon_block_root(&self) -> Result<types::H256> {
        self.query_smart_contract(QueryMsg::FinalizedBeaconBlockRoot)
    }

    fn finalized_beacon_block_slot(&self) -> Result<u64> {
        self.query_smart_contract(QueryMsg::FinalizedBeaconBlockSlot)
    }

    fn finalized_beacon_block_header(&self) -> Result<types::eth2::ExtendedBeaconBlockHeader> {
        self.query_smart_contract(QueryMsg::FinalizedBeaconBlockHeader)
    }

    fn get_light_client_state(&self) -> Result<types::eth2::LightClientState> {
        self.query_smart_contract(QueryMsg::GetLightClientState)
    }

    fn is_submitter_registered(&self, addr: cosmwasm_std::Addr) -> Result<bool> {
        self.query_smart_contract(QueryMsg::IsSubmitterRegistered { addr })
    }

    fn get_num_of_submitted_blocks_by_account(&self, addr: cosmwasm_std::Addr) -> Result<u32> {
        self.query_smart_contract(QueryMsg::GetNumOfSubmittedBlocksByAccount { addr })
    }

    fn get_max_submitted_blocks_by_account(&self) -> Result<u32> {
        self.query_smart_contract(QueryMsg::GetMaxSubmittedBlocksByAccount)
    }

    fn get_trusted_signer(&self) -> Result<Option<cosmwasm_std::Addr>> {
        self.query_smart_contract(QueryMsg::GetTrustedSigner)
    }

    fn submit_and_check_execution_headers(&mut self, block_headers: Vec<&types::BlockHeader>) -> Result<()> {
        todo!()
    }
}
