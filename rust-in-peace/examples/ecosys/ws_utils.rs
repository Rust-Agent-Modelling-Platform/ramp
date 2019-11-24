
use uuid::Uuid;
use std::ops::Range;
use rand::Rng;
use crate::agent_types::AgentType;

type Position = (u64, u64);

#[derive(Serialize, Deserialize, Debug)]
pub struct SerializedAgent {
    pub agent_type: AgentType,
    pub id: Uuid,
    pub energy: u64,
    pub position: Position,
}

pub fn deserialize(agent: Vec<u8>) -> (AgentType, Uuid, u64, Position) {
    let d: SerializedAgent = bincode::deserialize(&agent).unwrap();
    (d.agent_type, d.id, d.energy, d.position)
}

pub fn generate_random_position(range: &Range<u64>) -> Position {
    let mut rng = rand::thread_rng();
    (rng.gen_range(range.start, range.end), rng.gen_range(range.start, range.end))
}