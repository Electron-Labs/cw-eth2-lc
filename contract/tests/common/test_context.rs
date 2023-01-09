
use cosmwasm_std::{Addr};
use cw_eth2_lc::{
    contract::{Contract, ContractContext},
    state::ContractState,
    Result,
};

use types::{eth2::LightClientUpdate, BlockHeader};

use crate::common::test_contract_client::IntegrationTestContractImplementation;

use super::{contract_interface::ContractInterface, get_test_data, InitOptions};

pub struct TestContext<'a> {
    pub contract: Box<dyn ContractInterface>,
    pub headers: &'a Vec<BlockHeader>,
    pub updates: &'a Vec<LightClientUpdate>,
}

pub fn get_test_context(
    contract_caller: Addr,
    init_options: Option<InitOptions>,
) -> TestContext<'static> {
    let (headers, updates, init_input) = get_test_data(init_options);
    let contract = if true {
        let mut contract = Contract::new(
            ContractContext {
                env: cosmwasm_std::testing::mock_env(),
                info: Some(cosmwasm_std::testing::mock_info(
                    contract_caller.as_str(),
                    Vec::new().as_slice(),
                )),
            },
            ContractState::default(),
        );
        contract.init(init_input);
        Box::new(contract) as Box<dyn ContractInterface>
    } else {
        let contract = Box::new(IntegrationTestContractImplementation::new(init_input).unwrap())
            as Box<dyn ContractInterface>;
        contract
    };

    assert_eq!(contract.last_block_number().unwrap(), headers[0].number);

    TestContext {
        contract,
        headers,
        updates,
    }
}

pub fn submit_and_check_execution_headers(
    contract: &mut Box<dyn ContractInterface>,
    headers: Vec<&BlockHeader>,
) -> Result<()> {
    for header in headers {
        contract.submit_execution_header(header.clone())?;
        assert!(contract.is_known_execution_header(header.calculate_hash())?);
        assert!(contract.block_hash_safe(header.number)?.is_none());
    }

    Ok(())
}
