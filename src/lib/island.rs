use std::sync::{Arc, Barrier};

use uuid::Uuid;

use crate::address_book::AddressBook;
use std::time::Instant;

pub struct IslandEnv {
    pub address_book: AddressBook,
    pub stats_dir_path: String,
    pub start_time: Instant,
}

pub trait Island: Send {

    fn do_turn(&mut self, turn_number: u32);

    fn run_with_global_sync(&mut self);

    fn finish(&mut self);
}

pub trait IslandFactory {
    fn create(&self, island_id: Uuid, island_env: IslandEnv) -> Box<dyn Island>;
}
