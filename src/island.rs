use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::process;
use std::sync::{Arc, Barrier};
use std::time::Instant;

use colored::*;
use rand::{thread_rng, Rng};
use uuid::Uuid;

use crate::action::Action;
use crate::address_book::AddressBook;
use crate::agent::Agent;
use crate::message::Message;
use crate::settings::AgentConfig;
use crate::stats;

struct IdQueues {
    pub dead_ids: Vec<Uuid>,
    pub meeting_ids: Vec<(Uuid, f64)>,
    pub procreating_ids: Vec<(Uuid, f64)>,
    pub migrating_ids: Vec<Uuid>,
}

impl IdQueues {
    fn new() -> Self {
        IdQueues {
            dead_ids: vec![],
            meeting_ids: vec![],
            procreating_ids: vec![],
            migrating_ids: vec![],
        }
    }
}

pub struct Stats {
    pub best_fitness_in_turn: Vec<f64>,
    pub meetings_in_turn: Vec<u32>,
    pub procreations_in_turn: Vec<u32>,
    pub migrants_received_in_turn: Vec<u32>,
    pub deads_in_turn: Vec<u32>,
}

impl Stats {
    fn new() -> Self {
        Stats {
            best_fitness_in_turn: vec![],
            meetings_in_turn: vec![],
            procreations_in_turn: vec![],
            migrants_received_in_turn: vec![],
            deads_in_turn: vec![],
        }
    }
}

pub struct Island {
    pub id: Uuid,
    pub id_agent_map: HashMap<Uuid, RefCell<Agent>>,
    pub turn_number: u64,
    pub action_queue: Vec<Action>,
    pub agent_config: Arc<AgentConfig>,
    pub stats: Stats,
    turns: u32,
    island_stats_dir_path: String,
    address_book: AddressBook,
    id_queues: IdQueues,
    islands_sync: Option<Arc<Barrier>>,
}

impl Island {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        address_book: AddressBook,
        calculate_fitness: &dyn Fn(&[f64]) -> f64,
        agents_number: u32,
        turns: u32,
        agent_config: Arc<AgentConfig>,
        island_stats_dir_path: String,
        islands_sync: Option<Arc<Barrier>>,
    ) -> Self {
        Island {
            id,
            id_agent_map: Island::create_id_agent_map(
                agents_number,
                &agent_config,
                calculate_fitness,
            ),
            turn_number: 0,
            action_queue: Vec::new(),
            turns,
            agent_config,
            island_stats_dir_path,
            address_book,
            id_queues: IdQueues::new(),
            stats: Stats::new(),
            islands_sync,
        }
    }

    pub fn run(&mut self) {
        let start_time = Instant::now();
        for turn_number in 1..=self.turns {
            self.log_turn_start(turn_number);

            self.receive_messages();
            self.clear_action_queues();
            self.create_action_queues();
            self.resolve_migrations();
            self.resolve_procreations();
            self.resolve_meetings();
            self.resolve_deads();

            self.log_turn_end_and_update_best_agent();

            if let Some(islands_sync) = &self.islands_sync {
                islands_sync.wait();
            };
        }
        self.finish(start_time);
    }

    fn log_turn_start(&self, turn_number: u32) {
        log::debug!(
            "======================== TURN {} ========================== ",
            turn_number
        );
        log::debug!(
            "Number of agents at beginning of turn: {}",
            self.id_agent_map.len()
        );
    }

    // TODO: handle no agents properly
    fn log_turn_end_and_update_best_agent(&mut self) {
        match stats::get_best_fitness(&self) {
            Some(fitness) => {
                self.stats.best_fitness_in_turn.push(fitness);
                log::debug!("Best agent this turn: {}", fitness.to_string().blue());
            }
            None => {
                eprintln!(
                    "{}",
                    "Error: No more agents in system. Check input parameters".red()
                );
                process::exit(1)
            }
        };
        log::debug!(
            "Number of agents in system at end of turn: {}",
            self.id_agent_map.len()
        );
    }

    pub fn create_action_queues(&mut self) {
        for agent in self.id_agent_map.values() {
            let action = agent.borrow().get_action();
            match action {
                Action::Death(id) => self.id_queues.dead_ids.push(id),
                Action::Meeting(id, _) => self
                    .id_queues
                    .meeting_ids
                    .push((id, agent.borrow().fitness)),
                Action::Procreation(id, _) => self
                    .id_queues
                    .procreating_ids
                    .push((id, agent.borrow().fitness)),
                Action::Migration(id) => self.id_queues.migrating_ids.push(id),
            }
        }
    }

    pub fn resolve_procreations(&mut self) {
        if self.id_queues.procreating_ids.is_empty() {
            return;
        }
        log::debug!(
            "Number of agents that want to procreate this turn: {} --> will be {} new agents",
            self.id_queues.procreating_ids.len(),
            self.id_queues.procreating_ids.len() / 2
        );
        // TODO: handle this guy properly
        if self.id_queues.procreating_ids.len() % 2 == 1 {
            let _none_agent = self.id_queues.procreating_ids.remove(0);
        }

        let mut procreating_num = 0;
        while !self.id_queues.procreating_ids.is_empty() {
            let (id1, _) = self.id_queues.procreating_ids.pop().unwrap();
            let (id2, _) = self.id_queues.procreating_ids.pop().unwrap();

            let mut agent1 = self.id_agent_map.get(&id1).unwrap().borrow_mut();
            let mut agent2 = self.id_agent_map.get(&id2).unwrap().borrow_mut();

            let (uuid, new_agent) = agent1.procreate(&mut agent2);
            drop(agent1);
            drop(agent2);

            self.id_agent_map.insert(uuid, new_agent);
            procreating_num += 1;
        }
        self.stats.procreations_in_turn.push(procreating_num);
    }

    pub fn resolve_meetings(&mut self) {
        if self.id_queues.meeting_ids.is_empty() {
            return;
        }
        log::debug!(
            "Number of agents that want a meeting this turn: {}",
            self.id_queues.meeting_ids.len()
        );
        // TODO: handle this guy properly
        if self.id_queues.meeting_ids.len() % 2 == 1 {
            let _none_agent = self.id_queues.meeting_ids.remove(0);
        }

        let mut meeting_num = 0;
        while !self.id_queues.meeting_ids.is_empty() {
            let (id1, _) = self.id_queues.meeting_ids.pop().unwrap();
            let (id2, _) = self.id_queues.meeting_ids.pop().unwrap();

            let mut agent1 = self.id_agent_map.get(&id1).unwrap().borrow_mut();
            let mut agent2 = self.id_agent_map.get(&id2).unwrap().borrow_mut();

            agent1.meet(&mut agent2);
            meeting_num += 1;
        }
        self.stats.meetings_in_turn.push(meeting_num);
    }

    pub fn resolve_migrations(&mut self) {
        log::debug!(
            "Number of migrating agents this turn: {}",
            self.id_queues.migrating_ids.len()
        );
        if self.address_book.addresses.is_empty() {
            self.id_queues.migrating_ids.clear();
            return;
        }
        for id in &self.id_queues.migrating_ids {
            let prob = thread_rng().gen_range(0, 100);
            if prob <= 50 {
                match self.id_agent_map.remove(id) {
                    Some(agent) => match self
                        .address_book
                        .send_to_rnd(Message::Agent(agent.into_inner()))
                    {
                        Ok(()) => (),
                        Err(e) => match e.0 {
                            Message::Agent(agent) => {
                                self.id_agent_map.insert(*id, RefCell::new(agent));
                            }
                            _ => log::info!("Bad return message"),
                        },
                    },
                    None => log::info!("No agent with id {}", id),
                }
            } else {
                match self.id_agent_map.remove(id) {
                    Some(agent) => self
                        .address_book
                        .pub_rx
                        .send(Message::Agent(agent.into_inner()))
                        .unwrap(),
                    None => log::warn!("No id in agent map, id: {}", id),
                }
            }
        }
        self.id_queues.migrating_ids.clear();
    }

    pub fn resolve_deads(&mut self) {
        let deads_in_turn = self.id_queues.dead_ids.len();
        log::debug!(
            "Number of agents that want a death this turn: {}",
            deads_in_turn
        );
        for id in &self.id_queues.dead_ids {
            self.id_agent_map.remove(id);
        }
        self.stats.deads_in_turn.push(deads_in_turn as u32);
        self.id_queues.dead_ids.clear();
    }

    fn finish(&self, start_time: Instant) {
        let duration = start_time.elapsed().as_secs();
        stats::generate_stat_files(&self, duration, &self.island_stats_dir_path);

        log::info!("{}", "================= END =================".green());
        log::info!("Time elapsed: {} seconds", start_time.elapsed().as_secs());
        log::info!(
            "At end of simulation the best agent is: {}",
            stats::get_most_fit_agent(&self)
                .borrow()
                .fitness
                .to_string()
                .blue()
        );
    }

    fn receive_messages(&mut self) {
        let messages = self.address_book.self_rx.try_iter();
        let mut migrants_num = 0;
        for message in messages {
            match message {
                Message::Agent(migrant) => {
                    migrants_num += 1;
                    self.id_agent_map.insert(migrant.id, RefCell::new(migrant));
                }
                _ => log::error!("Unexpected msg"),
            }
        }
        self.stats.migrants_received_in_turn.push(migrants_num);
    }

    fn clear_action_queues(&mut self) {
        self.action_queue.clear();
    }

    fn create_id_agent_map(
        agents_number: u32,
        agent_config: &Arc<AgentConfig>,
        calculate_fitness: &dyn Fn(&[f64]) -> f64,
    ) -> HashMap<Uuid, RefCell<Agent>> {
        let mut id_agent_map: HashMap<Uuid, RefCell<Agent>> = HashMap::new();
        for _i in 0..agents_number {
            let genotype: Vec<f64> = (0..agent_config.genotype_dim)
                .map(|_| thread_rng().gen_range(agent_config.lower_bound, agent_config.upper_bound))
                .collect();
            let id = Uuid::new_v4();
            id_agent_map.insert(
                id,
                Agent::new(
                    id,
                    agent_config.clone(),
                    genotype,
                    calculate_fitness,
                    agent_config.initial_energy,
                ),
            );
        }
        id_agent_map
    }
}

// =============================================== Trait implementations ===========================================================
impl fmt::Display for Island {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Container {{\n id: {},\n agents{:#?}\n}}",
            self.id, self.id_agent_map
        )
    }
}
