use crate::message::Message;
use crate::network;
use crate::settings::ClientSettings;
use rand::{thread_rng, Rng};
use std::net::IpAddr;
use std::sync::mpsc::Receiver;
use zmq::Socket;

type Port = u32;

#[derive(Debug)]
pub enum DispatcherMessage {
    Random(Message),
    Broadcast(Message),
    Info(Message),
}

pub struct Dispatcher {
    rx: Receiver<DispatcherMessage>,
    pub_sock: Socket,
    ip_table: Vec<(IpAddr, Port)>,
    settings: ClientSettings,
    srv_req_sock: Socket,
}

impl Dispatcher {
    pub fn create(
        rx: Receiver<DispatcherMessage>,
        pub_sock: Socket,
        ip_table: Vec<(IpAddr, Port)>,
        settings: ClientSettings,
        srv_req_sock: Socket,
    ) -> Dispatcher {
        Dispatcher {
            rx,
            pub_sock,
            ip_table,
            settings,
            srv_req_sock,
        }
    }

    pub fn start(&self) {
        log::info!("Dispatcher started");
        let mut fin_sim = false;
        let mut confirmations = 0;
        let from = self.settings.network.host_ip.clone();
        while !fin_sim {
            let incoming = self.rx.try_iter();
            for msg in incoming {
                match msg {
                    DispatcherMessage::Random(Message::Agent(_)) => {
                        let random_index = thread_rng().gen_range(0, self.ip_table.len());
                        let (ip, port) = self.ip_table[random_index];
                        let key = format!("{}:{}", ip.to_string(), port.to_string());

                        network::send_ps(&self.pub_sock, key, from.clone(), msg.into());
                    }
                    DispatcherMessage::Broadcast(Message::Agent(_)) => {
                        let key = String::from(network::BROADCAST_KEY);
                        network::send_ps(&self.pub_sock, key, from.clone(), msg.into())
                    }
                    DispatcherMessage::Info(Message::TurnDone) => {
                        confirmations += 1;
                        if confirmations == self.settings.islands {
                            network::send_rr(&self.srv_req_sock, from.clone(), Message::TurnDone);
                            let (_, _) = network::recv_rr(&self.srv_req_sock);
                            confirmations = 0;
                        }
                    }
                    DispatcherMessage::Info(Message::FinSim) => {
                        log::info!("Finishing simulation in dispatcher ");
                        fin_sim = true;
                        break;
                    }
                    _ => log::warn!("Unexpected msg in dispatcher {:#?}", msg),
                }
            }
        }
        log::info!("Dispatcher finished")
    }
}

impl Into<Message> for DispatcherMessage {
    fn into(self) -> Message {
        match self {
            DispatcherMessage::Random(msg) => msg,
            DispatcherMessage::Broadcast(msg) => msg,
            DispatcherMessage::Info(msg) => msg,
        }
    }
}
