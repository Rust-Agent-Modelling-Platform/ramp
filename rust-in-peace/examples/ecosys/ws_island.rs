use crate::agent_types::AgentType;
use crate::settings::{IslandSettings, SheepSettings, WolfSettings};
use crate::sheep::Sheep;
use crate::wolves::Wolves;
use crate::ws_utils;
use rand::Rng;
use rust_in_peace::island::{Island, IslandEnv};
use rust_in_peace::map::{FragmentOwner, MapInstance};
use rust_in_peace::message::Message;
use std::ops::Range;
use std::sync::Arc;
use uuid::Uuid;

pub struct WSIsland {
    pub id: Uuid,
    island_env: IslandEnv,
    map: Option<MapInstance>,
    pub island_settings: Arc<IslandSettings>,
    pub sheep_settings: Arc<SheepSettings>,
    pub wolf_settings: Arc<WolfSettings>,
    pub sheep: Sheep,
    pub wolves: Wolves,

    pub remove_sheep: Vec<Uuid>,
    pub remove_wolves: Vec<Uuid>,

    pub outgoing_local: Vec<(AgentType, Uuid, FragmentOwner)>,
    pub outgoing_global: Vec<(AgentType, Uuid, FragmentOwner)>,

    pub new_sheep: Vec<Uuid>,
    pub new_wolves: Vec<Uuid>,
}
enum MoveVector {
    North,
    East,
    South,
    West,
    None,
}
enum BoundaryCheck {
    InBoundary,
    OutBoundaryLocal(FragmentOwner),
    OutBoundaryGlobal(FragmentOwner),
    Impossible,
}

impl Island for WSIsland {
    fn on_start(&mut self) {
        let map = MapInstance::get_instance(&self.island_env);
        log::warn!("{:#?}", map);
        self.map = Some(map);

        let range = self.map.as_ref().unwrap().get_my_range();
        self.sheep.set_initial_sheep_positions(
            range.clone(),
            self.map.as_ref().unwrap().map.chunk_len.clone(),
        );
        self.wolves.set_initial_wolf_positions(
            range.clone(),
            self.map.as_ref().unwrap().map.chunk_len.clone(),
        );

        self.map.as_mut().unwrap().init_with_val(0);
    }

    fn do_turn(&mut self, turn_number: u32, messages: Vec<Message>) {
        log::warn!("================================== TURN {} ON ISLAND {}: ===============================", turn_number, &self.id.to_string()[..8]);
        self.receive_migrants(messages);
        self.do_sheep_turn();
        self.do_wolf_turn();
        self.update_grass();
        self.send_local_migrants();
        self.send_global_migrants();
        self.add_new_agents();
        self.remove_dead_agents();
        self.clear_queues();
        self.display_turn_stats();
    }

    fn on_finish(&mut self) {
        let _duration = self.island_env.start_time.elapsed().as_secs();
        self.display_final_stats();
    }
}
impl WSIsland {
    pub fn new(
        id: Uuid,
        island_env: IslandEnv,
        island_settings: Arc<IslandSettings>,
        sheep_settings: Arc<SheepSettings>,
        wolf_settings: Arc<WolfSettings>,
    ) -> Self {
        Self {
            id,
            island_env,
            map: None,
            island_settings: island_settings.clone(),
            sheep_settings: sheep_settings.clone(),
            wolf_settings: wolf_settings.clone(),
            sheep: Sheep::new(sheep_settings.clone()),
            wolves: Wolves::new(wolf_settings.clone()),
            remove_sheep: vec![],
            remove_wolves: vec![],
            outgoing_local: vec![],
            outgoing_global: vec![],
            new_sheep: vec![],
            new_wolves: vec![],
        }
    }

    //=================================================================================================
    //========================================= Turn methods ==========================================
    //=================================================================================================

    fn receive_migrants(&mut self, messages: Vec<Message>) {
        log::info!("Receiving migrants in {}", &self.id.to_string()[..8]);
        let mut migrants_num = 0;
        for message in messages {
            match message {
                Message::Agent(migrant) => {
                    migrants_num += 1;
                    let (agent_type, id, energy, position) = ws_utils::deserialize(migrant);
                    match agent_type {
                        AgentType::Sheep => {
                            log::warn!("Received new sheep {} with position {:?}", id, position);
                            self.sheep.add_new_sheep(id, energy, position)
                        }
                        AgentType::Wolf => {
                            log::warn!("Received new wolf {} with position {:?}", id, position);
                            self.wolves.add_new_wolf(id, energy, position)
                        }
                    }
                }
                _ => log::error!("Unexpected msg"),
            }
        }
        log::info!("Total number of migrants received: {}", migrants_num);
    }

    fn do_sheep_turn(&mut self) {
        log::info!("Beginning sheep turn in {}", &self.id.to_string()[..8]);
        if self.sheep.id.is_empty() {
            log::warn!(
                "There are no more sheep on island {}",
                &self.id.to_string()[..8]
            );
            return;
        }
        let range = self.map.as_ref().unwrap().get_my_range();
        for sheep in &self.sheep.id {
            self.sheep.print_sheep(sheep);

            let grass = self
                .map
                .as_ref()
                .unwrap()
                .get_value(*self.sheep.position.get(sheep).unwrap());
            if grass == 0 {
                *self.sheep.energy.get_mut(sheep).unwrap() += self.sheep_settings.energy_gain;
                self.map
                    .as_mut()
                    .unwrap()
                    .update_value(-1, *self.sheep.position.get(sheep).unwrap());
            }

            if self.is_reproducing() {
                log::warn!("Sheep {} is reproducing", &sheep.to_string()[..8]);
                self.new_sheep.push(Uuid::new_v4());
            }

            let curr_pos = *self.sheep.position.get(&sheep).unwrap();
            let move_vector = self.get_random_movement_dir();
            let new_pos = self.get_new_position(curr_pos, move_vector);
            log::info!("The new position for this sheep is to be {:?}", new_pos);

            //log::info!("Verifying range {:?} for pos {:?}", &range, &new_pos);
            let move_action =
                self.determine_move_action(&new_pos, &range, &self.map.as_ref().unwrap());

            *self.sheep.position.get_mut(&sheep).unwrap() = new_pos;
            *self.sheep.energy.get_mut(sheep).unwrap() -= self.sheep_settings.energy_loss;
            if *self.sheep.energy.get(&sheep).unwrap() <= 0 {
                self.remove_sheep.push(*sheep);
            }

            match move_action {
                BoundaryCheck::InBoundary => {
                    log::info!("This position is in the current range ");
                }
                BoundaryCheck::OutBoundaryLocal(owner) => {
                    log::info!("Sending to local island {} ", &owner.2.to_string()[..8]);
                    self.outgoing_local.push((AgentType::Sheep, *sheep, owner));
                    self.remove_sheep.push(*sheep);
                }
                BoundaryCheck::OutBoundaryGlobal(owner) => {
                    let address = &format!("{}:{}", owner.0.to_string(), owner.1.to_string());
                    log::info!(
                        "Sent to host {} to island {}",
                        address,
                        &owner.2.to_string()[..8]
                    );
                    self.outgoing_global.push((AgentType::Sheep, *sheep, owner));
                    self.remove_sheep.push(*sheep);
                }
                _ => {
                    log::info!("Move out of bounds - sheep stays where it is");
                    *self.sheep.position.get_mut(&sheep).unwrap() = curr_pos;
                }
            }
        }
    }

    fn do_wolf_turn(&mut self) {
        log::info!("Beginning wolf turn in {}", &self.id.to_string()[..8]);
        if self.wolves.id.is_empty() {
            log::warn!(
                "There are no more wolves on island {}",
                &self.id.to_string()[..8]
            );
            return;
        }
        let range = self.map.as_ref().unwrap().get_my_range();
        for wolf in &self.wolves.id {
            let prey = self.check_for_sheep_at_position(*self.wolves.position.get(wolf).unwrap());
            if prey != None {
                log::warn!(
                    "Wolf {} is consuming sheep {}",
                    &wolf.to_string()[..8],
                    &prey.unwrap().to_string()[..8]
                );
                self.remove_sheep.push(prey.unwrap());
                *self.wolves.energy.get_mut(wolf).unwrap() += self.wolf_settings.energy_gain;
            }

            if self.is_reproducing() {
                log::warn!("Wolf {} is reproducing", &wolf.to_string()[..8]);
                self.new_wolves.push(Uuid::new_v4());
            }

            let curr_pos = *self.wolves.position.get(&wolf).unwrap();
            let move_vector = self.get_random_movement_dir();
            let new_pos = self.get_new_position(curr_pos, move_vector);
            log::info!("The new position for this wolf is to be {:?}", new_pos);

            //log::info!("Verifying range {:?} for pos {:?}", &range, &new_pos);

            let move_action =
                self.determine_move_action(&new_pos, &range, &self.map.as_ref().unwrap());

            *self.wolves.position.get_mut(&wolf).unwrap() = new_pos;
            *self.wolves.energy.get_mut(wolf).unwrap() -= self.wolf_settings.energy_loss;
            if *self.wolves.energy.get(&wolf).unwrap() <= 0 {
                self.remove_wolves.push(*wolf);
            }

            match move_action {
                BoundaryCheck::InBoundary => {
                    log::info!("This position is in the current range ");
                }
                BoundaryCheck::OutBoundaryLocal(owner) => {
                    log::info!(
                        "This should be sent to local island {} ",
                        &owner.2.to_string()[..8]
                    );
                    self.outgoing_local.push((AgentType::Wolf, *wolf, owner));
                    self.remove_wolves.push(*wolf);
                }
                BoundaryCheck::OutBoundaryGlobal(owner) => {
                    let address = &format!("{}:{}", owner.0.to_string(), owner.1.to_string());
                    log::info!(
                        "This should be sent to host {} to island {}",
                        address,
                        &owner.2.to_string()[..8]
                    );
                    self.outgoing_global.push((AgentType::Wolf, *wolf, owner));
                    self.remove_wolves.push(*wolf);
                }
                _ => {
                    log::info!("Move out of bounds - sheep therefore stays where it is");
                    *self.wolves.position.get_mut(&wolf).unwrap() = curr_pos;
                }
            }
        }
    }

    fn update_grass(&mut self) {
        for val in self.map.as_mut().unwrap().data.iter_mut() {
            if *val == -1 {
                *val = self.island_settings.grass_interval;
            } else if *val == 0 {
                continue;
            } else if 0 < *val && *val <= self.island_settings.grass_interval {
                *val = *val - 1;
            }
        }
    }

    fn send_local_migrants(&mut self) {
        for (agent_type, id, (_, _, island_id)) in self.outgoing_local.iter() {
            let serialized;
            match agent_type {
                AgentType::Sheep => {
                    log::warn!(
                        "Sending sheep {} with position {:?} to local island",
                        &id.to_string()[..8],
                        *self.sheep.position.get(&id).unwrap()
                    );
                    serialized = ws_utils::serialize(
                        AgentType::Sheep,
                        *id,
                        *self.sheep.energy.get(id).unwrap(),
                        *self.sheep.position.get(id).unwrap(),
                    );
                }
                AgentType::Wolf => {
                    log::warn!(
                        "Sending wolf {} with position {:?} to local island",
                        &id.to_string()[..8],
                        *self.wolves.position.get(&id).unwrap()
                    );
                    serialized = ws_utils::serialize(
                        AgentType::Wolf,
                        *id,
                        *self.wolves.energy.get(id).unwrap(),
                        *self.wolves.position.get(id).unwrap(),
                    );
                }
            }
            self.island_env
                .send_to_local(*island_id, Message::Agent(serialized))
                .expect("Error sending local migrant");
        }
    }

    fn send_global_migrants(&mut self) {
        for (agent_type, id, (ip, port, _island_id)) in self.outgoing_global.iter() {
            let serialized;
            match agent_type {
                AgentType::Sheep => {
                    log::warn!(
                        "Sending sheep {} with position {:?} to another host",
                        &id.to_string()[..8],
                        *self.sheep.position.get(&id).unwrap()
                    );
                    serialized = ws_utils::serialize(
                        AgentType::Sheep,
                        *id,
                        *self.sheep.energy.get(id).unwrap(),
                        *self.sheep.position.get(id).unwrap(),
                    );
                }
                AgentType::Wolf => {
                    log::warn!(
                        "Sending wolf {} with position {:?} to another host",
                        &id.to_string()[..8],
                        *self.wolves.position.get(&id).unwrap()
                    );
                    serialized = ws_utils::serialize(
                        AgentType::Wolf,
                        *id,
                        *self.wolves.energy.get(id).unwrap(),
                        *self.wolves.position.get(id).unwrap(),
                    );
                }
            }
            self.island_env
                .send_to_global((ip.clone(), *port), Message::Agent(serialized));
        }
    }

    fn add_new_agents(&mut self) {
        for new_sheep in self.new_sheep.iter_mut() {
            self.sheep.add_new_sheep(
                *new_sheep,
                self.sheep_settings.init_energy,
                ws_utils::generate_random_position(
                    &self.map.as_ref().unwrap().get_my_range(),
                    self.map.as_ref().unwrap().map.chunk_len.clone(),
                ),
            );
        }
    }

    fn remove_dead_agents(&mut self) {
        for dead_agent in self.remove_sheep.iter() {
            self.sheep.id.retain(|s| s != dead_agent);
            self.sheep.position.remove(dead_agent);
            self.sheep.energy.remove(dead_agent);
        }
        //log::warn!("Sheep after removal: {:?}", &self.sheep.id);
        for dead_agent in self.remove_wolves.iter() {
            self.wolves.id.retain(|s| s != dead_agent);
            self.wolves.position.remove(dead_agent);
            self.wolves.energy.remove(dead_agent);
        }
        //log::warn!("Wolves after removal: {:?}", &self.wolves.id);
    }

    fn clear_queues(&mut self) {
        self.outgoing_local.clear();
        self.outgoing_global.clear();
        self.remove_sheep.clear();
        self.remove_wolves.clear();
    }

    //=================================================================================================
    //========================================= Helper methods ========================================
    //=================================================================================================

    fn is_reproducing(&self) -> bool {
        let mut rng = rand::thread_rng();
        let chance = rng.gen_range(0.0, 1.0);
        chance <= self.sheep.reproduction_chance
    }

    fn get_random_movement_dir(&self) -> MoveVector {
        let mut rng = rand::thread_rng();
        let dir = rng.gen_range(0, 4);
        match dir {
            0 => MoveVector::North,
            1 => MoveVector::East,
            2 => MoveVector::South,
            3 => MoveVector::West,
            _ => MoveVector::None,
        }
    }

    fn get_new_position(&self, pos: (i64, i64), delta: MoveVector) -> (i64, i64) {
        match delta {
            MoveVector::North => (pos.0, pos.1 + 1),
            MoveVector::East => (pos.0 + 1, pos.1),
            MoveVector::South => (pos.0 - 1, pos.1),
            MoveVector::West => (pos.0, pos.1 - 1),
            MoveVector::None => pos,
        }
    }

    fn determine_move_action(
        &self,
        pos: &(i64, i64),
        range: &Range<u64>,
        map_instance: &MapInstance,
    ) -> BoundaryCheck {
        if pos.0 < 0 || pos.1 < 0 || pos.0 > map_instance.map.chunk_len - 1 {
            return BoundaryCheck::Impossible;
        }
        let offset = map_instance.pos_to_offset(pos.0, pos.1) as u64;
        if range.contains(&offset) {
            return BoundaryCheck::InBoundary;
        } else {
            let owner = map_instance
                .map
                .owners
                .keys()
                .find(|&r| r.contains(&offset));

            if owner != None {
                let destination = map_instance.map.owners.get(owner.unwrap()).unwrap();
                let dest_copy = (destination.0.clone(), destination.1, destination.2);
                if destination.0 == map_instance.fragment_owner.0 {
                    return BoundaryCheck::OutBoundaryLocal(dest_copy);
                } else {
                    return BoundaryCheck::OutBoundaryGlobal(dest_copy);
                }
            }
        }
        BoundaryCheck::Impossible
    }

    fn check_for_sheep_at_position(&self, pos: (i64, i64)) -> Option<Uuid> {
        for (k, v) in self.sheep.position.iter() {
            if *v == pos {
                return Some(*k);
            }
        }
        return None;
    }

    fn display_turn_stats(&self) {
        println!("AT THE END OF THE TURN:");
        println!("Number of sheep: {}", self.sheep.id.len());
        println!("Number of wolves: {}", self.wolves.id.len());
    }

    fn display_final_stats(&self) {
        println!("========================== AT THE END OF THE SIMULATION ON ISLAND {}: ========================== ", &self.id.to_string()[..8]);
        println!("Number of sheep: {}", self.sheep.id.len());
        println!("Number of wolves: {}", self.wolves.id.len());
        println!("=====================================================================================================");
    }
}
