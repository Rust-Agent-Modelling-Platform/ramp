use crate::settings::SheepSettings;
use crate::ws_utils;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;
use uuid::Uuid;

type Position = (i64, i64);

pub struct Sheep {
    pub id: Vec<Uuid>,
    pub energy: HashMap<Uuid, i64>,
    pub position: HashMap<Uuid, Position>,
    pub reproduction_chance: f64,
    pub energy_gain: i64,
    pub energy_loss: i64,
}
impl Sheep {
    pub fn new(settings: Arc<SheepSettings>) -> Self {
        let mut id = vec![];
        let mut energy = HashMap::new();
        let position = HashMap::new();

        for _i in 0..settings.init_num {
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
            energy_loss: settings.energy_loss,
        }
    }

    pub fn add_new_sheep(&mut self, id: Uuid, energy: i64, position: Position) {
        self.id.push(id);
        self.energy.insert(id, energy);
        self.position.insert(id, position);
    }

    pub fn set_initial_sheep_positions(&mut self, range: Range<u64>, chunk_len: i64) {
        for id in self.id.iter() {
            let (x, y) = ws_utils::generate_random_position(&range, chunk_len);
            self.position.insert(*id, (x, y));
        }
    }

    pub fn print_sheep(&self, id: &Uuid) {
        println!(
            "<--------------- Sheep {} -------------------->",
            &id.to_string()[..8]
        );
        println!("Energy: {:?}", self.energy.get(id).unwrap());
        println!("Position: {:?}", self.position.get(id).unwrap());
        println!("<---------------------------------------------------->");
    }
}
