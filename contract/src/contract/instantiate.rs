use std::str::FromStr;

use utility::{
    consensus::Network,
    types::{ExecutionHeaderInfo, InitInput},
};

use crate::state::NonMappedState;

use super::Contract;

impl Contract<'_> {
    pub fn init(&mut self, args: InitInput) {
        let deps = self.ctx.get_deps_mut().unwrap();
        let network =
            Network::from_str(args.network.as_str()).unwrap_or_else(|e| panic!("{}", e.as_str()));

        #[cfg(feature = "mainnet")]
        {
            assert!(
                args.validate_updates,
                "The updates validation can't be disabled for mainnet"
            );

            assert!(
                (cfg!(feature = "bls") && args.verify_bls_signatures)
                    || args.trusted_signer.is_some(),
                "The client can't be executed in the trustless mode without BLS sigs verification on Mainnet"
            );
        }

        assert!(
            args.finalized_execution_header.calculate_hash()
                == args.finalized_beacon_header.execution_block_hash,
            "Invalid execution block"
        );

        let finalized_execution_header_info = ExecutionHeaderInfo {
            parent_hash: args.finalized_execution_header.parent_hash,
            block_number: args.finalized_execution_header.number,
            submitter: self.ctx.info.clone().unwrap().sender,
        };

        self.state.non_mapped.save(
            deps.borrow_mut().storage,
            &NonMappedState {
                trusted_signer: args.trusted_signer,
                validate_updates: args.validate_updates,
                verify_bls_signatures: args.verify_bls_signatures,
                hashes_gc_threshold: args.hashes_gc_threshold,
                network,
                max_submitted_blocks_by_account: args.max_submitted_blocks_by_account,
                finalized_beacon_header: args.finalized_beacon_header,
                finalized_execution_header: Some(finalized_execution_header_info),
                current_sync_committee: Some(args.current_sync_committee),
                next_sync_committee: Some(args.next_sync_committee),
                initialized: true,
            },
        ).unwrap();
    }
}
