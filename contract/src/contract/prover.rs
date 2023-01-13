use super::Contract;
use crate::msg::VerifyLogEntryRequest;
use cosmwasm_std::Deps;
use rlp::Rlp;
use types::{near_keccak256, BlockHeader, LogEntry, Receipt, H256};

impl Contract<'_> {
    pub fn verify_log_entry(&self, deps: Deps, req: VerifyLogEntryRequest) -> bool {
        let log_entry: LogEntry = rlp::decode(req.log_entry_data.as_slice()).unwrap();
        let receipt: Receipt = rlp::decode(req.receipt_data.as_slice()).unwrap();
        let header: BlockHeader = rlp::decode(req.header_data.as_slice()).unwrap();

        // Verify log_entry included in receipt
        assert_eq!(receipt.logs[req.log_index as usize], log_entry);

        // Verify receipt included into header
        let data = verify_trie_proof(
            header.receipts_root,
            rlp::encode(&req.receipt_index),
            req.proof,
        );
        let verification_result = req.receipt_data == data;

        if !verification_result {
            return false;
        }

        if self.block_hash_safe(deps, header.number) != header.hash {
            return false;
        }

        true
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
