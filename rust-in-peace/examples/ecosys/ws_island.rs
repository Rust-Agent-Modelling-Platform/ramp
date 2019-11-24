
use uuid::Uuid;
use rust_in_peace::message::Message;
use rust_in_peace::island::{Island, IslandEnv};
use rust_in_peace::map::{MapInstance, Fragment};
use std::collections::HashMap;
use std::sync::Arc;
use std::cell::RefCell;
use crate::settings::{SheepSettings, WolfSettings, IslandSettings};
use crate::sheep::Sheep;
use crate::wolves::Wolves;
use crate::{ws_utils};
use crate::agent_types::AgentType;
use std::borrow::Borrow;
use std::ops::Range;

pub struct WSIsland {
    pub id: Uuid,
    island_env: IslandEnv,
    map: Option<MapInstance>,
    pub island_settings: Arc<IslandSettings>,
    pub sheep_settings: Arc<SheepSettings>,
    pub wolf_settings: Arc<WolfSettings>,
    pub sheep: Sheep,
    pub wolves: Wolves,
}
impl WSIsland {
    pub fn new(id: Uuid,
               island_env: IslandEnv,
               island_settings: Arc<IslandSettings>,
               sheep_settings: Arc<SheepSettings>,
               wolf_settings: Arc<WolfSettings>) -> Self {
        Self {
            id,
            island_env,
            map: None,
            island_settings: island_settings.clone(),
            sheep_settings: sheep_settings.clone(),
            wolf_settings: wolf_settings.clone(),
            sheep: Sheep::new(sheep_settings.clone()),
            wolves: Wolves::new(wolf_settings.clone())
        }
    }

    fn resolve_messages(&mut self, messages: Vec<Message>) {
        let mut migrants_num = 0;
        for message in messages {
            match message {
                Message::Agent(migrant) => {
                    migrants_num += 1;
                    let (agent_type, id, energy, position) = ws_utils::deserialize(migrant);
                    match agent_type {
                        AgentType::Sheep => self.sheep.add_new_sheep(id, energy, position),
                        AgentType::Wolf => self.wolves.add_new_wolf(id, energy, position)
                    }
                }
                _ => log::error!("Unexpected msg"),
            }
        }
    }

    fn update_grass(&mut self) {
        for val in self.map.as_mut().unwrap().data.iter_mut() {
            if *val == -1 {
                *val=self.island_settings.grass_interval;
            }
            else if *val == 0 {
                continue;
            }
            else if 0 < *val && *val <= self.island_settings.grass_interval {
                *val=*val-1;
            }
        }
    }

    fn do_sheep_turn(&mut self) {
        for sheep in &self.sheep.id {
            //1. Check if grass is here. If so - eat it, update energy, update grass.
            //2. Get random move direction. Move or if boundary - add to outgoing/local migrant queue
            //3. Decrease energy. if <= 0 -> remove this sheep
            //4. Check for reproduction if alive. If reproduction -> make new sheep, add it to island
        }
    }

    fn do_wolf_turn(&mut self) {
        unimplemented!();
        //1. Check if sheep is here. If so - remove the first found sheep, update
        // energy.
        //2. Get random move direction. Move or if boundary - add to outgoing/local migrant queue
        //3. Decrease energy. If <= 0 -> remove this wolf
        //4. Check for procreation if alive. If reproduction -> add new wolf, add it to island.
    }

    fn send_migrants(&mut self) {
        unimplemented!();
    }





}
impl Island for WSIsland {
    fn on_start(&mut self) {
        let map = MapInstance::get_instance(&self.island_env);
        log::warn!("{:#?}", map);
        self.map = Some(map);

        //Set the initial amount of wolves and sheep at random parts of the map
        let range = self.map.as_ref().unwrap().get_my_range();
        self.sheep.set_initial_sheep_positions(range.clone());
        self.wolves.set_initial_wolf_positions(range.clone());

        //Initially whole map has grass (0 means grass)
        self.map.as_mut().unwrap().init_with_val(0);
    }

    fn do_turn(&mut self, turn_number: u32, messages: Vec<Message>) {
        println!("DOING TURN {} ON ISLAND {}", turn_number, &self.id);
//        self.resolve_messages(messages);
//        self.do_sheep_turn();
//        self.do_wolf_turn();
//        self.update_grass();
//        self.send_migrants();
    }

    fn on_finish(&mut self) {
        let duration = self.island_env.start_time.elapsed().as_secs();
    }
}

