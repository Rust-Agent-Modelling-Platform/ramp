pub type TurnNumber = u32;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    Agent(Vec<u8>),
    StartSim,
    FinSim,
    HostReady,
    NextTurn(TurnNumber),
    TurnDone,
    Ok,
}

impl Message {
    pub fn as_string(&self) -> String {
        match self {
            Message::Agent(agent_vec) => format!("{:#?}", agent_vec),
            Message::StartSim => String::from("START SIM"),
            Message::FinSim => String::from("FIN SIM"),
            Message::HostReady => String::from("HOST READY"),
            Message::NextTurn(turn_number) => format!("NEXT TURN ({})", turn_number),
            Message::TurnDone => String::from("TURN DONE"),
            Message::Ok => String::from("OK"),
        }
    }
}
