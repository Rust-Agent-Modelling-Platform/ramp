
use uuid::Uuid;
use rust_in_peace::message::Message;
use rust_in_peace::island::{Island, IslandEnv};
use rust_in_peace::map::{MapInstance};
use std::collections::HashMap;
use std::sync::Arc;
use std::cell::RefCell;
use crate::settings::{AgentSettings, SheepSettings, WolfSettings};
use crate::sheep::Sheep;
use crate::wolves::Wolves;

pub struct WSIsland {
    pub id: Uuid,
    island_env: IslandEnv,
    map: Option<MapInstance>,
    pub sheep_settings: Arc<SheepSettings>,
    pub wolf_settings: Arc<WolfSettings>,
    pub sheep: Sheep,
    pub wolves: Wolves,
}
impl WSIsland {
    pub fn new(id: Uuid,
               island_env: IslandEnv,
               sheep_settings: Arc<SheepSettings>,
               wolf_settings: Arc<WolfSettings>) -> Self {
        Self {
            id,
            island_env,
            map: None,
            sheep_settings,
            wolf_settings,
            sheep: Sheep::new(sheep_settings),
            wolves: Wolves::new(wolf_settings)
        }
    }




}



impl Island for WSIsland {
    fn on_start(&mut self) {
        let map = MapInstance::get_instance(&self.island_env);
        log::warn!("{:#?}", map);
        self.map = Some(map);

        self.sheep.set_initial_positions(&map, self.id);


    }

    fn do_turn(&mut self, turn_number: u32, messages: Vec<Message>) {
        //self.map.as_mut().unwrap().set(&mut self.island_env, 9, 8, 1);
    }

    fn on_finish(&mut self) {

    }
}

