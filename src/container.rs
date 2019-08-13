use colored::*;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::fmt;
use std::time::Instant;
use uuid::Uuid;

use crate::action::Action;
use crate::agent::Agent;
use crate::functions;

pub struct Container {
    pub id: Uuid,
    pub id_agent_map: HashMap<Uuid, Agent>,
    pub max_agent_num: usize,
    pub turn_number: u64,
    pub action_queue: Vec<Action>,
    pub dim: i32,
    pub interval: (f64, f64),
    turns: u32,

    pub dead_ids: Vec<Uuid>,
    pub meeting_ids: Vec<(Uuid, f64)>,
    pub procreating_ids: Vec<(Uuid, f64)>,
    pub migrating_ids: Vec<Uuid>,
}

impl Container {
    pub fn new(
        calculate_fitness: &dyn Fn(&[f64]) -> f64,
        agents_number: u32,
        dim: i32,
        interval: (f64, f64),
        max_agent_num: usize,
        turns: u32,
    ) -> Self {
        let mut id_agent_map: HashMap<Uuid, Agent> = HashMap::with_capacity(max_agent_num);

        for _i in 0..agents_number {
            let genotype: Vec<f64> = (0..dim)
                .map(|_| thread_rng().gen_range(interval.0, interval.1))
                .collect();
            let id = Uuid::new_v4();
            id_agent_map.insert(id, Agent::new(id, genotype, calculate_fitness));
        }
        Container {
            id: Uuid::new_v4(),
            id_agent_map,
            max_agent_num,
            turn_number: 0,
            action_queue: Vec::new(),
            dim,
            interval,
            turns,

            dead_ids: Vec::new(),
            meeting_ids: Vec::new(),
            procreating_ids: Vec::new(),
            migrating_ids: Vec::new(),
        }
    }

    pub fn create_action_queues(&mut self) {
        for agent in self.id_agent_map.values() {
            let action = agent.get_action();
            match action {
                Action::Death(id) => self.dead_ids.push(id),
                Action::Meeting(id, _) => self.meeting_ids.push((id, agent.fitness)),
                Action::Procreation(id, _) => self.procreating_ids.push((id, agent.fitness)),
                Action::Migration(id) => self.migrating_ids.push(id),
            }
        }
    }

    pub fn resolve_procreation(&mut self) {
        // log::info! {"Number of agents that want to procreate this turn: {}", self.procreating_ids.len()}

        if self.procreating_ids.is_empty() {
            return;
        }

        // no pair - just remove him at this moment
        if self.procreating_ids.len() % 2 == 1 {
            self.procreating_ids.pop().unwrap();
        }

        //sort the vector by fitness and procreate
        //we want the agents to mate in order of fitness
        //docs claim implementation of sort_by is O(n log(n)) worst-case
        //sorted from lowest to highest fitness, reverse a and b below to get opposite ordering
        self.procreating_ids
            .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        //log::info!(" SORTED =============== {:?}", self.procreating_ids);

        //check hashmap capacity to see how many new agents can be created
        let mut free_places_in_map = self.max_agent_num - self.id_agent_map.keys().len();
        let mut places_required = self.procreating_ids.len() / 2;
        // log::info!("Free: {}, needed: {}", free_places_in_map, places_required);

        //procreate as many as you can
        while free_places_in_map > 0 && places_required != 0 {
            let id1 = self.procreating_ids.pop().unwrap();
            let id2 = self.procreating_ids.pop().unwrap();
            self.procreate(id1.0, id2.0);
            free_places_in_map -= 1;
            places_required -= 1;
        }

        //add rest to meeting_ids
        //TODO: optimize with vec::append method
        for (id, fitness) in &self.procreating_ids {
            self.meeting_ids.push((*id, *fitness));
        }

        self.procreating_ids.clear();
    }

    pub fn resolve_meetings(&mut self) {
        // log::info! {"Number of agents that want a meeting this turn: {}", self.meeting_ids.len()}
        if self.meeting_ids.is_empty() {
            return;
        }

        //self.meeting_ids.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // no pair - just remove him at this moment
        if self.meeting_ids.len() % 2 == 1 {
            self.meeting_ids.pop();
        }
        while !self.meeting_ids.is_empty() {
            let (id1, _) = self.meeting_ids.pop().unwrap();
            let (id2, _) = self.meeting_ids.pop().unwrap();
            self.meet(id1, id2);
        }
    }

    pub fn clear_action_queues(&mut self) {
        self.action_queue.clear();
    }

    pub fn remove_dead_agents(&mut self) {
        for id in &self.dead_ids {
            self.id_agent_map.remove(id);
        }
        self.dead_ids.clear();
    }

    pub fn remove_migrants(&mut self) {
        for id in &self.migrating_ids {
            self.id_agent_map.remove(id);
        }
    }

    pub fn run(&mut self) {
        let now = Instant::now();
        for _turn_number in 0..=self.turns {
            self.remove_migrants();
            self.create_action_queues();
            self.resolve_procreation();
            self.resolve_meetings();
            self.remove_dead_agents();
            self.clear_action_queues();
        }
        log::info!("{}", "================= END =================".green());
        log::info!("Time elapsed: {} seconds", now.elapsed().as_secs());
        log::info!("At end of simulation the best agent is:");
        self.print_most_fit_agent();
    }

    // ================================================ Private methods ====================================================
    fn meet(&mut self, id1: Uuid, id2: Uuid) {
        if self.id_agent_map.get_mut(&id1).unwrap().fitness
            > self.id_agent_map.get_mut(&id2).unwrap().fitness
        {
            self.id_agent_map.get_mut(&id1).unwrap().energy += 40;
            self.id_agent_map.get_mut(&id2).unwrap().energy -= 40;
        } else {
            self.id_agent_map.get_mut(&id2).unwrap().energy += 40;
            self.id_agent_map.get_mut(&id1).unwrap().energy -= 40;
        }
    }

    fn procreate(&mut self, id1: Uuid, id2: Uuid) {
        self.id_agent_map.get_mut(&id1).unwrap().energy -= 10;
        self.id_agent_map.get_mut(&id2).unwrap().energy -= 10;

        let mut new_genotype = Agent::crossover(
            &self.id_agent_map[&id1].genotype,
            &self.id_agent_map[&id2].genotype,
        );
        Agent::mutate_genotype(&mut new_genotype, self.interval);
        let uuid = Uuid::new_v4();
        let new_agent = Agent::new(uuid, new_genotype, &functions::rastrigin);

        self.id_agent_map.insert(uuid, new_agent);
    }

    // =============================================== Public utility methods =========================================================
    // These print functions will be used in [#27]
    // pub fn print_action_queue(&self) {
    //     for action in &self.action_queue {
    //         log::info!("{}", action)
    //     }
    //     log::info!("Nr of entries in this queue: {}", self.action_queue.len());
    // }

    // pub fn print_agent_stats(&self) {
    //     for agent in self.id_agent_map.values() {
    //         log::info!(
    //             "Agent {}: Fitness - {}, energy - {}",
    //             &agent.id.to_string()[..5],
    //             agent.fitness,
    //             agent.energy
    //         )
    //     }
    // }

    // pub fn print_agent_count(&self) {
    //     log::info!("{}", self.id_agent_map.len());
    // }

    pub fn print_most_fit_agent(&self) {
        let tg = Agent::new_dummy();
        let mut top_guy = &tg;
        for agent in self.id_agent_map.values() {
            if agent.fitness > top_guy.fitness {
                top_guy = agent;
            }
        }
        log::info!("{}", top_guy);
    }
}

// =============================================== Trait implementations ===========================================================
impl fmt::Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Container {{\n id: {},\n agents{:#?}\n}}",
            self.id, self.id_agent_map
        )
    }
}
