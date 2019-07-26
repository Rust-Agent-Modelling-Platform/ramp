use std::fmt;
use uuid::Uuid;
use rand::{thread_rng, Rng};

use crate::agent::Agent;


pub struct Container {
    pub id: Uuid,
    pub agents: Vec<Agent>
}

impl Container {

    pub fn create(calculate_fitness: &Fn(&Vec<f64>) -> f64, agents_number: i32, dim: i32, interval: (f64, f64)) -> Container {
        let agents: Vec<Agent> = (0..agents_number).map(|_| {
            let genotype: Vec<f64> = (0..dim).map(|_| {
                thread_rng().gen_range(interval.0, interval.1)
            }).collect();
            Agent::create(genotype, calculate_fitness)
        }).collect();

        Container {
            id: Uuid::new_v4(),
            agents: agents,
        }
    } 
}

impl fmt::Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Container {{\n id: {},\n agents{:#?}\n}}", self.id, self.agents)
    }
}