pub mod execute;
pub mod instantiate;
pub mod query;

use std::collections::HashMap;

use bitvec::{order::Lsb0, prelude::BitVec};
use cosmwasm_std::{Addr, Env, MessageInfo};
use tree_hash::TreeHash;

use types::eth2::{ExtendedBeaconBlockHeader, LightClientUpdate};
use utility::consensus::{
    compute_sync_committee_period, convert_branch, validate_beacon_block_header_update,
    FINALITY_TREE_DEPTH, FINALITY_TREE_INDEX, MIN_SYNC_COMMITTEE_PARTICIPANTS,
    SYNC_COMMITTEE_TREE_DEPTH, SYNC_COMMITTEE_TREE_INDEX,
};

use crate::state::ContractState;

pub struct Contract {
    pub ctx: ContractContext,
    pub state: ContractState,
}

pub struct ContractContext {
    pub env: Env,
    pub info: Option<MessageInfo>,
}

impl Contract {
    fn validate_light_client_update(&self, update: &LightClientUpdate) {
        #[cfg(feature = "logs")]
        env::log_str(format!("Validate update. Used gas: {}", env::used_gas().0).as_str());

        let finalized_period =
            compute_sync_committee_period(self.state.finalized_beacon_header.header.slot);
        self.verify_finality_branch(update, finalized_period);

        // Verify sync committee has sufficient participants
        let sync_committee_bits =
            BitVec::<u8, Lsb0>::from_slice(&update.sync_aggregate.sync_committee_bits.0);
        let sync_committee_bits_sum: u64 = sync_committee_bits.count_ones().try_into().unwrap();

        assert!(
            sync_committee_bits_sum >= MIN_SYNC_COMMITTEE_PARTICIPANTS,
            "{}",
            format!("Invalid sync committee bits sum: {sync_committee_bits_sum}").as_str()
        );

        assert!(
            sync_committee_bits_sum * 3 >= (sync_committee_bits.len() * 2).try_into().unwrap(),
            "{}",format!(
                "Sync committee bits sum is less than 2/3 threshold, bits sum: {sync_committee_bits_sum}"
            )
        );

        #[cfg(feature = "bls")]
        if self.state.verify_bls_signatures {
            self.state
                .verify_bls_signatures(update, sync_committee_bits, finalized_period);
        }

        #[cfg(feature = "logs")]
        env::log_str(format!("Finish validate update. Used gas: {}", env::used_gas().0).as_str());
    }

    fn verify_finality_branch(&self, update: &LightClientUpdate, finalized_period: u64) {
        // The active header will always be the finalized header because we don't accept updates without the finality update.
        let active_header = &update.finality_update.header_update.beacon_header;

        assert!(
            active_header.slot > self.state.finalized_beacon_header.header.slot,
            "The active header slot number should be higher than the finalized slot"
        );

        assert!(
            update.attested_beacon_header.slot
                >= update.finality_update.header_update.beacon_header.slot,
            "The attested header slot should be equal to or higher than the finalized header slot"
        );

        assert!(
            update.signature_slot > update.attested_beacon_header.slot,
            "The signature slot should be higher than the attested header slot"
        );

        let update_period = compute_sync_committee_period(active_header.slot);
        assert!(
            update_period == finalized_period || update_period == finalized_period + 1,
            "{}",
            format!(
                "The acceptable update periods are '{}' and '{}' but got {}",
                finalized_period,
                finalized_period + 1,
                update_period
            )
            .as_str()
        );

        // Verify that the `finality_branch`, confirms `finalized_header`
        // to match the finalized checkpoint root saved in the state of `attested_header`.
        let branch = convert_branch(&update.finality_update.finality_branch);
        assert!(
            merkle_proof::verify_merkle_proof(
                update
                    .finality_update
                    .header_update
                    .beacon_header
                    .tree_hash_root(),
                &branch,
                FINALITY_TREE_DEPTH.try_into().unwrap(),
                FINALITY_TREE_INDEX.try_into().unwrap(),
                update.attested_beacon_header.state_root.0
            ),
            "Invalid finality proof"
        );
        assert!(
            validate_beacon_block_header_update(&update.finality_update.header_update),
            "Invalid execution block hash proof"
        );

        // Verify that the `next_sync_committee`, if present, actually is the next sync committee saved in the
        // state of the `active_header`
        if update_period != finalized_period {
            let sync_committee_update = update
                .sync_committee_update
                .as_ref()
                .unwrap_or_else(|| panic!("{}", "The sync committee update is missed"));
            let branch = convert_branch(&sync_committee_update.next_sync_committee_branch);
            assert!(
                merkle_proof::verify_merkle_proof(
                    sync_committee_update.next_sync_committee.tree_hash_root(),
                    &branch,
                    SYNC_COMMITTEE_TREE_DEPTH.try_into().unwrap(),
                    SYNC_COMMITTEE_TREE_INDEX.try_into().unwrap(),
                    active_header.state_root.0
                ),
                "Invalid next sync committee proof"
            );
        }
    }

    #[cfg(feature = "bls")]
    fn verify_bls_signatures(
        &self,
        update: &LightClientUpdate,
        sync_committee_bits: BitVec<u8>,
        finalized_period: u64,
    ) {
        let config = NetworkConfig::new(&self.state.network);
        let signature_period = compute_sync_committee_period(update.signature_slot);

        // Verify signature period does not skip a sync committee period
        assert!(
            signature_period == finalized_period || signature_period == finalized_period + 1,
            "{}",
            format!(
                "The acceptable signature periods are '{}' and '{}' but got {}",
                finalized_period,
                finalized_period + 1,
                signature_period
            )
        );

        // Verify sync committee aggregate signature
        let sync_committee = if signature_period == finalized_period {
            self.state.current_sync_committee.get().unwrap()
        } else {
            self.state.next_sync_committee.get().unwrap()
        };

        let participant_pubkeys =
            get_participant_pubkeys(&sync_committee.pubkeys.0, &sync_committee_bits);
        let fork_version = config
            .compute_fork_version_by_slot(update.signature_slot)
            .unwrap_or_else(|| panic!("{}", "Unsupported fork"));
        let domain = compute_domain(
            DOMAIN_SYNC_COMMITTEE,
            fork_version,
            config.genesis_validators_root.into(),
        );
        let signing_root = compute_signing_root(
            types::H256(update.attested_beacon_header.tree_hash_root()),
            domain,
        );

        let aggregate_signature =
            bls::AggregateSignature::deserialize(&update.sync_aggregate.sync_committee_signature.0)
                .unwrap();
        let pubkeys: Vec<bls::PublicKey> = participant_pubkeys
            .into_iter()
            .map(|x| bls::PublicKey::deserialize(&x.0).unwrap())
            .collect();
        assert!(
            aggregate_signature
                .fast_aggregate_verify(signing_root.0, &pubkeys.iter().collect::<Vec<_>>()),
            "Failed to verify the bls signature"
        );
    }

    fn update_finalized_header(&mut self, finalized_header: ExtendedBeaconBlockHeader) {
        #[cfg(feature = "logs")]
        env::log_str(format!("Update finalized header. Used gas: {}", env::used_gas().0).as_str());
        let finalized_execution_header_info = self
            .state
            .unfinalized_headers
            .get(&finalized_header.execution_block_hash.to_string())
            .unwrap_or_else(|| panic!("{}", "Unknown execution block hash"))
            .clone();
        #[cfg(feature = "logs")]
        env::log_str(
            format!(
                "Current finalized slot: {}, New finalized slot: {}",
                self.state.finalized_beacon_header.header.slot, finalized_header.header.slot
            )
            .as_str(),
        );

        let mut cursor_header = finalized_execution_header_info.clone();
        let mut cursor_header_hash = finalized_header.execution_block_hash;

        let mut submitters_update: HashMap<Addr, u32> = HashMap::new();
        loop {
            let num_of_removed_headers = *submitters_update
                .get(&cursor_header.submitter)
                .unwrap_or(&0);
            submitters_update.insert(cursor_header.submitter, num_of_removed_headers + 1);

            self.state
                .unfinalized_headers
                .remove(&cursor_header_hash.to_string());
            self.state
                .finalized_execution_blocks
                .insert(cursor_header.block_number, cursor_header_hash);

            if cursor_header.parent_hash == self.state.finalized_beacon_header.execution_block_hash
            {
                break;
            }

            cursor_header_hash = cursor_header.parent_hash;
            cursor_header = self
                .state
                .unfinalized_headers
                .get(&cursor_header.parent_hash.to_string())
                .unwrap_or_else(|| {
                    panic!(
                        "{}",
                        format!(
                            "Header has unknown parent {:?}. Parent should be submitted first.",
                            cursor_header.parent_hash
                        )
                        .as_str(),
                    )
                })
                .clone();
        }
        self.state.finalized_beacon_header = finalized_header;
        self.state.finalized_execution_header = Some(finalized_execution_header_info.clone());

        for (submitter, num_of_removed_headers) in &submitters_update {
            self.update_submitter(submitter, -(*num_of_removed_headers as i64));
        }

        #[cfg(feature = "logs")]
        env::log_str(
            format!(
                "Finish update finalized header. Used gas: {}",
                env::used_gas().0
            )
            .as_str(),
        );

        if finalized_execution_header_info.block_number > self.state.hashes_gc_threshold {
            self.gc_finalized_execution_blocks(
                finalized_execution_header_info.block_number - self.state.hashes_gc_threshold,
            );
        }
    }

    fn commit_light_client_update(&mut self, update: LightClientUpdate) {
        // Update finalized header
        let finalized_header_update = update.finality_update.header_update;
        let finalized_period =
            compute_sync_committee_period(self.state.finalized_beacon_header.header.slot);
        let update_period =
            compute_sync_committee_period(finalized_header_update.beacon_header.slot);

        if update_period == finalized_period + 1 {
            self.state.current_sync_committee =
                Some(self.state.next_sync_committee.clone().unwrap());
            self.state.next_sync_committee =
                Some(update.sync_committee_update.unwrap().next_sync_committee);
        }

        self.update_finalized_header(finalized_header_update.into());
    }

    /// Remove information about the headers that are at least as old as the given block number.
    fn gc_finalized_execution_blocks(&mut self, mut header_number: u64) {
        loop {
            if self
                .state
                .finalized_execution_blocks
                .remove(&header_number)
                .is_some()
            {
                if header_number == 0 {
                    break;
                } else {
                    header_number -= 1;
                }
            } else {
                break;
            }
        }
    }

    fn update_submitter(&mut self, submitter: &Addr, value: i64) {
        let mut num_of_submitted_headers: i64 =
            *self.state.submitters.get(submitter).unwrap_or_else(|| {
                panic!(
                    "{}",
                    "The account can't submit blocks because it is not registered"
                )
            }) as i64;

        num_of_submitted_headers += value;

        assert!(
            num_of_submitted_headers <= self.state.max_submitted_blocks_by_account.into(),
            "The submitter exhausted the limit of blocks"
        );

        self.state.submitters.insert(
            submitter.clone(),
            num_of_submitted_headers.try_into().unwrap(),
        );
    }

    fn is_light_client_update_allowed(&self) {
        if let Some(trusted_signer) = &self.state.trusted_signer {
            assert!(
                self.ctx.info.clone().unwrap().sender == *trusted_signer,
                "Eth-client is deployed as trust mode, only trusted_signer can update the client"
            );
        }
    }
}
