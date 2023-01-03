use bitvec::{bitarr, order::Lsb0};
use cosmwasm_std::{Addr, Coin};
use cw_eth2_lc::{
    contract::{Contract, ContractContext},
    state::ContractState,
};
use hex::FromHex;
use tree_hash::TreeHash;
use types::{eth2::LightClientUpdate, BlockHeader, H256, U256};
use utility::consensus::*;

pub mod common;

use common::{accounts, get_test_data, InitOptions};

pub struct TestContext<'a> {
    contract: Contract,
    headers: &'a Vec<BlockHeader>,
    updates: &'a Vec<LightClientUpdate>,
}

pub fn get_test_context(
    sender: Addr,
    funds: Vec<Coin>,
    init_options: Option<InitOptions>,
) -> TestContext<'static> {
    let (headers, updates, init_input) = get_test_data(init_options);
    let mut contract = Contract::new(
        ContractContext {
            env: cosmwasm_std::testing::mock_env(),
            info: Some(cosmwasm_std::testing::mock_info(
                sender.as_str(),
                funds.as_slice(),
            )),
        },
        ContractState::default(),
    );
    contract.init(init_input);
    assert_eq!(contract.last_block_number(), headers[0].number);

    TestContext {
        contract,
        headers,
        updates,
    }
}

pub fn submit_and_check_execution_headers(contract: &mut Contract, headers: Vec<&BlockHeader>) {
    for header in headers {
        contract.submit_execution_header(header.clone());
        assert!(contract.is_known_execution_header(header.calculate_hash()));
        assert!(contract.block_hash_safe(header.number).is_none());
    }
}

mod generic_tests {
    use crate::common::read_beacon_header;

    use super::*;

    #[test]
    pub fn test_header_root() {
        let header = read_beacon_header(format!("./tests/data/kiln/beacon_header_{}.json", 5000));
        assert_eq!(
            H256(header.tree_hash_root()),
            Vec::from_hex("c613fbf1a8e95c2aa0f76a5d226ee1dc057cce18b235803f50e7a1bde050d290")
                .unwrap()
                .into()
        );

        let header = read_beacon_header(format!(
            "./tests/data/mainnet/beacon_header_{}.json",
            4100000
        ));
        assert_eq!(
            H256(header.tree_hash_root()),
            Vec::from_hex("342ca1455e976f300cc96a209106bed2cbdf87243167fab61edc6e2250a0be6c")
                .unwrap()
                .into()
        );
    }

    #[test]
    pub fn test_submit_update_two_periods() {
        let TestContext {
            mut contract,
            headers,
            updates,
        } = get_test_context(accounts(0), Vec::new(), None);

        contract.register_submitter();
        // After submitting the execution header, it should be present in the execution headers list
        // but absent in canonical chain blocks (not-finalized)
        submit_and_check_execution_headers(&mut contract, headers.iter().skip(1).collect());

        contract.submit_beacon_chain_light_client_update(updates[1].clone());

        // After Beacon Chain `LightClientUpdate` is submitted,
        // all execution headers having a height lower than the update's height,
        // should be removed from the execution headers list. Meantime, all these
        // removed execution headers should become a part of the canonical chain blocks (finalized)
        for header in headers.iter().skip(1) {
            let header_hash = header.calculate_hash();
            assert!(!contract.is_known_execution_header(header_hash));
            assert!(
                contract.block_hash_safe(header.number).unwrap_or_default() == header_hash,
                "Execution block hash is not finalized: {header_hash:?}"
            );
        }

        assert_eq!(contract.last_block_number(), headers.last().unwrap().number);
        assert!(!contract.is_known_execution_header(
            contract
                .finalized_beacon_block_header()
                .execution_block_hash
        ));

        contract.unregister_submitter();
    }

    #[test]
    pub fn test_submit_execution_block_from_fork_chain() {
        let TestContext {
            mut contract,
            headers,
            updates,
        } = get_test_context(accounts(0), Vec::new(), None);

        contract.register_submitter();
        submit_and_check_execution_headers(&mut contract, headers.iter().skip(1).collect());

        // Submit execution header with different hash
        let mut fork_header = headers[5].clone();
        // Difficulty is modified just in order to get a different header hash. Any other field would be suitable too
        fork_header.difficulty = U256::from(ethereum_types::U256::from(99));
        contract.submit_execution_header(fork_header.clone());
        contract.submit_beacon_chain_light_client_update(updates[1].clone());

        for header in headers.iter().skip(1) {
            let header_hash = header.calculate_hash();
            assert!(!contract.is_known_execution_header(header_hash));
            assert!(
                contract.block_hash_safe(header.number).unwrap_or_default() == header_hash,
                "Execution block hash is not finalized: {header_hash:?}"
            );
        }

        // Check that forked execution header was not finalized
        assert!(contract.is_known_execution_header(fork_header.calculate_hash()));
        assert!(
        contract
            .block_hash_safe(fork_header.number)
            .unwrap_or_default()
            != fork_header.calculate_hash(),
        "The fork's execution block header {:?} is expected not to be finalized, but it is finalized",
        fork_header.calculate_hash()
    );

        assert_eq!(contract.last_block_number(), headers.last().unwrap().number);
    }

    #[test]
    pub fn test_gc_headers() {
        let TestContext {
            mut contract,
            headers,
            updates,
        } = get_test_context(
            accounts(0),
            Vec::new(),
            Some(InitOptions {
                validate_updates: true,
                verify_bls_signatures: true,
                hashes_gc_threshold: 500,
                max_submitted_blocks_by_account: 7000,
                trusted_signer: None,
            }),
        );

        contract.register_submitter();
        submit_and_check_execution_headers(&mut contract, headers.iter().skip(1).collect());

        contract.submit_beacon_chain_light_client_update(updates[1].clone());

        // Last 500 execution headers are finalized
        for header in headers.iter().skip(1).rev().take(500) {
            assert!(!contract.is_known_execution_header(header.calculate_hash()));
            assert!(
                contract.block_hash_safe(header.number).unwrap_or_default()
                    == header.calculate_hash(),
                "Execution block hash is not finalized: {:?}",
                header.calculate_hash()
            );
        }

        assert_eq!(contract.last_block_number(), headers.last().unwrap().number);

        // Headers older than last 500 hundred headers are both removed and are not present in execution header list
        for header in headers.iter().skip(1).rev().skip(500) {
            assert!(!contract.is_known_execution_header(header.calculate_hash()));
            assert!(
                contract.block_hash_safe(header.number).is_none(),
                "Execution block hash was not removed: {:?}",
                header.calculate_hash()
            );
        }
    }

    #[test]
    #[should_panic(expected = "exhausted the limit of blocks")]
    pub fn test_panic_on_exhausted_submit_limit() {
        let TestContext {
            mut contract,
            headers,
            updates: _,
        } = get_test_context(
            accounts(0),
            Vec::new(),
            Some(InitOptions {
                validate_updates: true,
                verify_bls_signatures: true,
                hashes_gc_threshold: 7100,
                max_submitted_blocks_by_account: 100,
                trusted_signer: None,
            }),
        );
        contract.register_submitter();

        submit_and_check_execution_headers(&mut contract, headers.iter().skip(1).collect());
    }

    #[test]
    pub fn test_max_submit_blocks_by_account_limit() {
        let TestContext {
            mut contract,
            headers,
            updates: _,
        } = get_test_context(
            accounts(0),
            Vec::new(),
            Some(InitOptions {
                validate_updates: true,
                verify_bls_signatures: true,
                hashes_gc_threshold: 7100,
                max_submitted_blocks_by_account: 100,
                trusted_signer: None,
            }),
        );
        contract.register_submitter();

        submit_and_check_execution_headers(
            &mut contract,
            headers.iter().skip(1).take(100).collect(),
        );
    }

    #[test]
    #[should_panic(expected = "only trusted_signer can update the client")]
    pub fn test_trusted_signer() {
        let trusted_signer = accounts(1);
        let TestContext {
            mut contract,
            headers: _,
            updates,
        } = get_test_context(
            accounts(0),
            Vec::new(),
            Some(InitOptions {
                validate_updates: true,
                verify_bls_signatures: true,
                hashes_gc_threshold: 7100,
                max_submitted_blocks_by_account: 100,
                trusted_signer: Some(trusted_signer),
            }),
        );
        contract.submit_beacon_chain_light_client_update(updates[1].clone());
    }

    #[test]
    #[should_panic(expected = "Invalid finality proof")]
    pub fn test_panic_on_invalid_finality_proof() {
        let TestContext {
            mut contract,
            headers: _,
            updates,
        } = get_test_context(accounts(0), Vec::new(), None);
        let mut update = updates[1].clone();
        update.finality_update.finality_branch[5] = H256::from(
            hex::decode("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap(),
        );
        contract.submit_beacon_chain_light_client_update(update);
    }

    #[test]
    #[should_panic(expected = "Invalid finality proof")]
    pub fn test_panic_on_empty_finality_proof() {
        let TestContext {
            mut contract,
            headers: _,
            updates,
        } = get_test_context(accounts(0), Vec::new(), None);
        let mut update = updates[1].clone();
        update.finality_update.finality_branch = vec![];
        contract.submit_beacon_chain_light_client_update(update);
    }

    #[test]
    #[should_panic(expected = "Invalid execution block hash proof")]
    pub fn test_panic_on_invalid_execution_block_proof() {
        let TestContext {
            mut contract,
            headers: _,
            updates,
        } = get_test_context(accounts(0), Vec::new(), None);
        let mut update = updates[1].clone();
        update.finality_update.header_update.execution_hash_branch[5] = H256::from(
            hex::decode("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap(),
        );
        contract.submit_beacon_chain_light_client_update(update);
    }

    #[test]
    #[should_panic(expected = "Invalid execution block hash proof")]
    pub fn test_panic_on_empty_execution_block_proof() {
        let TestContext {
            mut contract,
            headers: _,
            updates,
        } = get_test_context(accounts(0), Vec::new(), None);
        let mut update = updates[1].clone();
        update.finality_update.header_update.execution_hash_branch = vec![];
        contract.submit_beacon_chain_light_client_update(update);
    }

    #[test]
    #[should_panic(expected = "The acceptable update periods are")]
    pub fn test_panic_on_skip_update_period() {
        let TestContext {
            mut contract,
            headers: _,
            updates,
        } = get_test_context(accounts(0), Vec::new(), None);
        let mut update = updates[1].clone();
        update.finality_update.header_update.beacon_header.slot =
            update.signature_slot + EPOCHS_PER_SYNC_COMMITTEE_PERIOD * SLOTS_PER_EPOCH * 10;
        update.attested_beacon_header.slot =
            update.finality_update.header_update.beacon_header.slot;
        update.signature_slot = update.attested_beacon_header.slot + 1;
        contract.submit_beacon_chain_light_client_update(update);
    }

    #[test]
    #[should_panic(expected = "Unknown execution block hash")]
    pub fn test_panic_on_submit_update_with_missing_execution_blocks() {
        let TestContext {
            mut contract,
            headers,
            updates,
        } = get_test_context(accounts(0), Vec::new(), None);

        contract.register_submitter();
        submit_and_check_execution_headers(&mut contract, headers.iter().skip(1).take(5).collect());

        contract.submit_beacon_chain_light_client_update(updates[1].clone());
    }

    #[test]
    #[should_panic(expected = "already submitted")]
    pub fn test_panic_on_submit_same_execution_blocks() {
        let TestContext {
            mut contract,
            headers,
            updates: _,
        } = get_test_context(accounts(0), Vec::new(), None);

        contract.register_submitter();
        contract.submit_execution_header(headers[1].clone());
        contract.submit_execution_header(headers[1].clone());
    }

    #[test]
    #[should_panic(expected = "can't submit blocks because it is not registered")]
    pub fn test_panic_on_submit_execution_block_after_submitter_unregistered() {
        let TestContext {
            mut contract,
            headers,
            updates: _,
        } = get_test_context(accounts(0), Vec::new(), None);

        contract.register_submitter();
        contract.unregister_submitter();
        contract.submit_execution_header(headers[1].clone());
    }

    // #[test]
    // #[should_panic(expected = "paused")]
    // pub fn test_panic_on_submit_update_paused() {
    //     let TestContext {
    //         mut contract,
    //         headers: _,
    //         updates,
    //     } = get_test_context(None);
    //     set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: accounts(0), current_account_id: accounts(0));
    //     contract.set_paused(PAUSE_SUBMIT_UPDATE);
    //     set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: accounts(1), current_account_id: accounts(0));
    //     contract.submit_beacon_chain_light_client_update(updates[1].clone());
    // }

    #[test]
    #[should_panic(
        expected = "The active header slot number should be higher than the finalized slot"
    )]
    pub fn test_panic_on_submit_outdated_update() {
        let TestContext {
            mut contract,
            headers: _,
            updates,
        } = get_test_context(accounts(0), Vec::new(), None);

        contract.submit_beacon_chain_light_client_update(updates[0].clone());
    }

    #[test]
    #[should_panic(expected = "Parent should be submitted first")]
    pub fn test_panic_on_submit_blocks_with_unknown_parent() {
        let TestContext {
            mut contract,
            headers,
            updates: _,
        } = get_test_context(accounts(0), Vec::new(), None);

        assert_eq!(contract.last_block_number(), headers[0].number);
        contract.register_submitter();

        contract.submit_execution_header(headers[1].clone());
        // Skip 2th block
        contract.submit_execution_header(headers[3].clone());
    }

    // #[test]
    // #[should_panic(expected = "Can't unregister the account with used storage")]
    // pub fn test_panic_on_unregister_submitter() {
    //     let TestContext {
    //         mut contract,
    //         headers,
    //         updates: _,
    //     } = get_test_context(accounts(0), Vec::new(), None);

    //     assert_eq!(contract.last_block_number(), headers[0].number);

    //     contract.register_submitter();

    //     submit_and_check_execution_headers(&mut contract, headers.iter().skip(1).take(5).collect());

    //     contract.unregister_submitter();
    // }

    #[test]
    #[should_panic(expected = "can't submit blocks because it is not registered")]
    pub fn test_panic_on_skipping_register_submitter() {
        let TestContext {
            mut contract,
            headers,
            updates: _,
        } = get_test_context(accounts(0), Vec::new(), None);

        assert_eq!(contract.last_block_number(), headers[0].number);

        contract.submit_execution_header(headers[1].clone());
    }

    #[test]
    #[should_panic(expected = "Sync committee bits sum is less than 2/3 threshold, bits sum: 341")]
    pub fn test_panic_on_sync_committee_bits_is_less_than_threshold() {
        let TestContext {
            mut contract,
            headers: _,
            updates,
        } = get_test_context(accounts(0), Vec::new(), None);
        let mut update = updates[1].clone();

        let mut sync_committee_bits = bitarr![u8, Lsb0; 0; 512];

        // The number of participants should satisfy the inequality:
        // num_of_participants * 3 >= sync_committee_bits_size * 2
        // If the sync_committee_bits_size = 512, then
        // the minimum allowed value of num_of_participants is 342.

        // Fill the sync_committee_bits with 341 participants to trigger panic
        let num_of_participants = (((512.0 * 2.0 / 3.0) as f32).ceil() - 1.0) as usize;
        sync_committee_bits
            .get_mut(0..num_of_participants)
            .unwrap()
            .fill(true);
        update.sync_aggregate.sync_committee_bits =
            sync_committee_bits.as_raw_mut_slice().to_vec().into();
        contract.submit_beacon_chain_light_client_update(update);
    }

    #[test]
    #[should_panic(expected = "The sync committee update is missed")]
    pub fn test_panic_on_missing_sync_committee_update() {
        let TestContext {
            mut contract,
            headers: _,
            updates,
        } = get_test_context(accounts(0), Vec::new(), None);
        let mut update = updates[1].clone();
        update.sync_committee_update = None;
        contract.submit_beacon_chain_light_client_update(update);
    }
}

#[cfg(feature = "mainnet")]
mod mainnet_tests {
    use super::*;

    #[test]
    #[should_panic(
        expected = "The client can't be executed in the trustless mode without BLS sigs verification on Mainnet"
    )]
    pub fn test_panic_on_init_in_trustless_mode_without_bls_on_mainnet() {
        let (_headers, _updates, init_input) = get_test_data(Some(InitOptions {
            validate_updates: true,
            verify_bls_signatures: false,
            hashes_gc_threshold: 500,
            max_submitted_blocks_by_account: 7000,
            trusted_signer: None,
        }));

        Contract::init(init_input);
    }

    #[test]
    #[cfg_attr(feature = "bls", ignore)]
    #[should_panic(
        expected = "The client can't be executed in the trustless mode without BLS sigs verification on Mainnet"
    )]
    pub fn test_panic_on_init_in_trustless_mode_without_bls_feature_flag() {
        let (_headers, _updates, init_input) = get_test_data(Some(InitOptions {
            validate_updates: true,
            verify_bls_signatures: true,
            hashes_gc_threshold: 500,
            max_submitted_blocks_by_account: 7000,
            trusted_signer: None,
        }));

        Contract::init(init_input);
    }
}
