use std::fmt;
use uuid::Uuid;
use rand::{thread_rng, Rng};

use crate::agent::Agent;
use crate::action::Action;

use crate::functions;

pub struct Container {
    pub id: Uuid,
    pub agents: Vec<Agent>,
    pub turn_number: u64,
    pub action_queue: Vec<Action>
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
            agents,
            turn_number: 0,
            action_queue: Vec::new()
        }
    }


    pub fn create_action_queue(&mut self) {
        for agent in &self.agents {
            let action = agent.get_action();
            self.action_queue.push(action);
        }
    }


    pub fn resolve_meetings(&mut self) {
        //1. First resolve meetings
        //a. get those who want a meeting
        let mut want_meeting: Vec<&Action> = self.action_queue.iter()
            .filter(|x| {
                match **x {
                    Action::Meeting(_, _) => true,
                    _ => false
                }
            }).collect();

        println! {"Number of agents that want a meeting this turn: {}", want_meeting.len()}
        if want_meeting.len() % 2 != 0 { println! {"There is an agent without a pair - gets the None action"} }

        //b. construct a meeting from them
        //if nobody wants to meet - great
        if want_meeting.len() == 0 { return }

        //temp structures - both get dropped at the end of the function
        let mut new_actions_list = Vec::new();
        let mut ids = Vec::new();

        //for now, just get anybody to meet with
        //first deal with uneven agent, set his action to be None(uuid)
        if want_meeting.len() % 2 == 1 {
            let old_action = want_meeting.pop().unwrap();
            let id = match *old_action {
                Action::Meeting(id, _) => id,
                _ => Uuid::nil()
            };
            let new_action = Action::None(id);
            new_actions_list.push(new_action);
            ids.push(id);
        }

        //rest of dudes meet
        //add meeting action with different values to temp list
        if want_meeting.len() % 2 != 1 {
            while want_meeting.len() != 0 {
                let action_1 = want_meeting.pop().unwrap();
                let id1 = match *action_1 {
                    Action::Meeting(id, _) => id,
                    _ => Uuid::nil()
                };

                let action_2 = want_meeting.pop().unwrap();
                let id2 = match *action_2 {
                    Action::Meeting(id, _) => id,
                    _ => Uuid::nil()
                };

                let new_action = Action::Meeting(id1, id2);
                new_actions_list.push(new_action);
                ids.push(id1);
                ids.push(id2);
            }
        }
        //substitute in action_queue according to temp list
        self.action_queue.retain(|action|
            match action {
                Action::Meeting(_, _) => false,
                _ => true
            });
        for action in new_actions_list {
            self.action_queue.push(action);
        }
    }


    pub fn resolve_procreation(&mut self) {
        //2. Resolve procreation
        //a. get those who want to procreate
        let mut want_procreation: Vec<&Action> = self.action_queue.iter()
            .filter(|x| {
                match **x {
                    Action::Procreation(_, _) => true,
                    _ => false
                }
            }).collect();

        println! {"Number of agents that want to procreate this turn: {}", want_procreation.len()}
        if want_procreation.len() % 2 != 0 { println! {"There is an agent without a pair - gets the None action"} }

        //b. construct a meeting from them
        //if nobody wants to meet - great
        if want_procreation.len() == 0 { return }

        //temp structures - both get dropped at the end of the function
        let mut new_actions_list = Vec::new();
        let mut ids = Vec::new();

        //for now, just get anybody to procreate with
        //first deal with uneven agent, set his action to be None(uuid)
        if want_procreation.len() % 2 == 1 {
            let old_action = want_procreation.pop().unwrap();
            let id = match *old_action {
                Action::Procreation(id, _) => id,
                _ => Uuid::nil()
            };
            let new_action = Action::None(id);
            new_actions_list.push(new_action);
            ids.push(id);
        }

        //rest of dudes meet
        //add meeting action with different values to temp list
        if want_procreation.len() % 2 != 1 {
            while want_procreation.len() != 0 {
                let action_1 = want_procreation.pop().unwrap();
                let id1 = match *action_1 {
                    Action::Procreation(id, _) => id,
                    _ => Uuid::nil()
                };

                let action_2 = want_procreation.pop().unwrap();
                let id2 = match *action_2 {
                    Action::Procreation(id, _) => id,
                    _ => Uuid::nil()
                };

                let new_action = Action::Procreation(id1, id2);
                new_actions_list.push(new_action);
                ids.push(id1);
                ids.push(id2);
            }
        }
        //substitute in action_queue according to temp list
        self.action_queue.retain(|action|
            match action {
                Action::Procreation(_, _) => false,
                _ => true
            });
        for action in new_actions_list {
            self.action_queue.push(action);
        }
    }


    pub fn clear_action_queue(&mut self) {
        self.action_queue.clear();
    }


    pub fn execute_actions(&mut self) {
        self.execute_meetings();
        self.execute_procreation();

        //cleanup
        self.action_queue.retain(|x| {
            match x {
                Action::Meeting(_,_) => false,
                Action::Procreation(_,_) => false,
                _ => true
            }
        });

    }


    pub fn remove_dead_agents(&mut self) {
        for action in &self.action_queue {
            if let Action::Death(id1) = action  {
                let (index, _) = self.agents.iter().enumerate().find(|(_i, agent)| agent.id == *id1).unwrap();
                self.agents.remove(index);
            }
        }
        self.action_queue.retain(|x| {
            match x {
                Action::Death(_) => false,
                _ => true
            }
        });
    }


    pub fn remove_none_actions (&mut self) {
        self.action_queue.retain(|x| {
            match x {
                Action::None(_) => false,
                _ => true
            }
        });
    }


    pub fn remove_migrants(&mut self ) {
        for action in &self.action_queue {
            if let Action::Migration(id1) = action {
                let (index, _) = self.agents.iter().enumerate().find(|(_i, agent)| agent.id == *id1).unwrap();
                self.agents.remove(index);
            }
        }
        self.action_queue.retain(|x| {
            match x {
                Action::Migration(_) => false,
                _ => true
            }
        });
    }


    // ================================================ Private methods ====================================================
    fn remove_one_agent(&mut self, id: Uuid) {
        self.agents.retain(|agent| agent.id != id)
    }


    fn remove_agents(&mut self, ids: Vec<Uuid>) {
        self.agents.retain(|agent| !ids.contains(&agent.id))
    }


    fn execute_meetings(&mut self) {
        for action in &self.action_queue {
            if let Action::Meeting(id1, id2) = action {
                let (index1, _) = self.agents.iter().enumerate().find(|(_i, agent)| agent.id == *id1).unwrap();
                let (index2, _) = self.agents.iter().enumerate().find(|(_i, agent)| agent.id == *id2).unwrap();

                if self.agents[index1].fitness > self.agents[index2].fitness  {
                    self.agents[index1].energy+=40;
                    self.agents[index2].energy-=40;
                } else {
                    self.agents[index2].energy-=40;
                    self.agents[index1].energy+=40;
                }
            }
        }
    }

    fn execute_procreation(&mut self) {
        for action in &self.action_queue {
            if let Action::Procreation(id1, id2) = action {
                let (index1, _) = self.agents.iter().enumerate().find(|(_i, agent)| agent.id == *id1).unwrap();
                let (index2, _) = self.agents.iter().enumerate().find(|(_i, agent)| agent.id == *id2).unwrap();

                //incur penalty for procreation
                self.agents[index1].energy-=10;
                self.agents[index2].energy-=10;

                //create new genotype
                let mut new_genotype = vec![];

                //crossover
                self.crossover(index1, index2, &mut new_genotype);

                //mutate the new genotype
                self.mutate_genotype(&mut new_genotype);

                let new_agent = Agent::create(new_genotype , &functions::rastrigin);
                println!("NEW AGENT {}", new_agent);

                self.agents.push(new_agent);
            }
        }
    }


    fn mutate_genotype(&self, genotype: &mut Vec<f64>) {
        let length = genotype.len();
        let gene = thread_rng().gen_range(0, genotype.len() - 1);
        //TODO: range should not be hardcoded
        genotype[gene] = thread_rng().gen_range(-5.12, 5.12);
    }


    fn crossover(&self, id1: usize, id2: usize, genotype: &mut Vec<f64>) {
        //TODO: ranges cannot be hardcoded
        let head = &self.agents[id1].genotype[..1];
        let tail = &self.agents[id2].genotype[1..];
        genotype.extend_from_slice(head);
        genotype.extend_from_slice(tail);
    }


    // =============================================== Public utility methods =========================================================
    pub fn print_action_queue(&self) {
        for action in &self.action_queue {
            println!("{}", action)
        }
        println!("Nr of entries in this queue: {}", self.action_queue.len());
    }

    pub fn print_agent_stats(&self) {
        for agent in &self.agents {
            println!("Agent {}: Fitness - {}, energy - {}", &agent.id.to_string()[..5], agent.fitness, agent.energy)
        }
    }
}

impl fmt::Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Container {{\n id: {},\n agents{:#?}\n}}", self.id, self.agents)
    }
}