use cosmwasm_schema::cw_serde;
use schemars::{schema::Schema, JsonSchema};
use tree_hash::{MerkleHasher, TreeHash};

use crate::{arr_wrapper_impl_tree_hash_and_borsh, vec_wrapper_impl_tree_hash, H256};

use hex::FromHex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub const PUBLIC_KEY_BYTES_LEN: usize = 48;
pub const SIGNATURE_BYTES_LEN: usize = 96;
pub const SYNC_COMMITTEE_BITS_SIZE_IN_BYTES: usize = 512 / 8;

pub type Slot = u64;
pub type Epoch = u64;
pub type ForkVersion = [u8; 4];
pub type DomainType = [u8; 4];

#[derive(Debug, Clone, PartialEq)]
pub struct PublicKeyBytes(pub [u8; PUBLIC_KEY_BYTES_LEN]);
#[derive(Debug, Clone, PartialEq)]
pub struct SignatureBytes(pub [u8; SIGNATURE_BYTES_LEN]);
#[derive(Debug, Clone, PartialEq)]
pub struct SyncCommitteeBits(pub [u8; SYNC_COMMITTEE_BITS_SIZE_IN_BYTES]);

arr_wrapper_impl_tree_hash_and_borsh!(PublicKeyBytes, PUBLIC_KEY_BYTES_LEN);
arr_wrapper_impl_tree_hash_and_borsh!(SignatureBytes, SIGNATURE_BYTES_LEN);
arr_wrapper_impl_tree_hash_and_borsh!(SyncCommitteeBits, SYNC_COMMITTEE_BITS_SIZE_IN_BYTES);

#[cw_serde]
#[derive(Default, tree_hash_derive::TreeHash)]
pub struct BeaconBlockHeader {
    #[schemars(schema_with = "crate::string_schema")]
    #[serde(with = "eth2_serde_utils::quoted_u64")]
    pub slot: Slot,
    #[schemars(schema_with = "crate::string_schema")]
    #[serde(with = "eth2_serde_utils::quoted_u64")]
    pub proposer_index: u64,
    pub parent_root: H256,
    pub state_root: H256,
    pub body_root: H256,
}
#[derive(Debug, Clone, PartialEq, tree_hash_derive::TreeHash)]
pub struct ForkData {
    pub current_version: ForkVersion,
    pub genesis_validators_root: H256,
}

#[derive(Debug, PartialEq, Clone, tree_hash_derive::TreeHash)]
pub struct SigningData {
    pub object_root: H256,
    pub domain: H256,
}

#[cw_serde]
#[derive(Default)]
pub struct ExtendedBeaconBlockHeader {
    pub header: BeaconBlockHeader,
    pub beacon_block_root: H256,
    pub execution_block_hash: H256,
}

impl From<HeaderUpdate> for ExtendedBeaconBlockHeader {
    fn from(item: HeaderUpdate) -> Self {
        let root = H256(item.beacon_header.tree_hash_root());
        ExtendedBeaconBlockHeader {
            header: item.beacon_header,
            beacon_block_root: root,
            execution_block_hash: item.execution_block_hash,
        }
    }
}

#[cw_serde]
pub struct SyncCommitteePublicKeys(pub Vec<PublicKeyBytes>);
vec_wrapper_impl_tree_hash!(SyncCommitteePublicKeys);

#[cw_serde]
#[derive(tree_hash_derive::TreeHash)]
pub struct SyncCommittee {
    pub pubkeys: SyncCommitteePublicKeys,
    pub aggregate_pubkey: PublicKeyBytes,
}

#[cw_serde]
pub struct SyncAggregate {
    pub sync_committee_bits: SyncCommitteeBits,
    pub sync_committee_signature: SignatureBytes,
}

#[cw_serde]
pub struct SyncCommitteeUpdate {
    pub next_sync_committee: SyncCommittee,
    pub next_sync_committee_branch: Vec<H256>,
}

#[cw_serde]
pub struct HeaderUpdate {
    pub beacon_header: BeaconBlockHeader,
    pub execution_block_hash: H256,
    pub execution_hash_branch: Vec<H256>,
}

#[cw_serde]
pub struct FinalizedHeaderUpdate {
    pub header_update: HeaderUpdate,
    pub finality_branch: Vec<H256>,
}

#[cw_serde]
pub struct LightClientUpdate {
    pub attested_beacon_header: BeaconBlockHeader,
    pub sync_aggregate: SyncAggregate,
    #[schemars(schema_with = "crate::string_schema")]
    #[serde(with = "eth2_serde_utils::quoted_u64")]
    pub signature_slot: Slot,
    pub finality_update: FinalizedHeaderUpdate,
    pub sync_committee_update: Option<SyncCommitteeUpdate>,
}

#[cw_serde]
pub struct LightClientState {
    pub finalized_beacon_header: ExtendedBeaconBlockHeader,
    pub current_sync_committee: SyncCommittee,
    pub next_sync_committee: SyncCommittee,
}
