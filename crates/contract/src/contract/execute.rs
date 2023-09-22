use super::Contract;
use crate::eth_utility::compute_sync_committee_period;
use crate::msg::LightClientUpdate;
use cosmwasm_std::DepsMut;

impl Contract<'_> {
    pub fn update_light_client(&self, deps: DepsMut, light_client_update: LightClientUpdate) {
        let mut non_mapped_state = self.state.non_mapped.load(deps.storage).unwrap();

        // processing lc_update
        if light_client_update.finalized_slot < non_mapped_state.head_slot {
            panic!("Finalized slot is behind head slot");
        }

        if light_client_update.participation * 3 < 2 * 512 {
            panic!("Participation is less than 2/3rd threshold");
        }

        self.lc_update_proof_verify(deps.as_ref(), light_client_update.clone());
        non_mapped_state.head_slot = light_client_update.finalized_slot;
        self.state
            .non_mapped
            .save(deps.storage, &non_mapped_state)
            .unwrap();

        // updating roots
        let root = self
            .state
            .mapped
            .header_roots
            .may_load(deps.storage, light_client_update.finalized_slot)
            .unwrap();
        match root {
            Some(r) => assert_eq!(
                &light_client_update.finalized_header_root, &r,
                "Header root already set with different value"
            ),
            None => {
                self.state
                    .mapped
                    .header_roots
                    .save(
                        deps.storage,
                        light_client_update.finalized_slot,
                        &light_client_update.finalized_header_root,
                    )
                    .unwrap();
            }
        }

        let root = self
            .state
            .mapped
            .execution_state_roots
            .may_load(deps.storage, light_client_update.finalized_slot)
            .unwrap();
        match root {
            Some(r) => assert_eq!(
                &light_client_update.execution_state_root, &r,
                "Header root already set with different value"
            ),
            None => {
                self.state
                    .mapped
                    .execution_state_roots
                    .save(
                        deps.storage,
                        light_client_update.finalized_slot,
                        &light_client_update.execution_state_root,
                    )
                    .unwrap();
            }
        }

        // processing sc_update
        if let Some(next_sync_committee_update) = light_client_update.clone().next_sync_committee {
            self.sc_update_proof_verify(deps.as_ref(), light_client_update.clone());
            let next_period = compute_sync_committee_period(light_client_update.finalized_slot) + 1;
            let hash = self
                .state
                .mapped
                .sync_committee_poseidon_hashes
                .may_load(deps.storage, next_period)
                .unwrap();
            match hash {
                Some(h) => assert_eq!(
                    &next_sync_committee_update.sync_committee_poseidon_hash, &h,
                    "Sync Committee poseidon hash already set with different value"
                ),
                None => {
                    self.state
                        .mapped
                        .sync_committee_poseidon_hashes
                        .save(
                            deps.storage,
                            next_period,
                            &next_sync_committee_update.sync_committee_poseidon_hash,
                        )
                        .unwrap();
                }
            }
        }
    }
}
