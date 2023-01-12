use cosmwasm_std::Addr;
use cw_eth2_lc::{contract::Contract, Result};
use std::{
    cell::RefCell,
    panic::{catch_unwind, AssertUnwindSafe},
    rc::Rc,
};
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

impl ContractInterface for Contract<'_> {
    fn register_submitter(&mut self) -> Result<()> {
        let rc_self = AssertUnwindSafe(Rc::new(RefCell::new(self)));
        catch_unwind(|| rc_self.borrow_mut().register_submitter())
            .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn unregister_submitter(&mut self) -> Result<()> {
        let rc_self = AssertUnwindSafe(Rc::new(RefCell::new(self)));
        catch_unwind(|| rc_self.borrow_mut().unregister_submitter())
            .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn submit_beacon_chain_light_client_update(&mut self, update: LightClientUpdate) -> Result<()> {
        let rc_self = AssertUnwindSafe(Rc::new(RefCell::new(self)));
        catch_unwind(|| {
            rc_self
                .borrow_mut()
                .submit_beacon_chain_light_client_update(update)
        })
        .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn submit_execution_header(&mut self, block_header: BlockHeader) -> Result<()> {
        let rc_self = AssertUnwindSafe(Rc::new(RefCell::new(self)));
        catch_unwind(|| rc_self.borrow_mut().submit_execution_header(block_header))
            .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn update_trusted_signer(&mut self, trusted_signer: Option<Addr>) -> Result<()> {
        let rc_self = AssertUnwindSafe(Rc::new(RefCell::new(self)));
        catch_unwind(|| rc_self.borrow_mut().update_trusted_signer(trusted_signer))
            .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn last_block_number(&self) -> Result<u64> {
        let safe = AssertUnwindSafe(self);
        catch_unwind(|| safe.last_block_number())
            .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn block_hash_safe(&self, block_number: u64) -> Result<Option<H256>> {
        let safe = AssertUnwindSafe(self);
        catch_unwind(|| safe.block_hash_safe(block_number))
            .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn is_known_execution_header(&self, hash: H256) -> Result<bool> {
        let safe = AssertUnwindSafe(self);
        catch_unwind(|| safe.is_known_execution_header(hash))
            .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn finalized_beacon_block_root(&self) -> Result<H256> {
        let safe = AssertUnwindSafe(self);
        catch_unwind(|| safe.finalized_beacon_block_root())
            .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn finalized_beacon_block_slot(&self) -> Result<u64> {
        let safe = AssertUnwindSafe(self);
        catch_unwind(|| safe.finalized_beacon_block_slot())
            .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn finalized_beacon_block_header(&self) -> Result<ExtendedBeaconBlockHeader> {
        let safe = AssertUnwindSafe(self);
        catch_unwind(|| safe.finalized_beacon_block_header())
            .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn get_light_client_state(&self) -> Result<LightClientState> {
        let safe = AssertUnwindSafe(self);
        catch_unwind(|| safe.get_light_client_state())
            .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn is_submitter_registered(&self, addr: Addr) -> Result<bool> {
        let safe = AssertUnwindSafe(self);
        catch_unwind(|| safe.is_submitter_registered(addr))
            .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn get_num_of_submitted_blocks_by_account(&self, addr: Addr) -> Result<u32> {
        let safe = AssertUnwindSafe(self);
        catch_unwind(|| safe.get_num_of_submitted_blocks_by_account(addr))
            .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn get_max_submitted_blocks_by_account(&self) -> Result<u32> {
        let safe = AssertUnwindSafe(self);
        catch_unwind(|| safe.get_max_submitted_blocks_by_account())
            .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn get_trusted_signer(&self) -> Result<Option<Addr>> {
        let safe = AssertUnwindSafe(self);
        catch_unwind(|| safe.get_trusted_signer())
            .map_err(|_| "contract call panicked unexpectedly".into())
    }
}
