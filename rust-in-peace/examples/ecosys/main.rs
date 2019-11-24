#[macro_use]
extern crate serde_derive;

use rust_in_peace::island::{Island, IslandEnv, IslandFactory};
use rust_in_peace::simulation::Simulation;
use rust_in_peace::utils;
use std::sync::Arc;
use uuid::Uuid;

use crate::ws_island::WSIsland;
use crate::settings::SimulationSettings;

mod sheep;
mod wolves;
mod ws_island;
mod settings;

struct WSIslandFactory;
const EXPECTED_ARGS_NUM: usize = 3;

impl IslandFactory for WSIslandFactory {
    fn create(&self, id: Uuid, island_env: IslandEnv) -> Box<dyn Island> {
        let args: Vec<String> = utils::parse_input_args(EXPECTED_ARGS_NUM);
        let settings_file_name = args[2].clone();
        let settings = load_settings(settings_file_name.clone());

        let island = WSIsland::new(
            id,
            island_env,
            Arc::new(settings.sheep_settings),
            Arc::new(settings.wolf_settings) );

        Box::new(island)
    }
}

fn main() {
    let factory = WSIslandFactory {};
    Simulation::start_simulation(Box::new(factory));
}

fn load_settings(file_name: String) -> SimulationSettings {
    SimulationSettings::new(file_name).unwrap()
}