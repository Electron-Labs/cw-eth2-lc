use std::collections::HashMap;
use std::str::FromStr;

use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use borsh::{BorshDeserialize, BorshSerialize};
use cosmwasm_std::{Env, MessageInfo};
use eth2_utility::consensus::*;
use eth2_utility::types::*;
use eth_types::eth2::*;
use eth_types::{BlockHeader, H256};

use near_sdk::{assert_self, require, AccountId, PanicOnDefault};

use tree_hash::TreeHash;

pub struct Eth2Client {
    pub ctx: Context,
    pub state: Eth2ClientState,
}

pub struct Context {
    pub env: Env,
    pub info: Option<MessageInfo>,
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Eth2ClientState {
    /// If set, only light client updates by the trusted signer will be accepted
    trusted_signer: Option<AccountId>,
    /// Whether the client validates the updates.
    /// Should only be set to `false` for debugging, testing, and diagnostic purposes
    validate_updates: bool,
    /// Whether the client verifies BLS signatures.
    verify_bls_signatures: bool,
    /// We store the hashes of the blocks for the past `hashes_gc_threshold` headers.
    /// Events that happen past this threshold cannot be verified by the client.
    /// It is desirable that this number is larger than 7 days' worth of headers, which is roughly
    /// 51k Ethereum blocks. So this number should be 51k in production.
    hashes_gc_threshold: u64,
    /// Network. e.g. mainnet, kiln
    network: Network,
    /// Hashes of the finalized execution blocks mapped to their numbers. Stores up to `hashes_gc_threshold` entries.
    /// Execution block number -> execution block hash
    finalized_execution_blocks: HashMap<u64, H256>,
    /// All unfinalized execution blocks' headers hashes mapped to their `HeaderInfo`.
    /// Execution block hash -> ExecutionHeaderInfo object
    unfinalized_headers: HashMap<Vec<u8>, ExecutionHeaderInfo>,
    /// `AccountId`s mapped to their number of submitted headers.
    /// Submitter account -> Num of submitted headers
    submitters: HashMap<AccountId, u32>,
    /// Max number of unfinalized blocks allowed to be stored by one submitter account
    /// This value should be at least 32 blocks (1 epoch), but the recommended value is 1024 (32 epochs)
    max_submitted_blocks_by_account: u32,
    /// Light client state
    finalized_beacon_header: ExtendedBeaconBlockHeader,
    finalized_execution_header: Option<ExecutionHeaderInfo>,
    current_sync_committee: Option<SyncCommittee>,
    next_sync_committee: Option<SyncCommittee>,
}

impl Eth2Client {
    pub fn init(ctx: Context, args: InitInput) -> Self {
        let network =
            Network::from_str(args.network.as_str()).unwrap_or_else(|e| panic!("{}", e.as_str()));

        #[cfg(feature = "mainnet")]
        {
            require!(
                args.validate_updates,
                "The updates validation can't be disabled for mainnet"
            );

            require!(
                (cfg!(feature = "bls") && args.verify_bls_signatures)
                    || args.trusted_signer.is_some(),
                "The client can't be executed in the trustless mode without BLS sigs verification on Mainnet"
            );
        }

        require!(
            args.finalized_execution_header.calculate_hash()
                == args.finalized_beacon_header.execution_block_hash,
            "Invalid execution block"
        );

        let finalized_execution_header_info = ExecutionHeaderInfo {
            parent_hash: args.finalized_execution_header.parent_hash,
            block_number: args.finalized_execution_header.number,
            submitter: AccountId::new_unchecked(ctx.info.clone().unwrap().sender.to_string()),
        };

        Self {
            ctx,
            state: Eth2ClientState {
                trusted_signer: args.trusted_signer,
                validate_updates: args.validate_updates,
                verify_bls_signatures: args.verify_bls_signatures,
                hashes_gc_threshold: args.hashes_gc_threshold,
                network,
                finalized_execution_blocks: HashMap::new(),
                unfinalized_headers: HashMap::new(),
                submitters: HashMap::new(),
                max_submitted_blocks_by_account: args.max_submitted_blocks_by_account,
                finalized_beacon_header: args.finalized_beacon_header,
                finalized_execution_header: Some(finalized_execution_header_info),
                current_sync_committee: Some(args.current_sync_committee),
                next_sync_committee: Some(args.next_sync_committee),
            },
        }
    }

    /// Returns finalized execution block number
    pub fn last_block_number(&self) -> u64 {
        self.state
            .finalized_execution_header
            .clone()
            .unwrap()
            .block_number
    }

    /// Returns finalized execution block hash
    pub fn block_hash_safe(&self, block_number: u64) -> Option<H256> {
        self.state
            .finalized_execution_blocks
            .get(&block_number)
            .copied()
    }

    /// Checks if the execution header is already submitted.
    pub fn is_known_execution_header(&self, hash: H256) -> bool {
        self.state
            .unfinalized_headers
            .get(&hash.try_to_vec().unwrap())
            .is_some()
    }

    /// Get finalized beacon block root
    pub fn finalized_beacon_block_root(&self) -> H256 {
        self.state.finalized_beacon_header.beacon_block_root
    }

    /// Returns finalized beacon block slot
    pub fn finalized_beacon_block_slot(&self) -> u64 {
        self.state.finalized_beacon_header.header.slot
    }

    /// Returns finalized beacon block header
    pub fn finalized_beacon_block_header(&self) -> ExtendedBeaconBlockHeader {
        self.state.finalized_beacon_header.clone()
    }

    /// Get the current light client state
    pub fn get_light_client_state(&self) -> LightClientState {
        LightClientState {
            finalized_beacon_header: self.state.finalized_beacon_header.clone(),
            current_sync_committee: self.state.current_sync_committee.clone().unwrap(),
            next_sync_committee: self.state.next_sync_committee.clone().unwrap(),
        }
    }

    pub fn register_submitter(&mut self) {
        let account_id = AccountId::new_unchecked(self.ctx.info.clone().unwrap().sender.to_string());
        require!(
            !self.state.submitters.contains_key(&account_id),
            "The account is already registered"
        );

        self.state.submitters.insert(account_id, 0);
    }

    pub fn unregister_submitter(&mut self) {
        let account_id = AccountId::new_unchecked(self.ctx.info.clone().unwrap().sender.to_string());
        if self.state.submitters.remove(&account_id).is_none() {
            panic!("{}", "The account is not registered");
        }
    }

    pub fn is_submitter_registered(&self, account_id: AccountId) -> bool {
        self.state.submitters.contains_key(&account_id)
    }

    pub fn get_num_of_submitted_blocks_by_account(&self, account_id: AccountId) -> u32 {
        *self
            .state
            .submitters
            .get(&account_id)
            .unwrap_or_else(|| panic!("{}", "The account is not registered"))
    }

    pub fn get_max_submitted_blocks_by_account(&self) -> u32 {
        self.state.max_submitted_blocks_by_account
    }

    pub fn submit_beacon_chain_light_client_update(&mut self, update: LightClientUpdate) {
        self.is_light_client_update_allowed();

        if self.state.validate_updates {
            self.validate_light_client_update(&update);
        }

        self.commit_light_client_update(update);
    }

    pub fn submit_execution_header(&mut self, block_header: BlockHeader) {
        if self.state.finalized_beacon_header.execution_block_hash != block_header.parent_hash {
            self.state
                .unfinalized_headers
                .get(&block_header.parent_hash.try_to_vec().unwrap())
                .unwrap_or_else(|| {
                    panic!(
                        "{}",
                        format!(
                            "Header has unknown parent {:?}. Parent should be submitted first.",
                            block_header.parent_hash
                        )
                        .as_str(),
                    )
                });
        }

        let submitter = AccountId::new_unchecked(self.ctx.info.clone().unwrap().sender.to_string());
        self.update_submitter(&submitter, 1);
        let block_hash = block_header.calculate_hash();
        #[cfg(feature = "logs")]
        env::log_str(
            format!(
                "Submitted header number {}, hash {:?}",
                block_header.number, block_hash
            )
            .as_str(),
        );

        let block_info = ExecutionHeaderInfo {
            parent_hash: block_header.parent_hash,
            block_number: block_header.number,
            submitter,
        };
        let insert_result = self
            .state
            .unfinalized_headers
            .insert(block_hash.try_to_vec().unwrap(), block_info);
        require!(
            insert_result.is_none(),
            format!("The block {} already submitted!", &block_hash)
        );
    }

    pub fn update_trusted_signer(&mut self, trusted_signer: Option<AccountId>) {
        assert_self();
        self.state.trusted_signer = trusted_signer;
    }

    pub fn get_trusted_signer(&self) -> Option<AccountId> {
        self.state.trusted_signer.clone()
    }
}

impl Eth2Client {
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

        require!(
            sync_committee_bits_sum >= MIN_SYNC_COMMITTEE_PARTICIPANTS,
            format!("Invalid sync committee bits sum: {sync_committee_bits_sum}")
        );

        require!(
            sync_committee_bits_sum * 3 >= (sync_committee_bits.len() * 2).try_into().unwrap(),
            format!(
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

        require!(
            active_header.slot > self.state.finalized_beacon_header.header.slot,
            "The active header slot number should be higher than the finalized slot"
        );

        require!(
            update.attested_beacon_header.slot
                >= update.finality_update.header_update.beacon_header.slot,
            "The attested header slot should be equal to or higher than the finalized header slot"
        );

        require!(
            update.signature_slot > update.attested_beacon_header.slot,
            "The signature slot should be higher than the attested header slot"
        );

        let update_period = compute_sync_committee_period(active_header.slot);
        require!(
            update_period == finalized_period || update_period == finalized_period + 1,
            format!(
                "The acceptable update periods are '{}' and '{}' but got {}",
                finalized_period,
                finalized_period + 1,
                update_period
            )
        );

        // Verify that the `finality_branch`, confirms `finalized_header`
        // to match the finalized checkpoint root saved in the state of `attested_header`.
        let branch = convert_branch(&update.finality_update.finality_branch);
        require!(
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
        require!(
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
            require!(
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
        require!(
            signature_period == finalized_period || signature_period == finalized_period + 1,
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
            eth_types::H256(update.attested_beacon_header.tree_hash_root()),
            domain,
        );

        let aggregate_signature =
            bls::AggregateSignature::deserialize(&update.sync_aggregate.sync_committee_signature.0)
                .unwrap();
        let pubkeys: Vec<bls::PublicKey> = participant_pubkeys
            .into_iter()
            .map(|x| bls::PublicKey::deserialize(&x.0).unwrap())
            .collect();
        require!(
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
            .get(&finalized_header.execution_block_hash.try_to_vec().unwrap())
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

        let mut submitters_update: HashMap<AccountId, u32> = HashMap::new();
        loop {
            let num_of_removed_headers = *submitters_update
                .get(&cursor_header.submitter)
                .unwrap_or(&0);
            submitters_update.insert(cursor_header.submitter, num_of_removed_headers + 1);

            self.state
                .unfinalized_headers
                .remove(&cursor_header_hash.try_to_vec().unwrap());
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
                .get(&cursor_header.parent_hash.try_to_vec().unwrap())
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

    fn update_submitter(&mut self, submitter: &AccountId, value: i64) {
        let mut num_of_submitted_headers: i64 =
            *self.state.submitters.get(submitter).unwrap_or_else(|| {
                panic!(
                    "{}",
                    "The account can't submit blocks because it is not registered"
                )
            }) as i64;

        num_of_submitted_headers += value;

        require!(
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
            require!(
                &AccountId::new_unchecked(self.ctx.info.clone().unwrap().sender.to_string())
                    == trusted_signer,
                "Eth-client is deployed as trust mode, only trusted_signer can update the client"
            );
        }
    }
}
