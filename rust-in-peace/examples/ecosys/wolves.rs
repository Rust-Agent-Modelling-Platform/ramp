
use uuid::Uuid;
use std::collections::HashMap;
use crate::settings::WolfSettings;
use std::sync::Arc;
use rust_in_peace::map::MapInstance;
use rand::Rng;
use std::ops::Range;
use crate::{utils, ws_utils};
use crate::agent_types::AgentType;
use crate::ws_utils::SerializedAgent;

type Position = (u64, u64);

pub struct Wolves {
    id: Vec<Uuid>,
    energy: HashMap<Uuid, u64>,
    position: HashMap<Uuid, Position>,
    reproduction_chance: f64,
    energy_gain: u64,
    energy_loss: u64
}
impl Wolves {
    pub fn new(settings: Arc<WolfSettings>) -> Self {
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

    pub fn serialize(&self, agent_type: AgentType, id: Uuid) -> Vec<u8> {
        let agent = SerializedAgent {
            agent_type,
            id,
            energy: *self.energy.get(&id).unwrap(),
            position: *self.position.get(&id).unwrap()
        };
        bincode::serialize(&agent).unwrap()
    }

    pub fn add_new_wolf(&mut self, id: Uuid, energy: u64, position: Position) {
        self.id.push(id);
        self.energy.insert(id, energy);
        self.position.insert(id, position);
    }

    pub fn set_initial_wolf_positions(&mut self, range: Range<u64>) {
        for id in self.id.iter() {
            let (x, y) = ws_utils::generate_random_position(&range);
            self.position.insert(*id, (x,y));
        }
    }

}