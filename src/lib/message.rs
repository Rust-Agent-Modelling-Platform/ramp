use crate::agent::Agent;

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Agent(Agent),
    StartSim,
    FinSim,
    HostReady,
    Ok,
}

impl Message {
    pub fn into_string(&self) -> String {
        match self {
            Message::Agent(agent) => agent.into_string(),
            Message::StartSim => String::from("START SIM"),
            Message::FinSim => String::from("FIN SIM"),
            Message::HostReady => String::from("HOST READY"),
            Message::NextTurn => String::from("NEXT TURN"),
            Message::TurnDone => String::from("TURN DONE"),
            Message::Ok => String::from("OK"),
        }
    }
}
