#[macro_use]
extern crate serde_derive;

use std::sync::{Arc, Barrier};

use uuid::Uuid;

use crate::myisland::MyIsland;
use rust_in_peace::island::{IslandFactory, IslandEnv, Island};
use rust_in_peace::utils;
use rust_in_peace::simulation::Simulation;
use rust_in_peace::settings::ClientSettings;

mod myisland;
mod stats;
mod functions;
mod action;
mod agent;

const EXPECTED_ARGS_NUM: usize = 2;

struct MyIslandFactory;

impl IslandFactory for MyIslandFactory {
    fn create(&self, id: Uuid, island_env: IslandEnv) -> Box<dyn Island> {
        let args: Vec<String> = utils::parse_input_args(EXPECTED_ARGS_NUM);
        let settings_file_name = args[1].clone();
        let settings = load_settings(settings_file_name.clone());

        let mut island = MyIsland::new(
            id,
            island_env,
            &functions::rastrigin,
            settings.island.agents_number,
            settings.turns,
            settings.agent_config,
        );
        Box::new(island)
    }
}


fn main() {
    let factory = MyIslandFactory {};
    Simulation::start_simulation(Box::new(factory));
}


fn load_settings(file_name: String) -> ClientSettings {
    ClientSettings::new(file_name).unwrap()
}