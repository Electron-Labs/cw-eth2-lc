use cw_storage_plus::Item;

use crate::rainbow::Eth2ClientState;

pub const STATE: Item<Eth2ClientState> = Item::new("state");
