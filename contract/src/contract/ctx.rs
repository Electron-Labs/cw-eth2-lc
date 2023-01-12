use std::{cell::RefCell, rc::Rc};

use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo};

pub struct ContractContext<'a> {
    pub env: Env,
    pub info: Option<MessageInfo>,
    pub deps: ContextDeps<'a>,
}

#[derive(Clone)]
pub enum ContextDeps<'a> {
    Mutable(Rc<RefCell<DepsMut<'a>>>),
    Immutable(Deps<'a>),
}

impl<'a> ContractContext<'a> {
    pub fn new(env: Env, info: Option<MessageInfo>, deps: Deps<'a>) -> Self {
        Self {
            env,
            info,
            deps: ContextDeps::Immutable(deps),
        }
    }

    pub fn new_mut(env: Env, info: Option<MessageInfo>, deps: DepsMut<'a>) -> Self {
        Self {
            env,
            info,
            deps: ContextDeps::Mutable(Rc::new(RefCell::new(deps))),
        }
    }

    pub fn get_deps<'b>(&'b self) -> crate::Result<Deps<'b>> {
        match &self.deps {
            ContextDeps::Mutable(d) => panic!("something"),
            ContextDeps::Immutable(d) => Ok(d.clone()),
        }
    }

    pub fn get_deps_mut(&self) -> crate::Result<Rc<RefCell<DepsMut<'a>>>> {
        match &self.deps {
            ContextDeps::Mutable(d) => Ok(d.clone()),
            ContextDeps::Immutable(_) => Err("could not get mutable deps".into()),
        }
    }
}
