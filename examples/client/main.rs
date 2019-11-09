#[macro_use]
extern crate serde_derive;

use uuid::Uuid;

use rust_in_peace::island::{Island, IslandEnv, IslandFactory};
use rust_in_peace::simulation::Simulation;
use rust_in_peace::utils;

use crate::myisland::MyIsland;
use crate::settings::SimulationSettings;

mod action;
mod agent;
mod functions;
mod myisland;
mod settings;
mod stats;

const EXPECTED_ARGS_NUM: usize = 3;

struct MyIslandFactory;

impl IslandFactory for MyIslandFactory {
    fn create(&self, id: Uuid, island_env: IslandEnv) -> Box<dyn Island> {
        let args: Vec<String> = utils::parse_input_args(EXPECTED_ARGS_NUM);
        let settings_file_name = args[2].clone();
        let settings = load_settings(settings_file_name.clone());

        let island = MyIsland::new(
            id,
            island_env,
            &functions::rastrigin,
            settings.island_settings.agents_number,
            settings.agent_settings,
        );
        Box::new(island)
    }
}

fn main() {
    let factory = MyIslandFactory {};
    Simulation::start_simulation(Box::new(factory));
}

fn load_settings(file_name: String) -> SimulationSettings {
    SimulationSettings::new(file_name).unwrap()
}
