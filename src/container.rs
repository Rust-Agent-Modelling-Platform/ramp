use std::fmt;
use std::collections::HashMap;
use uuid::Uuid;
use rand::{thread_rng, Rng};

use crate::agent::Agent;
use crate::action::Action;
use crate::functions;

pub struct Container {
    pub id: Uuid,
    pub id_agent_map: HashMap<Uuid, Agent>,
    pub max_agent_num: usize,
    pub turn_number: u64,
    pub action_queue: Vec<Action>,
    pub dim: i32,
    pub interval: (f64, f64),

    pub dead_ids: Vec<Uuid>,
    pub meeting_ids: Vec<Uuid>,
    pub procreating_ids: Vec<(Uuid, f64)>,
    pub migrating_ids: Vec<Uuid>,
}

impl Container {
    pub fn create(calculate_fitness: & dyn Fn(&Vec<f64>) -> f64, agents_number: i32, dim: i32, interval: (f64, f64), max_agent_num: usize) -> Container {
        let mut id_agent_map: HashMap<Uuid, Agent> = HashMap::with_capacity(max_agent_num);

        for i in 0..agents_number {
            let genotype: Vec<f64> = (0..dim).map(|_| {
                thread_rng().gen_range(interval.0, interval.1)
            }).collect();
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

            dead_ids: Vec::new(),
            meeting_ids: Vec::new(),
            procreating_ids: Vec::new(),
            migrating_ids: Vec::new(),
        }
    }

    pub fn create_action_queues(&mut self) {
        for (id, agent) in &self.id_agent_map {
            let action = agent.get_action();
            match action {
                Action::Death(id) => self.dead_ids.push(id),
                Action::Meeting(id, _) => self.meeting_ids.push(id),
                Action::Procreation(id, _) => self.procreating_ids.push((id, agent.fitness)),
                Action::Migration(id) => self.migrating_ids.push(id),
            }
        }
    }

    pub fn resolve_procreation(&mut self) {
        println! {"Number of agents that want to procreate this turn: {}", self.procreating_ids.len()}
        if self.procreating_ids.len() % 2 != 0 { println! {"There is an agent without a pair - gets the None action"} }

        if self.procreating_ids.len() == 0 { return }

        // no pair - just remove him at this moment
        if self.procreating_ids.len() % 2 == 1 {
            self.procreating_ids.pop().unwrap();
        }

        //sort the vector by fitness and procreate
        //we want the agents to mate in order of fitness
        //docs claim implementation of sort_by is O(n log(n)) worst-case
        //sorted from lowest to highest fitness, reverse a and b below to get opposite ordering
        self.procreating_ids.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        println!(" SORTED =============== {:?}", self.procreating_ids);

        //check hashmap capacity to see how many new agents can be created
        let mut free_places_in_map = self.max_agent_num - self.id_agent_map.keys().len();
        let mut places_required = self.procreating_ids.len() / 2;
        println!("Free: {}, needed: {}", free_places_in_map, places_required);

        //procreate as many as you can
        while free_places_in_map > 0  && places_required != 0 {
            let id1 = self.procreating_ids.pop().unwrap();
            let id2 = self.procreating_ids.pop().unwrap();
            self.procreate(id1.0, id2.0);
            free_places_in_map-=1;
            places_required-=1;
        }

        //add rest to meeting_ids
        //TODO: optimize with vec::append method
        for (id, fitness) in &self.procreating_ids {
            self.meeting_ids.push(*id);
        }
        //println!("Procreation queue at end of resolve_procreation (should be on meeting list): {:?}", self.procreating_ids);
        //println!("Meeting queue at end of resolve_procreation: {:?}", self.meeting_ids);

        self.procreating_ids.clear();
    }

    pub fn resolve_meetings(&mut self) {
        println! {"Number of agents that want a meeting this turn: {}", self.meeting_ids.len()}
        if self.meeting_ids.len() % 2 != 0 { println! {"There is an agent without a pair - gets the None action"} }
        if self.meeting_ids.len() == 0 { return }

        // no pair - just remove him at this moment
        if self.meeting_ids.len() % 2 == 1 {
           self.meeting_ids.pop();
        }
        while self.meeting_ids.len() != 0 {
            let id1 = self.meeting_ids.pop().unwrap();
            let id2 = self.meeting_ids.pop().unwrap();
            self.meet(id1, id2);
        }
    }

    pub fn clear_action_queues(&mut self) {
        self.action_queue.clear();
    }

    pub fn remove_dead_agents(&mut self) {
        println!("{:?}", self.dead_ids);
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

    // ================================================ Private methods ====================================================
    fn meet(&mut self, id1: Uuid, id2: Uuid) {
        if self.id_agent_map.get_mut(&id1).unwrap().fitness > self.id_agent_map.get_mut(&id2).unwrap().fitness  {
            self.id_agent_map.get_mut(&id1).unwrap().energy+=40;
            self.id_agent_map.get_mut(&id2).unwrap().energy-=40;
        } else {
            self.id_agent_map.get_mut(&id2).unwrap().energy+=40;
            self.id_agent_map.get_mut(&id1).unwrap().energy-=40;
        }
    }
    
    fn procreate(&mut self, id1: Uuid, id2: Uuid) {
        self.id_agent_map.get_mut(&id1).unwrap().energy-=10;
        self.id_agent_map.get_mut(&id2).unwrap().energy-=10;

        let mut new_genotype = Agent::crossover(&self.id_agent_map[&id1].genotype, &self.id_agent_map[&id2].genotype);
        Agent::mutate_genotype(&mut new_genotype, self.interval);
        let uuid = Uuid::new_v4();
        let new_agent = Agent::new(uuid, new_genotype , &functions::rastrigin);
        // println!("NEW AGENT {}", new_agent);

        self.id_agent_map.insert(uuid, new_agent);
    }

    // =============================================== Public utility methods =========================================================
    pub fn print_action_queue(&self) {
        for action in &self.action_queue {
            println!("{}", action)
        }
        println!("Nr of entries in this queue: {}", self.action_queue.len());
    }

    pub fn print_agent_stats(&self) {
        for (id, agent) in &self.id_agent_map {
            println!("Agent {}: Fitness - {}, energy - {}", &agent.id.to_string()[..5], agent.fitness, agent.energy)
        }
    }

    pub fn print_agent_count(&self) {
        println!("{}", self.id_agent_map.len());
    }

}


    // =============================================== Trait implementations ===========================================================
impl fmt::Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Container {{\n id: {},\n agents{:#?}\n}}", self.id, self.id_agent_map)
    }
}
