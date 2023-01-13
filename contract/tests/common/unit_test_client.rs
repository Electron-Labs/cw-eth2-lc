use cosmwasm_std::{
    testing::{mock_dependencies, MockApi, MockQuerier, MockStorage},
    Addr, Empty, OwnedDeps,
};
use cw_eth2_lc::{contract::Contract, Result};
use std::panic::{catch_unwind, AssertUnwindSafe};
use types::{
    eth2::{ExtendedBeaconBlockHeader, LightClientState, LightClientUpdate},
    BlockHeader, H256,
};

use super::contract_interface::ContractInterface;
pub struct UnitTestContractImplementation<'a> {
    pub inner: Contract<'a>,
    pub deps: OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
}

impl<'a> UnitTestContractImplementation<'a> {
    pub fn new(contract: Contract<'a>) -> UnitTestContractImplementation {
        Self {
            inner: contract,
            deps: mock_dependencies(),
        }
    }
}

impl ContractInterface for UnitTestContractImplementation<'_> {
    fn register_submitter(&mut self) -> Result<()> {
        catch_unwind(AssertUnwindSafe(|| {
            self.inner.register_submitter(self.deps.as_mut())
        }))
        .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn unregister_submitter(&mut self) -> Result<()> {
        catch_unwind(AssertUnwindSafe(|| {
            self.inner.unregister_submitter(self.deps.as_mut())
        }))
        .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn submit_beacon_chain_light_client_update(&mut self, update: LightClientUpdate) -> Result<()> {
        catch_unwind(AssertUnwindSafe(|| {
            self.inner
                .submit_beacon_chain_light_client_update(self.deps.as_mut(), update)
        }))
        .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn submit_execution_header(&mut self, block_header: BlockHeader) -> Result<()> {
        catch_unwind(AssertUnwindSafe(|| {
            self.inner
                .submit_execution_header(self.deps.as_mut(), block_header)
        }))
        .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn update_trusted_signer(&mut self, trusted_signer: Option<Addr>) -> Result<()> {
        catch_unwind(AssertUnwindSafe(|| {
            self.inner
                .update_trusted_signer(self.deps.as_mut(), trusted_signer)
        }))
        .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn last_block_number(&self) -> Result<u64> {
        catch_unwind(AssertUnwindSafe(|| {
            self.inner.last_block_number(self.deps.as_ref())
        }))
        .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn block_hash_safe(&self, block_number: u64) -> Result<Option<H256>> {
        catch_unwind(AssertUnwindSafe(|| {
            self.inner.block_hash_safe(self.deps.as_ref(), block_number)
        }))
        .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn is_known_execution_header(&self, hash: H256) -> Result<bool> {
        catch_unwind(AssertUnwindSafe(|| {
            self.inner
                .is_known_execution_header(self.deps.as_ref(), hash)
        }))
        .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn finalized_beacon_block_root(&self) -> Result<H256> {
        catch_unwind(AssertUnwindSafe(|| {
            self.inner.finalized_beacon_block_root(self.deps.as_ref())
        }))
        .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn finalized_beacon_block_slot(&self) -> Result<u64> {
        catch_unwind(AssertUnwindSafe(|| {
            self.inner.finalized_beacon_block_slot(self.deps.as_ref())
        }))
        .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn finalized_beacon_block_header(&self) -> Result<ExtendedBeaconBlockHeader> {
        catch_unwind(AssertUnwindSafe(|| {
            self.inner.finalized_beacon_block_header(self.deps.as_ref())
        }))
        .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn get_light_client_state(&self) -> Result<LightClientState> {
        catch_unwind(AssertUnwindSafe(|| {
            self.inner.get_light_client_state(self.deps.as_ref())
        }))
        .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn is_submitter_registered(&self, addr: Addr) -> Result<bool> {
        catch_unwind(AssertUnwindSafe(|| {
            self.inner.is_submitter_registered(self.deps.as_ref(), addr)
        }))
        .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn get_num_of_submitted_blocks_by_account(&self, addr: Addr) -> Result<u32> {
        catch_unwind(AssertUnwindSafe(|| {
            self.inner
                .get_num_of_submitted_blocks_by_account(self.deps.as_ref(), addr)
        }))
        .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn get_max_submitted_blocks_by_account(&self) -> Result<u32> {
        catch_unwind(AssertUnwindSafe(|| {
            self.inner
                .get_max_submitted_blocks_by_account(self.deps.as_ref())
        }))
        .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn get_trusted_signer(&self) -> Result<Option<Addr>> {
        catch_unwind(AssertUnwindSafe(|| {
            self.inner.get_trusted_signer(self.deps.as_ref())
        }))
        .map_err(|_| "contract call panicked unexpectedly".into())
    }

    fn submit_and_check_execution_headers(
        &mut self,
        block_headers: Vec<&BlockHeader>,
    ) -> Result<()> {
        for header in block_headers {
            self.submit_execution_header(header.clone())?;
            if !self.is_known_execution_header(header.calculate_hash())? {
                return Err("failed to submit execution header".into());
            }
            if self.block_hash_safe(header.number)?.is_some() {
                return Err("failed to submit execution header".into());
            }
        }

        Ok(())
    }
}
