use rand::{thread_rng, Rng};
use std::f64;
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

use crate::action::Action;
use crate::functions;
use crate::settings::AgentSettings;
use std::cell::RefCell;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Agent {
    pub id: Uuid,
    pub settings: Arc<AgentSettings>,
    pub energy: i32,
    pub genotype: Vec<f64>,
    pub fitness: f64,
}

impl Agent {
    pub fn new(
        id: Uuid,
        config: Arc<AgentSettings>,
        genotype: Vec<f64>,
        calculate_fitness: &dyn Fn(&[f64]) -> f64,
        energy: i32,
    ) -> RefCell<Agent> {
        let function = if config.minimum {
            -calculate_fitness(&genotype)
        } else {
            calculate_fitness(&genotype)
        };
        RefCell::new(Agent {
            id,
            settings: config,
            energy,
            fitness: function,
            genotype,
        })
    }

    pub fn procreate(&mut self, partner: &mut Agent) -> (Uuid, RefCell<Agent>) {
        let penalty = self.settings.procreation_penalty;

        self.energy = (f64::from(self.energy) * (1.0 - penalty)) as i32;
        partner.energy = (f64::from(partner.energy) * (1.0 - penalty)) as i32;

        let child_energy = self.energy + partner.energy;

        let mut new_genotype = Agent::crossover(&self.genotype, &partner.genotype);
        Agent::mutate_genotype(&self.settings, &mut new_genotype);

        let uuid = Uuid::new_v4();
        let new_agent = Agent::new(
            uuid,
            self.settings.clone(),
            new_genotype,
            &functions::rastrigin,
            child_energy,
        );
        (uuid, new_agent)
    }

    pub fn meet(&mut self, partner: &mut Agent) {
        let penalty = &self.settings.meeting_penalty;

        if self.fitness > partner.fitness {
            self.energy += *penalty;
            partner.energy -= *penalty;
        } else {
            partner.energy += *penalty;
            self.energy -= *penalty;
        }
    }

    pub fn mutate_genotype(config: &AgentSettings, genotype: &mut Vec<f64>) {
        let left_bound = config.lower_bound / 10.0;
        let right_bound = config.upper_bound / 10.0;

        for gene in genotype.iter_mut() {
            if thread_rng().gen_range(0.0, 1.0) <= config.mutation_rate {
                *gene += thread_rng().gen_range(left_bound, right_bound);
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
        } else if prob > self.settings.procreation_prob {
            Action::Procreation(self.id, Uuid::nil())
        } else {
            Action::Meeting(self.id, Uuid::nil())
        }
    }

    pub fn as_string(&self) -> String {
        format!(
            "Agent {{\n id: {},\n energy: {},\n genotype: {:#?},\n fitness: {}\n}}",
            self.id, self.energy, self.genotype, self.fitness
        )
    }
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}
