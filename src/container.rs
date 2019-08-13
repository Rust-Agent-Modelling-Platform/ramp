use colored::*;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::fmt;
use std::time::Instant;
use uuid::Uuid;

use crate::action::Action;
use crate::agent::Agent;
use crate::functions;
use crate::stats;

pub struct Container {
    pub id: Uuid,
    pub id_agent_map: HashMap<Uuid, Agent>,
    pub turn_number: u64,
    pub action_queue: Vec<Action>,
    pub dim: i32,
    pub interval: (f64, f64),
    turns: u32,
    island_stats_dir_path: String,

    pub dead_ids: Vec<Uuid>,
    pub meeting_ids: Vec<(Uuid, f64)>,
    pub procreating_ids: Vec<(Uuid, f64)>,
    pub migrating_ids: Vec<Uuid>,

    pub best_fitness_in_turn: Vec<f64>,
    pub meetings_in_turn: Vec<u32>,
    pub procreations_in_turn: Vec<u32>,
    pub migrants_received_in_turn: Vec<u32>,
}

impl Container {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        calculate_fitness: &dyn Fn(&[f64]) -> f64,
        agents_number: u32,
        dim: i32,
        interval: (f64, f64),
        turns: u32,
        island_stats_dir_path: String,
    ) -> Self {
        let mut id_agent_map: HashMap<Uuid, Agent> = HashMap::new();

        for _i in 0..agents_number {
            let genotype: Vec<f64> = (0..dim)
                .map(|_| thread_rng().gen_range(interval.0, interval.1))
                .collect();
            let id = Uuid::new_v4();
            id_agent_map.insert(id, Agent::new(id, genotype, calculate_fitness, 100));
        }
        Container {
            id,
            id_agent_map,
            turn_number: 0,
            action_queue: Vec::new(),
            dim,
            interval,
            turns,
            island_stats_dir_path,

            dead_ids: Vec::new(),
            meeting_ids: Vec::new(),
            procreating_ids: Vec::new(),
            migrating_ids: Vec::new(),

            best_fitness_in_turn: vec![],
            meetings_in_turn: vec![],
            procreations_in_turn: vec![],
            migrants_received_in_turn: vec![],
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
        let mut procreating_num = 0;

        if self.procreating_ids.is_empty() {
            return;
        }
        log::info!(
            "Number of agents that want to procreate this turn: {} --> will be {} new agents",
            self.procreating_ids.len(),
            self.procreating_ids.len() / 2
        );
        // no pair - just remove him at this moment
        if self.procreating_ids.len() % 2 == 1 {
            let _none_agent = self.procreating_ids.remove(0);
            //log::info!("Getting none from procreation is: {}", none_agent.0.to_string());
        }

        while !self.procreating_ids.is_empty() {
            let id1 = self.procreating_ids.pop().unwrap();
            let id2 = self.procreating_ids.pop().unwrap();
            self.procreate(id1.0, id2.0);
            procreating_num += 1;
        }

        //add rest to meeting_ids
        for (id, fitness) in &self.procreating_ids {
            self.meeting_ids.push((*id, *fitness));
        }

        self.procreations_in_turn.push(procreating_num);
        self.procreating_ids.clear();
    }

    pub fn resolve_meetings(&mut self) {
        let mut meeting_num = 0;
        if self.meeting_ids.is_empty() {
            return;
        }
        log::info!(
            "Number of agents that want a meeting this turn: {}",
            self.meeting_ids.len()
        );
        // no pair - just remove him at this moment
        if self.meeting_ids.len() % 2 == 1 {
            let _none_agent = self.meeting_ids.remove(0);
            //log::info!("Getting none from meeting is: {}", none_agent.0.to_string());
        }

        while !self.meeting_ids.is_empty() {
            let (id1, _) = self.meeting_ids.pop().unwrap();
            let (id2, _) = self.meeting_ids.pop().unwrap();
            self.meet(id1, id2);
            meeting_num += 1;
        }
        self.meetings_in_turn.push(meeting_num);
        self.migrating_ids.clear();
    }

    pub fn clear_action_queues(&mut self) {
        self.action_queue.clear();
    }

    pub fn remove_dead_agents(&mut self) {
        //log::info!("{:?}", self.dead_ids);
        for id in &self.dead_ids {
            self.id_agent_map.remove(id);
        }
        self.dead_ids.clear();
    }

    pub fn remove_migrants(&mut self) {
        for id in &self.migrating_ids {
            self.id_agent_map.remove(id);
            log::info!("Migrating agent id: {}", id);
        }
        self.meeting_ids.clear();
    }

    pub fn run(&mut self) {
        let now = Instant::now();
        for turn_number in 1..=self.turns {
            log::info!(
                "======================== TURN {} ========================== ",
                turn_number
            );
            let dead_num = self.dead_ids.len();
            self.remove_dead_agents();
            log::info!(
                "Number of agents at beginning of turn: {}, died: {}",
                self.id_agent_map.len(),
                dead_num
            );
            self.remove_migrants();
            self.clear_action_queues();
            self.create_action_queues();
            self.resolve_procreation();
            self.resolve_meetings();

            let best_agent_in_turn = stats::get_best_fitness(&self);
            self.best_fitness_in_turn.push(best_agent_in_turn);
            log::info!(
                "Number of agents in system at end of turn (including those who are now dead): {}",
                self.id_agent_map.len()
            );
            log::info!(
                "Best agent this turn: {}",
                best_agent_in_turn.to_string().blue()
            );
        }
        let time = now.elapsed().as_secs();
        stats::generate_stat_files(&self, time, &self.island_stats_dir_path);

        log::info!("{}", "================= END =================".green());
        log::info!("Time elapsed: {} seconds", now.elapsed().as_secs());
        log::info!("At end of simulation the best agent is:");
        stats::print_best_fitness(&self);
    }

    // ================================================ Private methods ====================================================
    fn meet(&mut self, id1: Uuid, id2: Uuid) {
        if self.id_agent_map.get_mut(&id1).unwrap().fitness
            > self.id_agent_map.get_mut(&id2).unwrap().fitness
        {
            self.id_agent_map.get_mut(&id1).unwrap().energy += 50;
            self.id_agent_map.get_mut(&id2).unwrap().energy -= 50;
        } else {
            self.id_agent_map.get_mut(&id2).unwrap().energy += 50;
            self.id_agent_map.get_mut(&id1).unwrap().energy -= 50;
        }
    }

    fn procreate(&mut self, id1: Uuid, id2: Uuid) {
        let child_energy = self.id_agent_map.get_mut(&id1).unwrap().energy / 2
            + self.id_agent_map.get_mut(&id2).unwrap().energy / 2;


        self.id_agent_map.get_mut(&id1).unwrap().energy =
            self.id_agent_map.get_mut(&id1).unwrap().energy / 2;
        self.id_agent_map.get_mut(&id2).unwrap().energy =
            self.id_agent_map.get_mut(&id2).unwrap().energy / 2;

        let mut new_genotype = Agent::crossover(
            &self.id_agent_map[&id1].genotype,
            &self.id_agent_map[&id2].genotype,
        );
        Agent::mutate_genotype(&mut new_genotype, self.interval);
        let uuid = Uuid::new_v4();
        let new_agent = Agent::new(uuid, new_genotype, &functions::rastrigin, child_energy);
        self.id_agent_map.insert(uuid, new_agent);
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
