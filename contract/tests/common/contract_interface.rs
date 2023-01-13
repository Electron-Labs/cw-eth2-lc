use cosmwasm_std::{
    Addr,
};
use cw_eth2_lc::{Result};

use types::{
    eth2::{ExtendedBeaconBlockHeader, LightClientState, LightClientUpdate},
    BlockHeader, H256,
};

pub trait ContractInterface {
    // Execute
    fn register_submitter(&mut self) -> Result<()>;
    fn unregister_submitter(&mut self) -> Result<()>;
    fn submit_beacon_chain_light_client_update(&mut self, update: LightClientUpdate) -> Result<()>;
    fn submit_execution_header(&mut self, block_header: BlockHeader) -> Result<()>;
    fn update_trusted_signer(&mut self, trusted_signer: Option<Addr>) -> Result<()>;

    // Query
    fn last_block_number(&self) -> Result<u64>;
    fn block_hash_safe(&self, block_number: u64) -> Result<Option<H256>>;
    fn is_known_execution_header(&self, hash: H256) -> Result<bool>;
    fn finalized_beacon_block_root(&self) -> Result<H256>;
    fn finalized_beacon_block_slot(&self) -> Result<u64>;
    fn finalized_beacon_block_header(&self) -> Result<ExtendedBeaconBlockHeader>;
    fn get_light_client_state(&self) -> Result<LightClientState>;
    fn is_submitter_registered(&self, addr: Addr) -> Result<bool>;
    fn get_num_of_submitted_blocks_by_account(&self, addr: Addr) -> Result<u32>;
    fn get_max_submitted_blocks_by_account(&self) -> Result<u32>;
    fn get_trusted_signer(&self) -> Result<Option<Addr>>;
}
