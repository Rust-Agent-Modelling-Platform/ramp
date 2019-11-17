use crate::network::Ip;
use crate::network::Port;

pub type TurnNumber = u32;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    Agent(Vec<u8>),
    Hello(Ip, Port),
    IpTable(Vec<(Ip, Port)>),
    StartSim,
    FinSim,
    HostReady,
    NextTurn(TurnNumber),
    TurnDone,
    Ok,
    Err,
}

impl Message {
    pub fn as_string(&self) -> String {
        match self {
            Message::Agent(agent_vec) => format!("{:#?}", agent_vec),
            Message::Hello(ip, port) => format!("HELLO FROM {}:{}", ip, port),
            Message::IpTable(table) => format!("IP TABLE {:#?}", table),
            Message::StartSim => String::from("START SIM"),
            Message::FinSim => String::from("FIN SIM"),
            Message::HostReady => String::from("HOST READY"),
            Message::NextTurn(turn_number) => format!("NEXT TURN ({})", turn_number),
            Message::TurnDone => String::from("TURN DONE"),
            Message::Ok => String::from("OK"),
            Message::Err => String::from("ERROR"),
        }
    }
}
