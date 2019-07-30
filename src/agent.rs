use std::fmt;
use uuid::Uuid;
use crate::action::Action;
use rand::{thread_rng, Rng};

#[derive(Debug)]
pub struct Agent {
    pub id: Uuid,
    pub energy: i32,    
    pub genotype: Vec<f64>,
    pub fitness: f64,
}

impl Agent {
    pub fn new(id: Uuid, genotype: Vec<f64>, calculate_fitness: &Fn(&Vec<f64>) -> f64) -> Agent {
        Agent { 
            id,
            energy: 100,
            fitness: calculate_fitness(&genotype),
            genotype,
        }
    }

    pub fn get_action(&self) -> Action {
        let prob = thread_rng().gen_range(1, 100);
        if self.energy <= 0 {
            Action::Death(self.id)
        } else if prob == 1 {
            Action::Migration(self.id)
        } else if self.energy > 0 && self.energy < 90 {
            Action::Meeting(self.id, Uuid::nil())
        } else {
            if prob > 50 {
                Action::Procreation(self.id, Uuid::nil())
            } else {
                Action::Meeting(self.id, Uuid::nil())
            }
        }
    }
}


impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Agent {{\n id: {},\n energy: {},\n genotype: {:#?},\n fitness: {}\n}}", 
            self.id, self.energy, self.genotype, self.fitness)
    }
}