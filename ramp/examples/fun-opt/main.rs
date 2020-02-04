#[macro_use]
extern crate serde_derive;

use ramp::island::{Island, IslandEnv, IslandFactory};
use ramp::metrics::MetricHub;
use ramp::simulation::Simulation;
use ramp::utils;
use std::sync::Arc;
use uuid::Uuid;

use crate::myisland::MyIsland;
use crate::settings::SimulationSettings;

mod action;
mod agent;
mod functions;
mod myisland;
mod settings;

const EXPECTED_ARGS_NUM: usize = 3;

struct MyIslandFactory;

// MN - metric name
const PROCREATIONS_MN: &str = "procreations";
const DEADS_MN: &str = "deads";
const MEETINGS_MN: &str = "meetings";
const BEST_FITNESS_MN: &str = "fitness_best";
const ALL_RECV_MIGR_MN: &str = "migrations_recv_all";
const ALL_SENT_MIGR_MN: &str = "migrations_sent_all";
const LOC_RECV_MIGR_MN: &str = "migrations_sent_loc";
const GLOB_RECV_MIGR_MN: &str = "migrations_sent_glob";

// LN - label name
const ISLAND_ID_LN: &str = "island_id";

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
            Arc::new(settings.agent_settings),
        );
        Box::new(island)
    }
}

fn main() {
    let mut metrics = MetricHub::default();
    register_metrics(&mut metrics);

    let factory = MyIslandFactory {};
    Simulation::start_simulation(Box::new(factory), metrics);
}

fn load_settings(file_name: String) -> SimulationSettings {
    SimulationSettings::new(file_name).unwrap()
}

fn register_metrics(metrics: &mut MetricHub) {
    metrics.register_int_gauge_vec(PROCREATIONS_MN, "procreations per turn", &[ISLAND_ID_LN]);
    metrics.register_int_gauge_vec(DEADS_MN, "deads per turn", &[ISLAND_ID_LN]);
    metrics.register_int_gauge_vec(MEETINGS_MN, "meetings per turn", &[ISLAND_ID_LN]);
    metrics.register_gauge_vec(BEST_FITNESS_MN, "best fitness in turn", &[ISLAND_ID_LN]);
    metrics.register_int_gauge_vec(ALL_RECV_MIGR_MN, "all recv migrations", &[ISLAND_ID_LN]);
    metrics.register_int_gauge_vec(ALL_SENT_MIGR_MN, "all sent migrations", &[ISLAND_ID_LN]);
    metrics.register_int_gauge_vec(LOC_RECV_MIGR_MN, "local sent migrations", &[ISLAND_ID_LN]);
    metrics.register_int_gauge_vec(GLOB_RECV_MIGR_MN, "global sent migrations", &[ISLAND_ID_LN]);
}
