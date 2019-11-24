
use uuid::Uuid;
use std::collections::HashMap;
use crate::settings::SheepSettings;
use std::sync::Arc;
use std::ops::Range;
use std::borrow::BorrowMut;
use rust_in_peace::map::MapInstance;
use rand::Rng;
use crate::agent_types::AgentType;
use crate::ws_utils;
use crate::ws_utils::SerializedAgent;
use rust_in_peace::network::Ip;

type Position = (u64, u64);

pub struct Sheep {
    pub id: Vec<Uuid>,
    pub energy: HashMap<Uuid, u64>,
    pub position: HashMap<Uuid, Position>,
    pub reproduction_chance: f64,
    pub energy_gain: u64,
    pub energy_loss: u64
}
impl Sheep {
    pub fn new(settings: Arc<SheepSettings>) -> Self {
        let mut id = vec![];
        let mut energy = HashMap::new();
        let position = HashMap::new();

        for i in 0..settings.init_num {
            let new_sheep = Uuid::new_v4();
            id.push(new_sheep);
            energy.insert(new_sheep, settings.init_energy);
        }
        Self {
            id,
            energy,
            position,
            reproduction_chance: settings.reproduction_chance,
            energy_gain: settings.energy_gain,
            energy_loss: settings.energy_loss
        }
    }

    fn is_reproducing(&self, id: Uuid) {
        unimplemented!();
    }

    pub fn serialize(&self, agent_type: AgentType, id: Uuid) -> Vec<u8> {
        let agent = SerializedAgent {
            agent_type,
            id,
            energy: *self.energy.get(&id).unwrap(),
            position: *self.position.get(&id).unwrap()
        };
        bincode::serialize(&agent).unwrap()
    }

    pub fn add_new_sheep(&mut self, id: Uuid, energy: u64, position: Position) {
        self.id.push(id);
        self.energy.insert(id, energy);
        self.position.insert(id, position);
    }

    pub fn set_initial_sheep_positions(&mut self, range: Range<u64>) {
        for id in self.id.iter() {
            let (x, y) = ws_utils::generate_random_position(&range);
            self.position.insert(*id, (x,y));
        }
    }
}