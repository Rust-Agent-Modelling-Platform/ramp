use std::fmt;
use uuid::Uuid;
use rand::{thread_rng, Rng};

use crate::action::Action;

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

    pub fn mutate_genotype(genotype: &mut Vec<f64>, interval: (f64, f64)) {
        let gene1 = thread_rng().gen_range(0, genotype.len() - 1);
        let gene2 = thread_rng().gen_range(0, genotype.len() - 1);

        let delta1 = thread_rng().gen_range(-0.1, 0.1);
        let delta2 = thread_rng().gen_range(-0.1, 0.1);

        genotype[gene1]+=delta1;
        genotype[gene2]+=delta2;
    }

    pub fn crossover(genotype1: &Vec<f64>, genotype2: &Vec<f64>) -> Vec<f64>{
        let division_point = thread_rng().gen_range(0, genotype1.len());
        let mut new_genotype = vec![];
        let head = &genotype1[..division_point];
        let tail = &genotype2[division_point..];
        new_genotype.extend_from_slice(head);
        new_genotype.extend_from_slice(tail);
        new_genotype
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