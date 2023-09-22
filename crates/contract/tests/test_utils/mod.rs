use cosmwasm_std::Addr;
use cw_eth2_lc::msg::{InitInput, LightClientUpdate};
use lazy_static::lazy_static;
pub mod contract_interface;
pub mod e2e_test_client;
pub mod test_context;
pub mod unit_test_client;

pub fn read_client_update(filename: String) -> LightClientUpdate {
    serde_json::from_reader(std::fs::File::open(std::path::Path::new(&filename)).unwrap()).unwrap()
}

pub fn read_init_input(filename: String) -> InitInput {
    serde_json::from_reader(std::fs::File::open(std::path::Path::new(&filename)).unwrap()).unwrap()
}

/// Returns a pre-defined account_id from a list of 6.
pub fn accounts(id: usize) -> Addr {
    Addr::unchecked(["alice", "bob", "charlie", "danny", "eugene", "fargo"][id].to_string())
}

pub fn read_client_updates(lc_update: bool, slots: Vec<u64>) -> Vec<LightClientUpdate> {
    let mut updates = vec![];
    for slot in slots {
        let client_update = read_client_update(format!(
            "./tests/data/light_client_update_{slot}_{}.json",
            if lc_update { "lc_update" } else { "sc_update" },
        ));
        updates.push(client_update);
    }

    updates
}

pub fn get_test_data() -> (
    InitInput,
    &'static Vec<LightClientUpdate>,
    &'static Vec<LightClientUpdate>,
) {
    lazy_static! {
        static ref LC_UPDATES: Vec<LightClientUpdate> =
            read_client_updates(true, vec![6509789, 6504319, 6514015]);
        static ref SC_UPDATE_UPDATES: Vec<LightClientUpdate> =
            read_client_updates(false, vec![6509789]);
    }

    let init_input: InitInput = read_init_input("./tests/data/init_input.json".to_string());

    (init_input, &LC_UPDATES, &SC_UPDATE_UPDATES)
}
