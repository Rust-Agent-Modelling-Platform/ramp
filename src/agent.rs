use rand::{thread_rng, Rng};
use std::f64;
use std::fmt;
use uuid::Uuid;

use crate::constants;
use crate::action::Action;

#[derive(Debug, Clone)]
pub struct Agent {
    pub id: Uuid,
    pub energy: i32,
    pub genotype: Vec<f64>,
    pub fitness: f64,
}

impl Agent {
    pub fn new(id: Uuid, genotype: Vec<f64>, calculate_fitness: &dyn Fn(&[f64]) -> f64) -> Agent {
        Agent {
            id,
            energy,
            fitness: -calculate_fitness(&genotype),
            genotype,
        }
    }

    pub fn mutate_genotype(genotype: &mut Vec<f64>, interval: (f64, f64)) {
        let left_bound = interval.0 / 10.0; // -0.512 rastrigin
        let right_bound = interval.1 / 10.0; //  0.512 rastrigin

        for gene in genotype.iter_mut() {
            if thread_rng().gen_range(0.0, 1.0) <= constants::MUTATION_RATE {
                *gene += thread_rng().gen_range(left_bound, right_bound);
                //return;
            }
        }
    }

    pub fn crossover(genotype1: &[f64], genotype2: &[f64]) -> Vec<f64> {
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
        } else if prob > constants::PROCREATION_PROB {
            Action::Procreation(self.id, Uuid::nil())
        } else {
            Action::Meeting(self.id, Uuid::nil())
        }
    }
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Agent {{\n id: {},\n energy: {},\n genotype: {:#?},\n fitness: {}\n}}",
            self.id, self.energy, self.genotype, self.fitness
        )
    }
}
