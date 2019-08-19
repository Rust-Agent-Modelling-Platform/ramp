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
use std::sync::Arc;
use std::thread;
use uuid::Uuid;

use crate::container::Container;

fn main() -> Result<(), ConfigError> {
    let settings = Settings::new()?;

    Logger::with_str("info")
        .format_for_stderr(flexi_logger::colored_default_format)
        .start()
        .unwrap();
    let simulation_dir_path = stats::create_simulation_dir(constants::STATS_DIR_NAME);
    stats::copy_simulation_settings(&simulation_dir_path);

    let agent_config = Arc::new(settings.agent_config);

    let mut threads = Vec::<thread::JoinHandle<_>>::new();
    for _ in 0..settings.islands {
        let container_id = Uuid::new_v4();
        let island_stats_dir_path =
            stats::create_island_stats_dir(&simulation_dir_path, &container_id);
        let mut container = Container::new(
            container_id,
            &functions::rastrigin,
            settings.container.agents_number,
            settings.turns,
            agent_config.clone(),
            island_stats_dir_path,
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
