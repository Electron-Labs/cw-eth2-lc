use cosmwasm_std::{Env, MessageInfo};

pub struct ContractContext {
    pub env: Env,
    pub info: Option<MessageInfo>,
}

impl ContractContext {
    pub fn new(env: Env, info: Option<MessageInfo>) -> Self {
        Self { env, info }
    }
}
