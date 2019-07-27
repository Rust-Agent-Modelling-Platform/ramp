use std::fmt;
use uuid::Uuid;
use rand::{thread_rng, Rng};

use crate::agent::Agent;
use crate::action::Action;
use std::any::Any;
use std::ops::Deref;


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
                match *x {
                    Action::Meeting(_, _) => true,
                    _ => false
                }
            }).collect();

        println!{"Number of agents that want a meeting this turn: {}", want_meeting.len()}
        if want_meeting.len() % 2 != 0 { println!{"There is an agent without a pair - gets the None action"} }



        //b. construct a meeting from them
        //if nobody wants to meet - great
        if want_meeting.len() == 0 {
            return
        }

        //for now just get anybody to meet with
        //first deal with unlucky uneven dude, set his action to be None(uuid)
        if want_meeting.len() % 2 == 1 {
            let old_action = want_meeting.pop().unwrap();
            let id = match *old_action {
                Action::Meeting(id, _) => id,
                _ => Uuid::nil()
            };

            let new_action = Action::None(id);

            self.action_queue.retain(|action| {
                match action {
                    Action::Meeting(id, _) => false,
                    _ => true
                }
            });
            self.action_queue.push(new_action);
        }

        //rest of dudes meet
        else {
            //defeated by the borrow checker

//            let want_meeting_clone = want_meeting.clone();
//            while want_meeting.len() > 0 {
//                let id1 = match *want_meeting.pop().unwrap(){
//                    Action::Meeting(id, _) => id,
//                    _ => Uuid::nil()
//                };
//                let id2 = match *want_meeting.pop().unwrap() {
//                    Action::Meeting(id, _) => id,
//                    _ => Uuid::nil()
//                };
//                let new_action = Action::Meeting(id1, id2);
//
//                self.action_queue.retain(|action| {
//                    match action {
//                        Action::Meeting(id1, _) => false,
//                        _ => true
//                    }
//                });
//                self.action_queue.push(new_action);
//            }



        }

    }

    pub fn clear_action_queue(&mut self) {
        self.action_queue.clear();
    }

    pub fn execute_actions(&mut self) {
        //TODO
    }

    pub fn remove_dead_agents(&mut self) {
        let dead_agents= self.action_queue.iter()
            .map(|a|
                match a {
                    Action::Death(id) => *id,
                    _ => Uuid::nil()
        }).collect();
        println!{"Dead: {:?}", dead_agents};
        self.remove_agents(dead_agents);
    }

    // Private methods
    fn remove_one_agent(&mut self, id: Uuid) {
        self.agents.retain(|agent| agent.id != id)
    }

    fn remove_agents(&mut self, ids: Vec<Uuid>) {
        self.agents.retain(|agent| !ids.contains(&agent.id))
    }


    // Public utility methods
    pub fn print_action_queue(&self) {
        for action in &self.action_queue {
            println!("{}", action)
        }
    }

    pub fn print_agent_fitness(&self) {
        for agent in &self.agents {
            println!("Agent {}: {}", &agent.id.to_string()[..5], agent.fitness)
        }
    }


    //pub fn mutate() { }
    //pub fn crossover() {}
}


impl fmt::Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Container {{\n id: {},\n agents{:#?}\n}}", self.id, self.agents)
    }
}