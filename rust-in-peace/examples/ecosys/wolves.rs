
use uuid::Uuid;
use std::collections::HashMap;
use crate::settings::WolfSettings;
use std::sync::Arc;

type Position = (u64, u64);

pub struct Wolves {
    energy: HashMap<Uuid, u64>,
    position: HashMap<Uuid, Position>,
    reproduction_chance: f64,
    energy_gain: u64,
    energy_loss: u64
}
impl Wolves {
    pub fn new(settings: Arc<WolfSettings>) -> Self {

    }


    fn serialize(&self, id: Uuid){
        unimplemented!();
    }
    fn is_reproducing(&self, id: Uuid){
        unimplemented!();
    }


}