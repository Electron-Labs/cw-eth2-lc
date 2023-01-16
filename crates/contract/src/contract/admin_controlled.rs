use cosmwasm_std::{Deps, DepsMut};

use super::Contract;

pub type Mask = u128;

pub const PAUSE_SUBMIT_UPDATE: Mask = 1;

pub trait AdminControlled {
    fn is_owner(&self, deps: Deps) -> bool;

    /// Return the current mask representing all paused events.
    fn get_paused(&self, deps: Deps) -> Mask;

    /// Update mask with all paused events.
    /// Implementor is responsible for guaranteeing that this function can only be
    /// called by owner of the contract.
    fn set_paused(&self, deps: DepsMut, paused: Mask);

    /// Return if the contract is paused for the current flag and user
    fn is_paused(&self, flag: Mask, deps: Deps) -> bool {
        (self.get_paused(deps) & flag) != 0 && !self.is_owner(deps)
    }

    fn check_not_paused(&self, flag: Mask, deps: Deps) {
        assert!(!self.is_paused(flag, deps));
    }
}

impl AdminControlled for Contract<'_> {
    fn is_owner(&self, deps: Deps) -> bool {
        let non_mapped_state = self.state.non_mapped.load(deps.storage).unwrap();
        return self.ctx.info.as_ref().unwrap().sender == non_mapped_state.admin;
    }

    fn get_paused(&self, deps: Deps) -> Mask {
        let non_mapped_state = self.state.non_mapped.load(deps.storage).unwrap();
        non_mapped_state.paused
    }

    fn set_paused(&self, deps: DepsMut, paused: Mask) {
        if !self.is_owner(deps.as_ref()) {
            panic!("cannot set pause if not owner")
        }

        let mut non_mapped_state = self.state.non_mapped.load(deps.storage).unwrap();
        non_mapped_state.paused = paused;
        self.state
            .non_mapped
            .save(deps.storage, &non_mapped_state)
            .unwrap();
    }
}
