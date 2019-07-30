use std::fmt;
use uuid::Uuid;
use rand::{thread_rng, Rng};

use crate::agent::Agent;
use crate::action::Action;

use crate::functions;
use std::collections::HashMap;

pub struct Container {
    pub id: Uuid,
    pub agents: HashMap<Uuid, Agent>,
    pub turn_number: u64,
    pub action_queue: Vec<Action>,
    pub dim: i32,
    pub interval: (f64, f64),

    pub dead_ids: Vec<Uuid>,
    pub meeting_ids: Vec<Uuid>,
    pub procreating_ids: Vec<Uuid>,
    pub migrating_ids: Vec<Uuid>,
    pub none_ids: Vec<Uuid>
}
impl Container {
    pub fn create(calculate_fitness: & dyn Fn(&Vec<f64>) -> f64, agents_number: i32, dim: i32, interval: (f64, f64)) -> Container {
        let mut agents_hm = HashMap::new();

        for i in 0..agents_number {
            let genotype: Vec<f64> = (0..dim).map(|_| {
                thread_rng().gen_range(interval.0, interval.1)
            }).collect();
            let id = Uuid::new_v4();
            agents_hm.insert( id, Agent::new(id, genotype, &functions::rastrigin) );
        }
        Container {
            id: Uuid::new_v4(),
            agents: agents_hm,
            turn_number: 0,
            action_queue: Vec::new(),
            dim,
            interval,

            dead_ids: Vec::new(),
            meeting_ids: Vec::new(),
            procreating_ids: Vec::new(),
            migrating_ids: Vec::new(),
            none_ids: Vec::new(),
        }
    }


    pub fn create_action_queues(&mut self) {
        for (id, agent) in &self.agents {
            let action = agent.get_action();
            match action {
                Action::Death(id) => self.dead_ids.push(id),
                Action::Meeting(id, _) => self.meeting_ids.push(id),
                Action::Procreation(id, _) => self.procreating_ids.push(id),
                Action::Migration(id) => self.migrating_ids.push(id),
                Action::None(id) => self.none_ids.push(id),
            }
        }
    }


    pub fn resolve_meetings(&mut self) {
        //1. First resolve meetings
        //a. get those who want a meeting
        println! {"Number of agents that want a meeting this turn: {}", self.meeting_ids.len()}
        if self.meeting_ids.len() % 2 != 0 { println! {"There is an agent without a pair - gets the None action"} }

        //b. construct a meeting from them
        //if nobody wants to meet - great
        if self.meeting_ids.len() == 0 { return }

        //for now, just get anybody to meet with
        //first deal with uneven agent, set his action to be None(uuid)
        if self.meeting_ids.len() % 2 == 1 {
            let id = self.meeting_ids.pop().unwrap();
            self.none_ids.push(id);
        }

        //rest of dudes meet
        //add meeting action with different values to temp list
        if self.meeting_ids.len() % 2 != 1 {
            while self.meeting_ids.len() != 0 {
                let id1 = self.meeting_ids.pop().unwrap();
                let id2 = self.meeting_ids.pop().unwrap();

                let new_action = Action::Meeting(id1, id2);
                self.action_queue.push(new_action);
            }
        }
    }


    pub fn resolve_procreation(&mut self) {
        //2. Resolve procreation
        //a. get those who want to procreate
        println! {"Number of agents that want to procreate this turn: {}", self.procreating_ids.len()}
        if self.procreating_ids.len() % 2 != 0 { println! {"There is an agent without a pair - gets the None action"} }

        //b. construct a meeting from them
        //if nobody wants to meet - great
        if self.procreating_ids.len() == 0 { return }

        //for now, just get anybody to procreate with
        //first deal with uneven agent, set his action to be None(uuid)
        if self.procreating_ids.len() % 2 == 1 {
            let id = self.procreating_ids.pop().unwrap();
            self.none_ids.push(id);
        }

        //rest of dudes meet
        //add meeting action with different values to temp list
        if self.procreating_ids.len() % 2 != 1 {
            while self.procreating_ids.len() != 0 {
                let id1 = self.meeting_ids.pop().unwrap();
                let id2 = self.meeting_ids.pop().unwrap();

                let new_action = Action::Procreation(id1, id2);
                self.action_queue.push(new_action);
            }
        }
    }


    pub fn clear_action_queue(&mut self) {
        self.action_queue.clear();
    }


    pub fn execute_actions(&mut self) {
        self.execute_meetings_and_procreation();

    }


    pub fn remove_dead_agents(&mut self) {
        for id in &self.dead_ids {
                self.agents.remove(id);
        }
    }


    pub fn remove_migrants(&mut self) {
        for id in &self.migrating_ids {
            self.agents.remove(id);
        }
    }


    // ================================================ Private methods ====================================================
    fn remove_one_agent(&mut self, id: Uuid) {
        self.agents.remove(&id);
    }


    fn remove_agents(&mut self, ids: Vec<Uuid>) {
        for id in &ids {
            self.agents.remove(id);
        }
    }


    fn execute_meetings_and_procreation(&mut self) {
        for action in &self.action_queue {
            if let Action::Meeting(id1, id2) = action {
                if self.agents.get_mut(id1).unwrap().fitness < self.agents.get_mut(id2).unwrap().fitness  {
                    self.agents.get_mut(id1).unwrap().energy+=40;
                    self.agents.get_mut(id2).unwrap().energy-=40;
                } else {
                    self.agents.get_mut(id2).unwrap().energy+=40;
                    self.agents.get_mut(id1).unwrap().energy-=40;
                }
            }
            if let Action::Procreation(id1, id2) = action {
                //incur penalty for procreation
                self.agents.get_mut(id1).unwrap().energy-=10;
                self.agents.get_mut(id2).unwrap().energy-=10;

                //create new genotype
                let mut new_genotype = vec![];

                //crossover
                self.crossover(*id1, *id2, &mut new_genotype);

                //mutate the new genotype
                self.mutate_genotype(&mut new_genotype);

                let uuid = Uuid::new_v4();
                let new_agent = Agent::new(uuid, new_genotype , &functions::rastrigin);
                println!("NEW AGENT {}", new_agent);

                self.agents.insert(uuid, new_agent);
            }
        }
    }

    fn execute_procreation(&mut self) {
        for action in &self.action_queue {
        }
    }


    fn mutate_genotype(&self, genotype: &mut Vec<f64>) {
        let length = genotype.len();
        let gene = thread_rng().gen_range(0, genotype.len() - 1);
        //TODO: range should not be hardcoded
        genotype[gene] = thread_rng().gen_range(-5.12, 5.12);
    }


    fn crossover(&self, id1: Uuid, id2: Uuid, genotype: &mut Vec<f64>) {
        //TODO: ranges cannot be hardcoded
        let head = &self.agents[&id1].genotype[..1];
        let tail = &self.agents[&id2].genotype[1..];
        genotype.extend_from_slice(head);
        genotype.extend_from_slice(tail);
    }
//
//
//    // =============================================== Public utility methods =========================================================
    pub fn print_action_queue(&self) {
        for action in &self.action_queue {
            println!("{}", action)
        }
        println!("Nr of entries in this queue: {}", self.action_queue.len());
    }

    pub fn print_agent_stats(&self) {
        for (id, agent) in &self.agents {
            println!("Agent {}: Fitness - {}, energy - {}", &agent.id.to_string()[..5], agent.fitness, agent.energy)
        }
    }
}

impl fmt::Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Container {{\n id: {},\n agents{:#?}\n}}", self.id, self.agents)
    }
}