use cosmwasm_std::Addr;

use types::{eth2::LightClientUpdate, BlockHeader};
use utility::types::ExecutionHeaderInfo;

use super::Contract;

impl Contract<'_> {
    pub fn register_submitter(&mut self) {
        let deps = self.ctx.get_deps_mut().unwrap();
        let _non_mapped_state = self
            .state
            .non_mapped
            .load(deps.borrow_mut().storage)
            .unwrap();
        // TODO add validation?
        let addr = self.ctx.info.clone().unwrap().sender;
        assert!(
            !self
                .state
                .mapped
                .submitters
                .has(deps.borrow_mut().storage, addr.clone()),
            "The account is already registered"
        );

        self.state
            .mapped
            .submitters
            .save(deps.borrow_mut().storage, addr, &0)
            .unwrap();
    }

    pub fn unregister_submitter(&mut self) {
        let deps = self.ctx.get_deps_mut().unwrap();
        let _non_mapped_state = self
            .state
            .non_mapped
            .load(deps.borrow_mut().storage)
            .unwrap();

        // TODO add validation?
        let addr = self.ctx.info.clone().unwrap().sender;
        self.state
            .mapped
            .submitters
            .remove(deps.borrow_mut().storage, addr);
    }

    pub fn submit_beacon_chain_light_client_update(&mut self, update: LightClientUpdate) {
        let deps = self.ctx.get_deps_mut().unwrap();
        let non_mapped_state = self
            .state
            .non_mapped
            .load(deps.borrow_mut().storage)
            .unwrap();

        self.is_light_client_update_allowed();

        if non_mapped_state.validate_updates {
            self.validate_light_client_update(&update);
        }

        self.commit_light_client_update(update);
    }

    pub fn submit_execution_header(&mut self, block_header: BlockHeader) {
        let deps = self.ctx.get_deps_mut().unwrap();
        let non_mapped_state = self
            .state
            .non_mapped
            .load(deps.borrow_mut().storage)
            .unwrap();

        if non_mapped_state
            .finalized_beacon_header
            .execution_block_hash
            != block_header.parent_hash
        {
            self.state
                .mapped
                .unfinalized_headers
                .load(
                    deps.borrow_mut().storage,
                    block_header.parent_hash.to_string(),
                )
                .unwrap_or_else(|_| {
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

        let submitter = self.ctx.info.clone().unwrap().sender;
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

        assert!(
            self.state
                .mapped
                .unfinalized_headers
                .load(deps.borrow_mut().storage, block_hash.to_string())
                .is_err(),
            "{}",
            format!("The block {} already submitted!", &block_hash).as_str()
        );

        self.state
            .mapped
            .unfinalized_headers
            .save(
                deps.borrow_mut().storage,
                block_hash.to_string(),
                &block_info,
            )
            .unwrap();
    }

    pub fn update_trusted_signer(&mut self, trusted_signer: Option<Addr>) {
        let deps = self.ctx.get_deps_mut().unwrap();
        let mut non_mapped_state = self
            .state
            .non_mapped
            .load(deps.borrow_mut().storage)
            .unwrap();

        assert!(self.ctx.info.clone().unwrap().sender == self.ctx.env.contract.address);
        non_mapped_state.trusted_signer = trusted_signer;

        self.state
            .non_mapped
            .save(deps.borrow_mut().storage, &non_mapped_state)
            .unwrap();
    }
}
