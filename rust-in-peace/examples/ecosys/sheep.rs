use crate::ws_utils;
use std::collections::HashMap;
use std::ops::Range;
use uuid::Uuid;

type Position = (i64, i64);

pub struct Sheep {
    pub id: Vec<Uuid>,
    pub energy: HashMap<Uuid, i64>,
    pub position: HashMap<Uuid, Position>,
}
impl Sheep {
    pub fn new(init_num: u32, init_energy: i64) -> Self {
        let mut id = vec![];
        let mut energy = HashMap::new();
        let position = HashMap::new();

        for _i in 0..init_num {
            let new_sheep = Uuid::new_v4();
            id.push(new_sheep);
            energy.insert(new_sheep, init_energy);
        }
        Self {
            id,
            energy,
            position,
        }
    }

    pub fn add_sheep(&mut self, id: Uuid, energy: i64, position: Position) {
        self.id.push(id);
        self.energy.insert(id, energy);
        self.position.insert(id, position);
    }

    pub fn remove_sheep(&mut self, id: &Uuid) {
        self.id.retain(|s| s != id);
        self.position.remove(id);
        self.energy.remove(id);
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

#[cfg(test)]
mod tests {
    use super::Sheep;
    use uuid::Uuid;

    #[test]
    fn test_add_remove_sheep() {
        let mut sheep = Sheep::new(0, 10);

        assert_eq!(sheep.id.len(), 0);
        assert_eq!(sheep.energy.len(), 0);
        assert_eq!(sheep.position.len(), 0);

        let id = Uuid::new_v4();
        sheep.add_sheep(id, 10, (1, 1));
        assert_eq!(sheep.id.len(), 1);
        assert_eq!(sheep.energy.len(), 1);
        assert_eq!(sheep.position.len(), 1);

        sheep.remove_sheep(&id);
        assert_eq!(sheep.id.len(), 0);
        assert_eq!(sheep.energy.len(), 0);
        assert_eq!(sheep.position.len(), 0);
    }
}
