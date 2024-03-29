use crate::msg::ExecuteMsg;
use cosmwasm_std::{to_binary, Addr, Binary, CosmosMsg, StdResult, WasmMsg};
use serde::{Deserialize, Serialize};

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CwTemplateContract(pub Addr);

impl CwTemplateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }
}

pub trait TryToBinary {
    fn try_to_binary(&self) -> StdResult<Binary>;
}

impl<T: serde::Serialize + ?Sized> TryToBinary for T {
    fn try_to_binary(&self) -> StdResult<Binary> {
        to_binary(self)
    }
}
