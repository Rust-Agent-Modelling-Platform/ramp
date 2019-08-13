#[macro_use]
extern crate serde_derive;

mod action;
mod agent;
mod container;
mod functions;
mod settings;

use config;
use config::ConfigError;
use flexi_logger::Logger;
use settings::Settings;
use std::thread;

use crate::container::Container;

fn main() -> Result<(), ConfigError> {
    let settings = Settings::new()?;

    Logger::with_str("info")
        .format_for_stderr(flexi_logger::colored_default_format)
        .start()
        .unwrap();

    let mut threads = Vec::<thread::JoinHandle<_>>::new();

    for _ in 0..settings.islands {
        threads.push(thread::spawn(move || {
            let mut container = Container::new(
                &functions::rastrigin,
                settings.container.agents_number,
                4,
                (-5.12, 5.12),
                settings.container.max_agents_number,
                settings.turns,
            );
            container.run();
        }));
    }
    for thread in threads {
        thread.join().unwrap();
    }

    Ok(())
}
