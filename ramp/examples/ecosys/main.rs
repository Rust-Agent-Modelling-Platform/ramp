#[macro_use]
extern crate serde_derive;

use ramp::island::{Island, IslandEnv, IslandFactory};
use ramp::simulation::Simulation;
use ramp::utils;
use std::sync::Arc;
use uuid::Uuid;

use crate::settings::SimulationSettings;
use crate::ws_island::WSIsland;
use ramp::metrics::MetricHub;

mod agent_types;
mod settings;
mod sheep;
mod wolves;
mod ws_island;
mod ws_utils;

struct WSIslandFactory;
const EXPECTED_ARGS_NUM: usize = 3;

// MN - metric name
const WOLVES_MN: &str = "wolves";
const SHEEP_MN: &str = "sheep";

// LN - label name
const ISLAND_ID_LN: &str = "island_id";

impl IslandFactory for WSIslandFactory {
    fn create(&self, id: Uuid, island_env: IslandEnv) -> Box<dyn Island> {
        let args: Vec<String> = utils::parse_input_args(EXPECTED_ARGS_NUM);
        let settings_file_name = args[2].clone();
        let settings = load_settings(settings_file_name.clone());

        let island = WSIsland::new(
            id,
            island_env,
            Arc::new(settings.island_settings),
            Arc::new(settings.sheep_settings),
            Arc::new(settings.wolf_settings),
        );

        Box::new(island)
    }
}

fn main() {
    let mut metrics = MetricHub::default();
    register_metrics(&mut metrics);

    let factory = WSIslandFactory {};
    Simulation::start_simulation(Box::new(factory), metrics);
}

fn load_settings(file_name: String) -> SimulationSettings {
    SimulationSettings::new(file_name).unwrap()
}

fn register_metrics(metrics: &mut MetricHub) {
    metrics.register_int_gauge_vec(WOLVES_MN, "wolves number per turn", &[ISLAND_ID_LN]);
    metrics.register_int_gauge_vec(SHEEP_MN, "sheep number per turn", &[ISLAND_ID_LN]);
}
