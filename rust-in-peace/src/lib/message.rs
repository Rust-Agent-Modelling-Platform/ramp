use uuid::Uuid;

use crate::map::MapOwners;
use crate::network::{Ip, Port};

pub type TurnNumber = u32;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    Islands(Vec<Uuid>),
    Owners(MapOwners),
    MapSet(u64, u64, i32),
    MapGet(u64, u64, i32),
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
            Message::Islands(island_ids) => format!("({:#?})", island_ids),
            Message::Owners(owners) => format!("MAP OWNERS {:#?}", owners),
            Message::MapSet(x, y, value) => format!("MAP SET ({}, {}) -> {}", x, y, value),
            Message::MapGet(x, y, value) => format!("MAP GET ({}, {}) -> {}", x, y, value),
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
