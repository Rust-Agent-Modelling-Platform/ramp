use crate::agent::Agent;
use uuid::Uuid;

pub enum Message {
    Agent(Agent),
    Fin(Uuid),
    FinSim,
}
