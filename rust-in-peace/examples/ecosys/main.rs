#[macro_use]
extern crate serde_derive;

use rust_in_peace::island::{Island, IslandEnv, IslandFactory};
use rust_in_peace::simulation::Simulation;
use rust_in_peace::utils;
use std::sync::Arc;
use uuid::Uuid;

use crate::settings::SimulationSettings;
use crate::ws_island::WSIsland;
use rust_in_peace::metrics::MetricHub;

mod agent_types;
mod settings;
mod sheep;
mod wolves;
mod ws_island;
mod ws_utils;

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
    //    metrics.register_int_gauge_vec(PROCREATIONS_MN, "procreations per turn", &[ISLAND_ID_LN]);
    //    metrics.register_int_gauge_vec(DEADS_MN, "deads per turn", &[ISLAND_ID_LN]);
    //    metrics.register_int_gauge_vec(MEETINGS_MN, "meetings per turn", &[ISLAND_ID_LN]);
    //    metrics.register_gauge_vec(BEST_FITNESS_MN, "best fitness in turn", &[ISLAND_ID_LN]);
    //    metrics.register_int_gauge_vec(ALL_RECV_MIGR_MN, "all recv migrations", &[ISLAND_ID_LN]);
    //    metrics.register_int_gauge_vec(ALL_SENT_MIGR_MN, "all sent migrations", &[ISLAND_ID_LN]);
    //    metrics.register_int_gauge_vec(LOC_RECV_MIGR_MN, "local sent migrations", &[ISLAND_ID_LN]);
    //    metrics.register_int_gauge_vec(GLOB_RECV_MIGR_MN, "global sent migrations", &[ISLAND_ID_LN]);
}
