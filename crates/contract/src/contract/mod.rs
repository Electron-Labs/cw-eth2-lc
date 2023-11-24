pub mod execute;
pub mod instantiate;
pub mod prover;
pub mod query;

use crate::eth_utility::compute_sync_committee_period;
use crate::msg::{LightClientUpdate, NextSyncCommittee};
use crate::state::ContractState;
use cosmwasm_std::{Attribute, Deps, Env, MessageInfo, Response};
use electron_rs::verifier::near::verify_proof;
use ethereum_types::U256;
use sha2::{Digest, Sha256};
use std::cell::RefCell;

pub struct ContractContext {
    pub env: Env,
    pub info: Option<MessageInfo>,
}

impl ContractContext {
    pub fn new(env: Env, info: Option<MessageInfo>) -> Self {
        Self { env, info }
    }
}

pub struct Contract<'a> {
    pub ctx: ContractContext,
    pub state: ContractState<'a>,
    logs: RefCell<Vec<String>>,
}

impl Contract<'_> {
    pub fn new(env: Env, info: Option<MessageInfo>) -> Self {
        Self {
            ctx: ContractContext::new(env, info),
            state: ContractState::new(),
            logs: RefCell::new(vec![]),
        }
    }

    // attach logs to instruction response
    pub fn response_with_logs(&self, mut res: Response) -> Response {
        for log in self.logs.borrow().iter() {
            res.attributes.push(Attribute::new("log", log));
        }

        res
    }

    fn lc_update_proof_verify(&self, deps: Deps, light_client_update: LightClientUpdate) {
        let mut attested_slot_le = vec![0u8; 8];
        attested_slot_le.clone_from_slice(&light_client_update.attested_slot.to_le_bytes());
        attested_slot_le.extend_from_slice(&[0u8; 24]);

        let mut finalized_slot_le = vec![0u8; 8];
        finalized_slot_le.clone_from_slice(&light_client_update.finalized_slot.to_le_bytes());
        finalized_slot_le.extend_from_slice(&[0u8; 24]);

        let mut participation_le = vec![0u8; 8];
        participation_le.copy_from_slice(&light_client_update.participation.to_le_bytes());
        participation_le.extend_from_slice(&[0u8; 24]);

        let finalized_period = compute_sync_committee_period(light_client_update.attested_slot);

        let sync_committee_poseidon = match self
            .state
            .mapped
            .sync_committee_poseidon_hashes
            .load(deps.storage, finalized_period)
            .ok()
        {
            Some(hash) => hash,
            None => panic!("Sync committee hash not known"),
        };

        let mut hasher = Sha256::new();
        let mut sha256_input = [attested_slot_le, finalized_slot_le].concat();
        hasher.update(&sha256_input);
        let mut hash = hasher.finalize().to_vec();

        hasher = Sha256::new();
        sha256_input = [hash, light_client_update.finalized_header_root].concat();
        hasher.update(&sha256_input);
        hash = hasher.finalize().to_vec();

        hasher = Sha256::new();
        sha256_input = [hash.to_vec(), participation_le].concat();
        hasher.update(&sha256_input);
        hash = hasher.finalize().to_vec();

        hasher = Sha256::new();
        sha256_input = [hash, light_client_update.execution_state_root].concat();
        hasher.update(&sha256_input);
        hash = hasher.finalize().to_vec();

        hasher = Sha256::new();
        sha256_input = [hash, sync_committee_poseidon].concat();
        hasher.update(&sha256_input);
        hash = hasher.finalize().to_vec();

        hash[31] &= 0x1f;

        let hash_and_mask = U256::from_little_endian(&hash);

        let public_inputs = format!("{:?}", vec![hash_and_mask.to_string()]);

        let non_mapped_state_lc = self.state.non_mapped_lc.load(deps.storage).unwrap();
        let vkey_lc_update = non_mapped_state_lc.vkey_lc_update;

        assert!(
            verify_proof(
                vkey_lc_update,
                light_client_update.lc_update_proof,
                public_inputs
            )
            .unwrap(),
            "Failed to verify lc_update proof"
        );
    }

    fn sc_update_proof_verify(&self, deps: Deps, light_client_update: LightClientUpdate) {
        let NextSyncCommittee {
            sync_committee_ssz,
            sync_committee_poseidon_hash,
            sc_update_proof,
        } = light_client_update.next_sync_committee.unwrap();
        let mut public_inputs = vec![String::new(); 65];
        for i in 0..32 {
            public_inputs[i] = sync_committee_ssz[i].to_string();
        }
        public_inputs[32] = U256::from_little_endian(&sync_committee_poseidon_hash).to_string();
        for (i, inp) in public_inputs.iter_mut().enumerate().take(65).skip(33) {
            *inp = light_client_update.finalized_header_root[i - 33].to_string();
        }
        let public_inputs = format!("{:?}", public_inputs);

        let non_mapped_state_sc = self.state.non_mapped_sc.load(deps.storage).unwrap();
        let vkey_sc_update = non_mapped_state_sc.vkey_sc_update;

        assert!(
            verify_proof(vkey_sc_update, sc_update_proof, public_inputs).unwrap(),
            "Failed to verify sc_update proof"
        );
    }
}
