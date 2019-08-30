use crate::agent::Agent;
use uuid::Uuid;
use std::fmt;

#[derive(Debug)]
pub enum Message {
    Agent(Agent),
    Fin(Uuid),
    FinSim,
}