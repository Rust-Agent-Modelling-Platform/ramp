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

    pub fn create(genotype: Vec<f64>, calculate_fitness: &Fn(&Vec<f64>) -> f64) -> Agent {
        Agent { 
            id: Uuid::new_v4(),
            energy: 100,
            fitness: calculate_fitness(&genotype),
            genotype,
        }
    }

    pub fn get_action(&self) -> Action {
        let prob = thread_rng().gen_range(0.0, 1.0);
        match self.energy {
            x if x <= 0 => Action::Death(self.id),
            x if (x > 0) && (x < 90) => Action::Meeting(self.id, self.id),
            x if x >= 90 =>
                if prob > 0.5 {
                    Action::Meeting(self.id, self.id)
                } else {
                    Action::Procreation(self.id, self.id)
                }
            _ => Action::Migration(self.id)
        }
    }

}


impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Agent {{\n id: {},\n energy: {},\n genotype: {:#?},\n fitness: {}\n}}", 
            self.id, self.energy, self.genotype, self.fitness)
    }
}