
use uuid::Uuid;
use std::collections::HashMap;
use crate::settings::SheepSettings;
use std::sync::Arc;
use std::ops::Range;
use std::intrinsics::unaligned_volatile_load;
use std::borrow::BorrowMut;
use rust_in_peace::map::MapInstance;

type Position = (u64, u64);

pub struct Sheep {
    ids: Vec<Uuid>,
    energy: HashMap<Uuid, u64>,
    position: HashMap<Uuid, Position>,
    reproduction_chance: f64,
    energy_gain: u64,
    energy_loss: u64
}
impl Sheep {
    pub fn new(settings: Arc<SheepSettings>) -> Self {
        let mut ids = vec![];
        let mut energy = HashMap::new();
        let mut position = HashMap::new();

        for i in 0..settings.init_num {
            let new_sheep = Uuid::new_v4();
            ids.push(new_sheep);
            energy.insert(new_sheep, settings.init_energy);
        }
        Self {
            ids,
            energy,
            position,
            reproduction_chance: settings.reproduction_chance,
            energy_gain: settings.energy_gain,
            energy_loss: settings.energy_loss
        }
    }


    fn serialize(&self, id: Uuid){
        unimplemented!();
    }
    fn is_reproducing(&self, id: Uuid){
        unimplemented!();
    }
    pub fn set_initial_positions(&mut self, map_instance: &MapInstance, island_id: Uuid) {
        let range = map_instance.map.owners.values().find(|&r| r.contains(&island_id)).unwrap();

        unimplemented!();
    }





}