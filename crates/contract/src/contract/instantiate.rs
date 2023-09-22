use super::Contract;
use crate::eth_utility::{compute_sync_committee_period, Network};
use crate::msg::InitInput;
use crate::state::NonMappedState;
use cosmwasm_std::DepsMut;
use electron_rs::verifier::near::{get_prepared_verifying_key, parse_verification_key};
use std::str::FromStr;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-eth2-cl";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

impl Contract<'_> {
    pub fn init(&mut self, deps: DepsMut, args: InitInput) {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION).unwrap();

        let vkey_lc_update =
            get_prepared_verifying_key(parse_verification_key(args.vkey_lc_update_string).unwrap());
        let vkey_sc_update =
            get_prepared_verifying_key(parse_verification_key(args.vkey_sc_update_string).unwrap());

        self.state
            .mapped
            .header_roots
            .save(deps.storage, args.head_slot, &args.header_root)
            .unwrap();

        self.state
            .mapped
            .execution_state_roots
            .save(deps.storage, args.head_slot, &args.execution_state_root)
            .unwrap();

        let period = compute_sync_committee_period(args.head_slot);
        self.state
            .mapped
            .sync_committee_poseidon_hashes
            .save(deps.storage, period, &args.sync_committee_poseidon_hash)
            .unwrap();

        let network =
            Network::from_str(args.network.as_str()).unwrap_or_else(|e| panic!("{}", e.as_str()));

        self.state
            .non_mapped
            .save(
                deps.storage,
                &NonMappedState {
                    admin: args.admin,
                    network,
                    head_slot: args.head_slot,
                    vkey_lc_update,
                    vkey_sc_update,
                },
            )
            .unwrap();
    }
}
