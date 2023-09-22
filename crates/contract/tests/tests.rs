use cw_eth2_lc::eth_utility::compute_sync_committee_period;
use cw_eth2_lc::msg::{
    ExecutionStateRootResponse, HeadResponse, HeaderRootResponse, SyncCommitteePoseidonHashResponse,
};
use test_utils::accounts;
use test_utils::test_context::{get_test_context, TestContext};

pub mod test_utils;

#[test]
pub fn test_submit_correct_lc_update() {
    let TestContext {
        mut contract,
        lc_updates,
        sc_updates: _,
    } = get_test_context(accounts(0));

    contract.update_light_client(lc_updates[0].clone()).unwrap();

    assert_eq!(
        contract.head().unwrap(),
        HeadResponse {
            head: lc_updates[0].finalized_slot
        }
    );
    assert_eq!(
        contract.header_root(lc_updates[0].finalized_slot).unwrap(),
        HeaderRootResponse {
            header_root: Some(lc_updates[0].finalized_header_root.clone())
        }
    );
    assert_eq!(
        contract
            .execution_state_root(lc_updates[0].finalized_slot)
            .unwrap(),
        ExecutionStateRootResponse {
            execution_state_root: Some(lc_updates[0].execution_state_root.clone())
        }
    );
}

#[test]
pub fn test_submit_correct_lc_update_identical() {
    let TestContext {
        mut contract,
        lc_updates,
        sc_updates: _,
    } = get_test_context(accounts(0));

    contract.update_light_client(lc_updates[0].clone()).unwrap();
    contract.update_light_client(lc_updates[0].clone()).unwrap();

    assert_eq!(
        contract.head().unwrap(),
        HeadResponse {
            head: lc_updates[0].finalized_slot
        }
    );
    assert_eq!(
        contract.header_root(lc_updates[0].finalized_slot).unwrap(),
        HeaderRootResponse {
            header_root: Some(lc_updates[0].finalized_header_root.clone())
        }
    );
    assert_eq!(
        contract
            .execution_state_root(lc_updates[0].finalized_slot)
            .unwrap(),
        ExecutionStateRootResponse {
            execution_state_root: Some(lc_updates[0].execution_state_root.clone())
        }
    );
}

#[test]
#[should_panic(expected = "Failed to verify lc_update proof")]
pub fn test_submit_incorrect_lc_update_zk_proof() {
    let TestContext {
        mut contract,
        lc_updates,
        sc_updates: _,
    } = get_test_context(accounts(0));

    let mut light_client_update = lc_updates[0].clone();
    light_client_update.lc_update_proof = lc_updates[1].lc_update_proof.clone();

    contract.update_light_client(light_client_update).unwrap();
}

#[test]
#[should_panic(expected = "Finalized slot is behind head slot")]
pub fn test_submit_incorrect_lc_update_lower_height() {
    let TestContext {
        mut contract,
        lc_updates,
        sc_updates: _,
    } = get_test_context(accounts(0));

    contract.update_light_client(lc_updates[1].clone()).unwrap();
}

#[test]
#[should_panic(expected = "Sync committee hash not known")]
pub fn test_submit_incorrect_lc_update_unknown_sync_committee() {
    let TestContext {
        mut contract,
        lc_updates,
        sc_updates: _,
    } = get_test_context(accounts(0));

    contract.update_light_client(lc_updates[2].clone()).unwrap();
}

#[test]
pub fn test_submit_correct_sc_update() {
    let TestContext {
        mut contract,
        lc_updates: _,
        sc_updates,
    } = get_test_context(accounts(0));

    contract.update_light_client(sc_updates[0].clone()).unwrap();

    assert_eq!(
        contract.head().unwrap(),
        HeadResponse {
            head: sc_updates[0].finalized_slot
        }
    );
    assert_eq!(
        contract.header_root(sc_updates[0].finalized_slot).unwrap(),
        HeaderRootResponse {
            header_root: Some(sc_updates[0].finalized_header_root.clone())
        }
    );
    assert_eq!(
        contract
            .execution_state_root(sc_updates[0].finalized_slot)
            .unwrap(),
        ExecutionStateRootResponse {
            execution_state_root: Some(sc_updates[0].execution_state_root.clone())
        }
    );
    assert_eq!(
        contract
            .sync_committee_poseidon_hash(
                compute_sync_committee_period(sc_updates[0].finalized_slot) + 1
            )
            .unwrap(),
        SyncCommitteePoseidonHashResponse {
            sync_committee_poseidon_hash: Some(
                sc_updates[0]
                    .next_sync_committee
                    .as_ref()
                    .unwrap()
                    .sync_committee_poseidon_hash
                    .clone()
            )
        }
    );
}

#[test]
pub fn test_submit_correct_sc_update_identical() {
    let TestContext {
        mut contract,
        lc_updates: _,
        sc_updates,
    } = get_test_context(accounts(0));

    contract.update_light_client(sc_updates[0].clone()).unwrap();
    contract.update_light_client(sc_updates[0].clone()).unwrap();

    assert_eq!(
        contract.head().unwrap(),
        HeadResponse {
            head: sc_updates[0].finalized_slot
        }
    );
    assert_eq!(
        contract.header_root(sc_updates[0].finalized_slot).unwrap(),
        HeaderRootResponse {
            header_root: Some(sc_updates[0].finalized_header_root.clone())
        }
    );
    assert_eq!(
        contract
            .execution_state_root(sc_updates[0].finalized_slot)
            .unwrap(),
        ExecutionStateRootResponse {
            execution_state_root: Some(sc_updates[0].execution_state_root.clone())
        }
    );
    assert_eq!(
        contract
            .sync_committee_poseidon_hash(
                compute_sync_committee_period(sc_updates[0].finalized_slot) + 1
            )
            .unwrap(),
        SyncCommitteePoseidonHashResponse {
            sync_committee_poseidon_hash: Some(
                sc_updates[0]
                    .next_sync_committee
                    .as_ref()
                    .unwrap()
                    .sync_committee_poseidon_hash
                    .clone()
            )
        }
    );
}

#[test]
#[should_panic(expected = "Failed to verify sc_update proof")]
pub fn test_submit_incorrect_sc_update_zk_proof() {
    let TestContext {
        mut contract,
        lc_updates: _,
        sc_updates,
    } = get_test_context(accounts(0));

    let mut light_client_update = sc_updates[0].clone();
    light_client_update
        .next_sync_committee
        .as_mut()
        .unwrap()
        .sc_update_proof = sc_updates[0].lc_update_proof.clone();
    contract.update_light_client(light_client_update).unwrap();
}

#[test]
pub fn test_submit_next_period_lc_update_after_sc_update() {
    let TestContext {
        mut contract,
        lc_updates,
        sc_updates,
    } = get_test_context(accounts(0));

    contract.update_light_client(sc_updates[0].clone()).unwrap();
    contract.update_light_client(lc_updates[2].clone()).unwrap();

    assert_eq!(
        contract.head().unwrap(),
        HeadResponse {
            head: lc_updates[2].finalized_slot
        }
    );
    assert_eq!(
        contract.header_root(lc_updates[2].finalized_slot).unwrap(),
        HeaderRootResponse {
            header_root: Some(lc_updates[2].finalized_header_root.clone())
        }
    );
    assert_eq!(
        contract
            .execution_state_root(lc_updates[2].finalized_slot)
            .unwrap(),
        ExecutionStateRootResponse {
            execution_state_root: Some(lc_updates[2].execution_state_root.clone())
        }
    );
}
