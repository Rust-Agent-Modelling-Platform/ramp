use crate::{
    ALL_RECV_MIGR_MN, ALL_SENT_MIGR_MN, BEST_FITNESS_MN, DEADS_MN, GLOB_RECV_MIGR_MN,
    LOC_RECV_MIGR_MN, MEETINGS_MN, PROCREATIONS_MN,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::process;
use std::sync::Arc;

use colored::*;
use rand::{thread_rng, Rng};
use uuid::Uuid;

use crate::action::Action;
use crate::agent::Agent;
use crate::settings::AgentSettings;
use crate::utils;
use ramp::island::{Island, IslandEnv};
use ramp::message::Message;

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

pub struct MyIsland {
    pub id: Uuid,
    pub id_agent_map: HashMap<Uuid, RefCell<Agent>>,
    pub action_queue: Vec<Action>,
    pub agent_settings: Arc<AgentSettings>,
    island_env: IslandEnv,
    id_queues: IdQueues,
}
impl Island for MyIsland {
    fn on_start(&mut self) {}

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

    fn on_finish(&mut self) {
        let duration = self.island_env.start_time.elapsed().as_secs();

        log::info!("{}", "================= END =================".green());
        log::info!("Time elapsed: {} seconds", duration);
        log::info!(
            "At end of simulation the best fitness is: {}",
            self.get_best_fitness().unwrap()
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
        agent_settings: Arc<AgentSettings>,
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
        }
    }

    pub fn get_best_fitness(&self) -> Option<f64> {
        let mut top_guy = match self.id_agent_map.values().take(1).last() {
            Some(a) => a,
            None => return None,
        };
        for agent in self.id_agent_map.values() {
            if agent.borrow().fitness > top_guy.borrow().fitness {
                top_guy = agent;
            }
        }
        Some(top_guy.borrow().fitness)
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
        match self.get_best_fitness() {
            Some(fitness) => {
                self.island_env.metric_hub.set_gauge_vec(
                    BEST_FITNESS_MN,
                    &[&utils::short_id(&self.id)],
                    fitness,
                );
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

        self.island_env.metric_hub.add_int_gauge_vec(
            PROCREATIONS_MN,
            &[&utils::short_id(&self.id)],
            procreating_num,
        );
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
        self.island_env.metric_hub.add_int_gauge_vec(
            MEETINGS_MN,
            &[&utils::short_id(&self.id)],
            meeting_num,
        );
    }

    pub fn resolve_migrations(&mut self) {
        log::debug!(
            "Number of migrating agents this turn: {}",
            self.id_queues.migrating_ids.len()
        );
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

        self.island_env.metric_hub.add_int_gauge_vec(
            ALL_SENT_MIGR_MN,
            &[&utils::short_id(&self.id)],
            local_migrations_num + global_migrations_num,
        );
        self.island_env.metric_hub.add_int_gauge_vec(
            LOC_RECV_MIGR_MN,
            &[&utils::short_id(&self.id)],
            local_migrations_num,
        );
        self.island_env.metric_hub.add_int_gauge_vec(
            GLOB_RECV_MIGR_MN,
            &[&utils::short_id(&self.id)],
            global_migrations_num,
        );
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
        self.island_env.metric_hub.add_int_gauge_vec(
            DEADS_MN,
            &[&utils::short_id(&self.id)],
            deads_in_turn as i64,
        );
        self.id_queues.dead_ids.clear();
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
        self.island_env.metric_hub.add_int_gauge_vec(
            ALL_RECV_MIGR_MN,
            &[&utils::short_id(&self.id)],
            migrants_num,
        );
    }

    fn clear_action_queues(&mut self) {
        self.action_queue.clear();
    }

    fn create_id_agent_map(
        agents_number: u32,
        agent_config: &Arc<AgentSettings>,
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

// ========================== Trait implementations ==========================
impl fmt::Display for MyIsland {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Container {{\n id: {},\n agents{:#?}\n}}",
            self.id, self.id_agent_map
        )
    }
}
