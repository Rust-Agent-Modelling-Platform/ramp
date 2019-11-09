use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::process;
use std::time::Instant;

use colored::*;
use rand::{thread_rng, Rng};
use uuid::Uuid;

use crate::action::Action;
use crate::agent::Agent;
use crate::settings::AgentSettings;
use crate::stats;
use rust_in_peace::island::{Island, IslandEnv};
use rust_in_peace::message::Message;

const LOCAL_MIGRATION_THRESHOLD: u32 = 50;

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
    pub all_received_migrations_in_turn: Vec<u32>,
    pub all_sent_migrations_in_turn: Vec<u32>,
    pub local_sent_migrations_in_turn: Vec<u32>,
    pub global_sent_migrations_in_turn: Vec<u32>,
    pub deads_in_turn: Vec<u32>,
}

impl Stats {
    fn new() -> Self {
        Stats {
            best_fitness_in_turn: vec![],
            meetings_in_turn: vec![],
            procreations_in_turn: vec![],
            all_received_migrations_in_turn: vec![],
            all_sent_migrations_in_turn: vec![],
            local_sent_migrations_in_turn: vec![],
            global_sent_migrations_in_turn: vec![],
            deads_in_turn: vec![],
        }
    }
}

pub struct MyIsland {
    pub id: Uuid,
    pub id_agent_map: HashMap<Uuid, RefCell<Agent>>,
    pub action_queue: Vec<Action>,
    pub agent_settings: AgentSettings,
    pub stats: Stats,
    island_env: IslandEnv,
    id_queues: IdQueues,
}
impl Island for MyIsland {
    fn do_turn(&mut self, turn_number: u32, messages: Vec<Message>) {
        self.log_turn_start(turn_number);

        self.resolve_messages(messages);
        self.clear_action_queues();
        self.create_action_queues();
        self.resolve_migrations();
        self.resolve_procreations();
        self.resolve_meetings();
        self.resolve_deads();

        self.log_turn_end_and_update_best_agent();
    }

    fn get_island_env(&self) -> &IslandEnv {
        &self.island_env
    }

    fn run_with_global_sync(&mut self) {
        unimplemented!();
    }
    // fn run_with_global_sync(&mut self) {
    //     log::info!("Run with global Sync");
    //     let start_time = Instant::now();

    //     loop {
    //         // let current_turn = self.receive_messages_with_global_sync();
    //         if current_turn == 0 {
    //             break;
    //         }
    //         self.log_turn_start(current_turn);
    //         self.clear_action_queues();
    //         self.create_action_queues();
    //         self.resolve_migrations();
    //         self.resolve_procreations();
    //         self.resolve_meetings();
    //         self.resolve_deads();

    //         self.log_turn_end_and_update_best_agent();
    //         // self.island_env
    //         //     .address_book
    //         //     .pub_tx
    //         //     .send(Message::TurnDone)
    //         //     .unwrap();
    //     }
    //     //        self.finish(start_time);
    // }

    fn finish(&mut self) {
        let duration = self.island_env.start_time.elapsed().as_secs();
        stats::generate_stat_files(&self, duration.clone(), &self.island_env.stats_dir_path);

        log::info!("{}", "================= END =================".green());
        log::info!("Time elapsed: {} seconds", duration);
        log::info!(
            "At end of simulation the best agent is: {}",
            stats::get_most_fit_agent(&self)
                .borrow()
                .fitness
                .to_string()
                .blue()
        );
    }
}

impl MyIsland {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        island_env: IslandEnv,
        calculate_fitness: &dyn Fn(&[f64]) -> f64,
        agents_number: u32,
        agent_settings: AgentSettings,
    ) -> Self {
        MyIsland {
            id,
            id_agent_map: MyIsland::create_id_agent_map(
                agents_number,
                &agent_settings,
                calculate_fitness,
            ),
            action_queue: Vec::new(),
            agent_settings,
            island_env,
            id_queues: IdQueues::new(),
            stats: Stats::new(),
        }
    }

    /// This method is waiting for ['Message::NexTurn(turn_number)'] msg.
    /// Until it is received each other msg (except ['Message::FinSim'])
    /// is pushed to queue and is proceed after receiving NextTurn msg.
    /// Method returns current turn number if NextTurn msg is received or
    /// 0 in other case.
    // fn receive_messages_with_global_sync(&mut self) -> message::TurnNumber {
    //     let mut msg_queue = vec![];
    //     let mut next_turn = false;
    //     let mut fin_sim = false;
    //     let mut current_turn = 0;
    //     while !next_turn && !fin_sim {
    //         let messages = self.island_env.address_book.self_rx.try_iter();
    //         for msg in messages {
    //             match msg {
    //                 Message::NextTurn(turn_number) => {
    //                     current_turn = turn_number;
    //                     next_turn = true
    //                 }
    //                 Message::FinSim => fin_sim = true,
    //                 _ => msg_queue.push(msg),
    //             }
    //         }
    //     }
    //     let mut migrants_num = 0;
    //     for msg in msg_queue {
    //         match msg {
    //             Message::Agent(migrant) => {
    //                 let d_migrant: Agent = bincode::deserialize(&migrant).unwrap();
    //                 migrants_num += 1;
    //                 self.id_agent_map
    //                     .insert(d_migrant.id, RefCell::new(d_migrant));
    //             }
    //             _ => log::error!("Unexpected msg {:#?}", msg),
    //         }
    //     }
    //     self.stats
    //         .all_received_migrations_in_turn
    //         .push(migrants_num);
    //     current_turn
    // }

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

            let mut agent1 = self.id_agent_map[&id1].borrow_mut();
            let mut agent2 = self.id_agent_map[&id2].borrow_mut();

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

            let mut agent1 = self.id_agent_map[&id1].borrow_mut();
            let mut agent2 = self.id_agent_map[&id2].borrow_mut();

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
        if self.island_env.get_active_islands_number() < 1 {
            self.id_queues.migrating_ids.clear();
            return;
        }
        let mut local_migrations_num = 0;
        let mut global_migrations_num = 0;
        for id in &self.id_queues.migrating_ids {
            let prob = thread_rng().gen_range(0, 100);
            match self.id_agent_map.remove(id) {
                Some(agent) => {
                    let s_agent = bincode::serialize(&agent.into_inner()).unwrap();
                    if prob <= LOCAL_MIGRATION_THRESHOLD {
                        match self.island_env.send_to_rnd_local(Message::Agent(s_agent)) {
                            Ok(()) => local_migrations_num += 1,
                            Err(e) => match e.0 {
                                Message::Agent(s_agent) => {
                                    let d_agent: Agent = bincode::deserialize(&s_agent).unwrap();
                                    self.id_agent_map.insert(*id, RefCell::new(d_agent));
                                }
                                _ => log::info!("Bad return message"),
                            },
                        }
                    } else {
                        self.island_env.send_to_rnd_global(Message::Agent(s_agent));
                        global_migrations_num += 1;
                    }
                }
                None => log::info!("No agent with id {}", id),
            }
        }
        self.stats
            .all_sent_migrations_in_turn
            .push(local_migrations_num + global_migrations_num);
        self.stats
            .local_sent_migrations_in_turn
            .push(local_migrations_num);
        self.stats
            .global_sent_migrations_in_turn
            .push(global_migrations_num);
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
        stats::generate_stat_files(&self, duration, &self.island_env.stats_dir_path);

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

    fn resolve_messages(&mut self, messages: Vec<Message>) {
        let mut migrants_num = 0;
        for message in messages {
            match message {
                Message::Agent(migrant) => {
                    migrants_num += 1;
                    let d_migrant: Agent = bincode::deserialize(&migrant).unwrap();
                    self.id_agent_map
                        .insert(d_migrant.id, RefCell::new(d_migrant));
                }
                _ => log::error!("Unexpected msg"),
            }
        }
        self.stats
            .all_received_migrations_in_turn
            .push(migrants_num);
    }

    fn clear_action_queues(&mut self) {
        self.action_queue.clear();
    }

    fn create_id_agent_map(
        agents_number: u32,
        agent_config: &AgentSettings,
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
impl fmt::Display for MyIsland {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Container {{\n id: {},\n agents{:#?}\n}}",
            self.id, self.id_agent_map
        )
    }
}
