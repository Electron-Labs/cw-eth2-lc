use super::Contract;
use crate::eth_utility::NetworkConfig;
use crate::msg::{VerifyLogEntryRequest, VerifyLogEntryResponse};
use cosmwasm_std::Deps;
use rlp::Rlp;
use sha2::{Digest, Sha256};
use types::{near_keccak256, LogEntry, Receipt, H256};

const SLOTS_PER_HISTORICAL_ROOT: u64 = 8192;
const HISTORICAL_ROOTS_LIMIT: u64 = 16777216;
const STATE_TO_HISTORICAL_G_INDEX: u64 = 27;

impl Contract<'_> {
    pub fn verify_log_entry(
        &self,
        deps: Deps,
        req: VerifyLogEntryRequest,
    ) -> VerifyLogEntryResponse {
        let log_entry: LogEntry = rlp::decode(req.log_entry_data.as_slice()).unwrap();
        let receipt: Receipt = rlp::decode(req.receipt_data.as_slice()).unwrap();

        // Verify log_entry included in receipt
        assert_eq!(receipt.logs[req.log_index as usize], log_entry);
        let receipts_root = req.receipts_root;

        // Verify receipt included into header
        let data = verify_trie_proof(
            receipts_root.clone().into(),
            rlp::encode(&req.receipt_index).to_vec(),
            req.proof,
        );
        let verification_result = req.receipt_data == data;

        if !verification_result {
            return VerifyLogEntryResponse { verified: false };
        } else if verification_result && req.skip_bridge_call {
            return VerifyLogEntryResponse { verified: true };
        }

        let non_mapped_data = self.state.non_mapped.load(deps.storage).unwrap();
        let network = non_mapped_data.network;
        let capella_fork_slot = {
            let network_config = NetworkConfig::new(&network);
            network_config.capella_fork_epoch * 32
        };

        let historical_list_index = (req.tx_slot - capella_fork_slot) / SLOTS_PER_HISTORICAL_ROOT;

        let mut index: u128 = 0u128;
        if req.src_slot == req.tx_slot {
            index = 8 + 3;
            index = index * 2u128.pow(9) + 387;
        } else if req.src_slot - req.tx_slot <= SLOTS_PER_HISTORICAL_ROOT {
            index = 8 + 3;
            index = index * 2u128.pow(5) + 6;
            index = index * u128::from(SLOTS_PER_HISTORICAL_ROOT)
                + u128::from(req.tx_slot % SLOTS_PER_HISTORICAL_ROOT);
            index = index * 2u128.pow(9) + 387;
        } else if req.tx_slot < req.src_slot {
            index = 8 + 3;
            index = index * 2u128.pow(5) + u128::from(STATE_TO_HISTORICAL_G_INDEX);
            index *= 2;
            index = index * u128::from(HISTORICAL_ROOTS_LIMIT) + u128::from(historical_list_index);
            index = index * 2 + 1;
            index = index * u128::from(SLOTS_PER_HISTORICAL_ROOT)
                + u128::from(req.tx_slot % SLOTS_PER_HISTORICAL_ROOT);
            index = index * 2u128.pow(9) + 387;
        }

        let expected_header_root =
            restore_merkle_root(receipts_root, index, req.receipts_root_proof);

        if self.header_root(deps, req.src_slot).header_root != Some(expected_header_root) {
            return VerifyLogEntryResponse { verified: false };
        }

        VerifyLogEntryResponse { verified: true }
    }
}

/// Verify the proof recursively traversing through the key.
/// Return the value at the end of the key, in case the proof is valid.
///
/// @param expected_root is the expected root of the current node.
/// @param key is the key for which we are proving the value.
/// @param proof contains relevant information to verify data is valid
///
/// Patricia Trie: https://eth.wiki/en/fundamentals/patricia-tree
/// Patricia Img:  https://ethereum.stackexchange.com/questions/268/ethereum-block-architecture/6413#6413
///
/// Verification:  https://github.com/slockit/in3/wiki/Ethereum-Verification-and-MerkleProof#receipt-proof
/// Article:       https://medium.com/@ouvrard.pierre.alain/merkle-proof-verification-for-ethereum-patricia-tree-48f29658eec
/// Python impl:   https://gist.github.com/mfornet/0ff283274c0162f1cca45966bccf69ee
///
fn verify_trie_proof(expected_root: H256, key: Vec<u8>, proof: Vec<Vec<u8>>) -> Vec<u8> {
    let mut actual_key = vec![];
    for el in key {
        actual_key.push(el / 16);
        actual_key.push(el % 16);
    }
    _verify_trie_proof((expected_root.0).0.into(), &actual_key, &proof, 0, 0)
}

fn _verify_trie_proof(
    expected_root: Vec<u8>,
    key: &Vec<u8>,
    proof: &Vec<Vec<u8>>,
    key_index: usize,
    proof_index: usize,
) -> Vec<u8> {
    let node = &proof[proof_index];

    if key_index == 0 {
        // trie root is always a hash
        assert_eq!(near_keccak256(node), expected_root.as_slice());
    } else if node.len() < 32 {
        // if rlp < 32 bytes, then it is not hashed
        assert_eq!(node.as_slice(), expected_root);
    } else {
        assert_eq!(near_keccak256(node), expected_root.as_slice());
    }

    let node = Rlp::new(node.as_slice());

    if node.iter().count() == 17 {
        // Branch node
        if key_index == key.len() {
            assert_eq!(proof_index + 1, proof.len());
            get_vec(&node, 16)
        } else {
            let new_expected_root = get_vec(&node, key[key_index] as usize);
            _verify_trie_proof(
                new_expected_root,
                key,
                proof,
                key_index + 1,
                proof_index + 1,
            )
        }
    } else {
        // Leaf or extension node
        assert_eq!(node.iter().count(), 2);
        let path_u8 = get_vec(&node, 0);
        // Extract first nibble
        let head = path_u8[0] / 16;
        // assert!(0 <= head); is implicit because of type limits
        assert!(head <= 3);

        // Extract path
        let mut path = vec![];
        if head % 2 == 1 {
            path.push(path_u8[0] % 16);
        }
        for val in path_u8.into_iter().skip(1) {
            path.push(val / 16);
            path.push(val % 16);
        }
        assert_eq!(path.as_slice(), &key[key_index..key_index + path.len()]);

        if head >= 2 {
            // Leaf node
            assert_eq!(proof_index + 1, proof.len());
            assert_eq!(key_index + path.len(), key.len());
            get_vec(&node, 1)
        } else {
            // Extension node
            let new_expected_root = get_vec(&node, 1);
            _verify_trie_proof(
                new_expected_root,
                key,
                proof,
                key_index + path.len(),
                proof_index + 1,
            )
        }
    }
}

/// Get element at position `pos` from rlp encoded data,
/// and decode it as vector of bytes
fn get_vec(data: &Rlp, pos: usize) -> Vec<u8> {
    data.at(pos).unwrap().as_val::<Vec<u8>>().unwrap()
}

fn restore_merkle_root(leaf: Vec<u8>, mut index: u128, branch: Vec<Vec<u8>>) -> Vec<u8> {
    let mut value = leaf;
    for b in branch.iter() {
        let mut hasher = Sha256::new();

        if index & 1 == 0 {
            hasher.update(&[&value, b.as_slice()].concat());
            value = hasher.finalize().to_vec();
        } else {
            hasher.update(&[b.as_slice(), &value].concat());
            value = hasher.finalize().to_vec();
        }
        index >>= 1;
    }
    value
}
