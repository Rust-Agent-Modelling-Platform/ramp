use crate::agent::Agent;

#[derive(Debug)]
pub enum Message {
    Agent(Agent),
    FinSim,
}
