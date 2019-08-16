#[macro_use]
extern crate serde_derive;

mod action;
mod agent;
mod constants;
mod container;
mod functions;
mod settings;
mod stats;
use config;
use config::ConfigError;
use flexi_logger::Logger;
use settings::Settings;
use std::thread;
use uuid::Uuid;
use std::sync::Arc;
use std::mem;

use crate::container::Container;
use crate::agent::AgentConfig;

fn main() -> Result<(), ConfigError> {
    let settings = Settings::new()?;

    Logger::with_str("info")
        .format_for_stderr(flexi_logger::colored_default_format)
        .start()
        .unwrap();
    let simulation_dir_path = stats::create_simulation_dir(constants::STATS_DIR_NAME);
    stats::copy_simulation_settings(&simulation_dir_path);

    let agent_config = Arc::new(AgentConfig {
        genotype_dim: settings.agent.genotype_dim,
        minimum: settings.agent.minimum,
        mutation_rate: settings.agent.mutation_rate,
        procreation_prob: settings.agent.procreation_prob,
        procreation_penalty: settings.agent.procreation_penalty,
        meeting_penalty: settings.agent.meeting_penalty
    });

    let mut threads = Vec::<thread::JoinHandle<_>>::new();
    for _ in 0..settings.islands {
        let container_id = Uuid::new_v4();
        let island_stats_dir_path = stats::create_island_stats_dir(&simulation_dir_path, &container_id);
        let mut container = Container::new(
            container_id,
            &functions::rastrigin,
            settings.container.agents_number,
            (-5.12, 5.12),
            settings.turns,
            agent_config.clone(),
            island_stats_dir_path
        );
        threads.push(thread::spawn(move || {
            container.run();
        }));
    }
    for thread in threads {
        thread.join().unwrap();
    }

    Ok(())
}
