use uuid::Uuid;
use rust_in_peace::message::Message;
use rust_in_peace::island::{Island, IslandEnv};
use rust_in_peace::map::{MapInstance};

pub struct MapIsland {
    id: Uuid,
    island_env: IslandEnv,
    map: Option<MapInstance>,
}

impl Island for MapIsland {
    fn on_start(&mut self) {
        let map = MapInstance::get_instance(&self.island_env);

        log::warn!("{:#?}", map);

        self.map = Some(map);
    }

    fn do_turn(&mut self, turn_number: u32, messages: Vec<Message>) {
        // here user has to receive MapSet msg and insert received value into map
        for msg in messages {
            match msg {
                Message::MapSet(x, y, val) => {
                    self.map.as_mut().map(|map| map.set(&mut self.island_env, x, y, val))
                }
                _ => println!("Other msg")
            }
        }
    }

    fn on_finish(&mut self) {

    }
}

impl MapIsland {
    pub fn new(id: Uuid, island_env: IslandEnv) -> Self {
        Self {
            id, 
            island_env,
            map: None,
        }
    }
}