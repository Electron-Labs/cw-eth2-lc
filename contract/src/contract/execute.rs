use cosmwasm_std::Addr;

use types::{eth2::LightClientUpdate, BlockHeader};
use utility::types::ExecutionHeaderInfo;

use super::Contract;

impl Contract {
    pub fn register_submitter(&mut self) {
        // TODO add validation?
        let addr = self.ctx.info.clone().unwrap().sender;
        assert!(
            !self.state.submitters.contains_key(&addr),
            "The account is already registered"
        );

        self.state.submitters.insert(addr, 0);
    }

    pub fn unregister_submitter(&mut self) {
        // TODO add validation?
        let addr = self.ctx.info.clone().unwrap().sender;
        if self.state.submitters.remove(&addr).is_none() {
            panic!("{}", "The account is not registered");
        }
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
                .get(&block_header.parent_hash.to_string())
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
        let insert_result = self
            .state
            .unfinalized_headers
            .insert(block_hash.to_string(), block_info);
        assert!(
            insert_result.is_none(),
            "{}",
            format!("The block {} already submitted!", &block_hash).as_str()
        );
    }

    pub fn update_trusted_signer(&mut self, trusted_signer: Option<Addr>) {
        assert!(self.ctx.info.clone().unwrap().sender == self.ctx.env.contract.address);
        self.state.trusted_signer = trusted_signer;
    }
}
