use crate::eth_utility::Network;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

use electron_rs::verifier::near::PreparedVerifyingKey;

const NON_MAPPED_STATE_KEY: &str = "non_mapped";
const NON_MAPPED_STATE_LC_KEY: &str = "non_mapped_lc";
const NON_MAPPED_STATE_SC_KEY: &str = "non_mapped_sc";
const HEADER_ROOTS: &str = "header_roots";
const EXECUTION_STATE_ROOTS: &str = "execution_state_roots";
const SYNC_COMMITTEE_POSEIDON_HASHES: &str = "sync_committee_poseidon_hashes";

pub struct ContractState<'a> {
    // state that is store in maps
    pub mapped: MappedState<'a>,

    // state that is not stored in maps
    pub non_mapped: Item<'a, NonMappedState>,
    pub non_mapped_lc: Item<'a, NonMappedStateLC>,
    pub non_mapped_sc: Item<'a, NonMappedStateSC>,
}

#[derive(Serialize, Deserialize)]
pub struct NonMappedState {
    pub admin: Addr,
    /// Network. e.g. mainnet, goerli,
    pub network: Network,
    /// Latest head slot
    pub head_slot: u64,
}

#[derive(Serialize, Deserialize)]
pub struct NonMappedStateLC {
    /// lc_update circuit verification key
    pub vkey_lc_update: PreparedVerifyingKey,
}

#[derive(Serialize, Deserialize)]
pub struct NonMappedStateSC {
    /// sc_update circuit verification key
    pub vkey_sc_update: PreparedVerifyingKey,
}

pub struct MappedState<'a> {
    /// Beacon block header roots  mapped to slot numbers
    pub header_roots: Map<'a, u64, Vec<u8>>,
    /// Execution state roots mapped to slot numbers
    pub execution_state_roots: Map<'a, u64, Vec<u8>>,
    /// Sync committee public keys poseidon hash mapped to period
    pub sync_committee_poseidon_hashes: Map<'a, u64, Vec<u8>>,
}

#[allow(clippy::new_without_default)]
impl ContractState<'_> {
    pub fn new() -> Self {
        Self {
            non_mapped: Item::new(NON_MAPPED_STATE_KEY),
            non_mapped_lc: Item::new(NON_MAPPED_STATE_LC_KEY),
            non_mapped_sc: Item::new(NON_MAPPED_STATE_SC_KEY),
            mapped: MappedState {
                header_roots: Map::new(HEADER_ROOTS),
                execution_state_roots: Map::new(EXECUTION_STATE_ROOTS),
                sync_committee_poseidon_hashes: Map::new(SYNC_COMMITTEE_POSEIDON_HASHES),
            },
        }
    }
}
