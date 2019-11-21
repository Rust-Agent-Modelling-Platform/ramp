#[macro_use]
extern crate serde_derive;

use rust_in_peace::island::{Island, IslandEnv, IslandFactory};
use rust_in_peace::simulation::Simulation;
use rust_in_peace::utils;
use std::sync::Arc;
use uuid::Uuid;

use crate::map_island::MapIsland;

mod map_island;
mod map_agent;

struct MapIslandFactory;

impl IslandFactory for MapIslandFactory {
    fn create(&self, id: Uuid, island_env: IslandEnv) -> Box<dyn Island> {
        let island = MapIsland::new(id, island_env);
        Box::new(island)
    }
}

fn main() {
    let factory = MapIslandFactory {};
    Simulation::start_simulation(Box::new(factory));
}
