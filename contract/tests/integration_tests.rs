// use core::prelude;
// use std::{
//     fs::{self, File},
//     io::Read,
//     str::FromStr,
// };

// use cosmrs::{
//     abci::MsgData,
//     bip32::{DerivationPath, Mnemonic, PublicKey},
//     cosmwasm::{MsgStoreCode, MsgStoreCodeResponse},
//     crypto::secp256k1::SigningKey,
//     proto::cosmos::{
//         auth::{
//             self,
//             v1beta1::{query_client::QueryClient, BaseAccount, QueryAccountRequest},
//         },
//         base::abci::v1beta1::TxMsgData,
//         tx::v1beta1::{service_client::ServiceClient, BroadcastTxRequest},
//     },
//     tx::{self, Fee, Msg, SignDoc, SignerInfo},
//     AccountId, Any, Coin,
// };
// use prost::Message;

// pub const COSMOS_DP: &str = "m/44'/118'/0'/0/0";
// pub const MNEMONIC: &str = "come fury another excite blue obtain throw rhythm enjoy pulse olive damage tomato mention patrol farm robot diesel doll execute vapor more theme flee";
// pub const ENDPOINT: &str = "http://localhost:9090/";

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let seed = Mnemonic::new(MNEMONIC, Default::default())?.to_seed("");
//     let priv_key = SigningKey::derive_from_path(seed, &DerivationPath::from_str(COSMOS_DP)?)?;
//     let pub_key = priv_key.public_key();
//     let address = pub_key.account_id("wasm")?;

//     let msg = MsgStoreCode {
//         sender: address.clone(),
//         wasm_byte_code: get_file_as_byte_vec("./artifacts/cw_eth2_lc.wasm"),
//         instantiate_permission: None,
//     };

//     let res: MsgStoreCodeResponse = send_tx(&priv_key, msg).await?;
//     println!("store code - {}", res.code_id);

//     Ok(())
// }

// async fn send_tx<M: cosmrs::tx::Msg, R: cosmrs::tx::Msg>(
//     sender_priv_key: &SigningKey,
//     msg: M,
// ) -> Result<R, Box<dyn std::error::Error>> {
//     let pub_key = sender_priv_key.public_key();
//     let address = pub_key.account_id("wasm")?;

//     let acc_resp = QueryClient::connect(ENDPOINT)
//         .await?
//         .account(QueryAccountRequest {
//             address: address.to_string(),
//         })
//         .await?;

//     let account_data: BaseAccount =
//         prost::Message::decode(acc_resp.get_ref().account.as_ref().unwrap().value.as_ref())?;

//     let chain_id = "testing".parse()?;
//     let account_number = account_data.account_number;
//     let sequence_number = account_data.sequence;
//     let gas = 10000000000_u64;
//     let timeout_height = 9001u16;
//     let memo = "cw-eth2-lc test";

//     let tx_body = tx::Body::new(vec![msg.to_any()?], memo, timeout_height);

//     let signer_info = SignerInfo::single_direct(Some(pub_key), sequence_number);
//     let auth_info = signer_info.auth_info(Fee::from_amount_and_gas(Coin::new(0, "ucosm")?, gas));
//     let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, account_number)?;
//     let tx_signed = sign_doc.sign(sender_priv_key)?;
//     let tx_bytes = tx_signed.to_bytes()?;

//     let mut tx_client = ServiceClient::connect(ENDPOINT).await?;
//     let res = tx_client
//         .broadcast_tx(BroadcastTxRequest { tx_bytes, mode: 1 })
//         .await?
//         .get_ref()
//         .clone()
//         .tx_response
//         .unwrap();

//     println!("{}", res.raw_log);

//     let res: R = MsgData::try_from(
//         TxMsgData::decode(hex::decode(res.data.as_str())?.as_slice())?
//             .data
//             .first()
//             .unwrap()
//             .clone(),
//     )?
//     .try_decode_as()?;

//     Ok(res)
// }

// fn get_file_as_byte_vec(filename: &str) -> Vec<u8> {
//     let mut f = File::open(filename).expect("no file found");
//     let metadata = fs::metadata(filename).expect("unable to read metadata");
//     let mut buffer = vec![0; metadata.len() as usize];
//     f.read(&mut buffer).expect("buffer overflow");

//     buffer
// }
