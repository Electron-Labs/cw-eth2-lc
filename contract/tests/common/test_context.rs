use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    Addr, DepsMut,
};
use cw_eth2_lc::{contract::Contract, Result};

use types::{eth2::LightClientUpdate, BlockHeader};

use crate::common::{
    contract_interface::UnitTestContractImplementation,
    test_contract_client::IntegrationTestContractImplementation,
};

use super::{contract_interface::ContractInterface, get_test_data, InitOptions};

pub struct TestContext<'a, 'b> {
    pub contract: Box<dyn ContractInterface + 'b>,
    pub headers: &'a Vec<BlockHeader>,
    pub updates: &'a Vec<LightClientUpdate>,
}

pub fn get_test_context<'a>(
    _deps: DepsMut<'a>,
    contract_caller: Addr,
    init_options: Option<InitOptions>,
) -> TestContext<'static, 'a> {
    let (headers, updates, init_input) = get_test_data(init_options);
    let contract = if true {
        let contract = Contract::new(
            mock_env(),
            Some(mock_info(contract_caller.to_string().as_str(), &[])),
        );
        let mut contract = UnitTestContractImplementation {
            contract,
            deps: mock_dependencies(),
        };
        contract.contract.init(contract.deps.as_mut(), init_input);
        Box::new(contract) as Box<dyn ContractInterface + 'a>
    } else {
        let contract = Box::new(IntegrationTestContractImplementation::new(init_input).unwrap())
            as Box<dyn ContractInterface + 'a>;
        contract
    };

    assert_eq!(contract.last_block_number().unwrap(), headers[0].number);

    TestContext {
        contract,
        headers,
        updates,
    }
}

pub fn submit_and_check_execution_headers<'a>(
    contract: &mut Box<dyn ContractInterface + 'a>,
    headers: Vec<&BlockHeader>,
) -> Result<()> {
    for header in headers {
        contract.submit_execution_header(header.clone())?;
        assert!(contract.is_known_execution_header(header.calculate_hash())?);
        assert!(contract.block_hash_safe(header.number)?.is_none());
    }

    Ok(())
}
