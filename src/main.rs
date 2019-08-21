#[macro_use]
extern crate serde_derive;

mod action;
mod address_book;
mod agent;
mod constants;
mod container;
mod functions;
mod message;
mod settings;
mod stats;

use config;
use config::ConfigError;
use flexi_logger::Logger;
use settings::Settings;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use uuid::Uuid;

use crate::address_book::AddressBook;
use crate::container::Container;
use crate::message::Message;
use crate::settings::AgentConfig;

fn main() -> Result<(), ConfigError> {
    init_logger();

    let settings = Settings::new()?;
    let simulation_dir_path = stats::create_simulation_dir(constants::STATS_DIR_NAME);
    let agent_config = Arc::new(settings.agent_config);
    let (txes, rxes) = create_channels(settings.islands);
    let island_ids = create_island_ids(settings.islands);

    stats::copy_simulation_settings(&simulation_dir_path);

    start_simulation(
        settings,
        agent_config,
        txes,
        rxes,
        island_ids,
        simulation_dir_path,
    );

    Ok(())
}

fn start_simulation(
    settings: Settings,
    agent_config: Arc<AgentConfig>,
    txes: Vec<Sender<Message>>,
    mut rxes: Vec<Receiver<Message>>,
    island_ids: Vec<Uuid>,
    simulation_dir_path: String,
) {
    let mut threads = Vec::<thread::JoinHandle<_>>::new();
    for island_no in 0..settings.islands {
        let island_stats_dir_path =
            stats::create_island_stats_dir(&simulation_dir_path, &island_ids[island_no as usize]);
        let address_book = create_address_book(&txes, &mut rxes, &island_ids, island_no as usize);
        let mut container = Container::new(
            island_ids[island_no as usize],
            address_book,
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
}

fn init_logger() {
    Logger::with_str("info")
        .format_for_stderr(flexi_logger::colored_default_format)
        .start()
        .unwrap();
}

fn create_channels(islands_number: u32) -> (Vec<Sender<Message>>, Vec<Receiver<Message>>) {
    let mut txes = Vec::<Sender<Message>>::new();
    let mut rxes = Vec::<Receiver<Message>>::new();
    for _ in 0..islands_number {
        let (tx, rx) = mpsc::channel();
        txes.push(tx);
        rxes.push(rx);
    }
    (txes, rxes)
}

fn create_island_ids(islands_number: u32) -> Vec<Uuid> {
    let mut island_ids = Vec::<Uuid>::new();
    for _ in 0..islands_number {
        island_ids.push(Uuid::new_v4());
    }
    island_ids
}

fn create_address_book(
    txes: &[Sender<Message>],
    rxes: &mut Vec<Receiver<Message>>,
    island_ids: &[Uuid],
    island_no: usize,
) -> AddressBook {
    let mut addresses: HashMap<Uuid, (Sender<Message>, bool)> = HashMap::new();

    for (i, tx) in txes.iter().enumerate().take(txes.len()) {
        if i != island_no {
            addresses.insert(island_ids[i], (mpsc::Sender::clone(tx), true));
        }
    }
    let rx = rxes.remove(0);
    AddressBook::new(addresses, rx)
}
