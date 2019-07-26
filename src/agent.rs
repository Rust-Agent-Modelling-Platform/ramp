use std::fmt;
use uuid::Uuid;

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
            genotype: genotype,
        }
    }
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Agent {{\n id: {},\n energy: {},\n genotype: {:#?},\n fitness: {}\n}}", 
            self.id, self.energy, self.genotype, self.fitness)
    }
}