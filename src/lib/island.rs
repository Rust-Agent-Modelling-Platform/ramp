use std::sync::{Arc, Barrier};

use uuid::Uuid;

use crate::address_book::AddressBook;

pub struct IslandEnv {
    pub address_book: AddressBook,
    pub stats_dir_path: String,
    pub islands_sync: Option<Arc<Barrier>>
}

pub trait Island: Send {

    fn run(&mut self);

    fn run_with_global_sync(&mut self);
}

pub trait IslandFactory {
    fn create(&self, island_id: Uuid, island_env: IslandEnv) -> Box<dyn Island>;
}
