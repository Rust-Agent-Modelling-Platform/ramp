use crate::agent_types::AgentType;
use rand::Rng;
use std::ops::Range;
use uuid::Uuid;

type Position = (i64, i64);

#[derive(Serialize, Deserialize, Debug)]
pub struct SerializedAgent {
    pub agent_type: AgentType,
    pub id: Uuid,
    pub energy: i64,
    pub position: Position,
}

pub fn serialize(agent_type: AgentType, id: Uuid, energy: i64, position: Position) -> Vec<u8> {
    let agent = SerializedAgent {
        agent_type,
        id,
        energy,
        position,
    };
    bincode::serialize(&agent).unwrap()
}

pub fn deserialize(agent: Vec<u8>) -> (AgentType, Uuid, i64, Position) {
    let d: SerializedAgent = bincode::deserialize(&agent).unwrap();
    (d.agent_type, d.id, d.energy, d.position)
}

pub fn generate_random_position(range: &Range<u64>, chunk_len: i64) -> Position {
    let mut rng = rand::thread_rng();
    let offset = rng.gen_range(range.start, range.end) as i64;
    let x = offset % chunk_len;
    let y = offset / chunk_len;
    (x, y)
}
