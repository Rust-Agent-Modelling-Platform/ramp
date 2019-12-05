use crate::message::Message;
use crate::network;
use crate::network::{DispatcherNetworkCtx, Ip, Port};
use rand::{thread_rng, Rng};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

pub type Addr = (Ip, Port);

#[derive(Debug)]
pub enum DispatcherMessage {
    UnicastRandom(Message),
    Unicast(Message, Addr),
    Broadcast(Message),
    Info(Message),
    Server(Message),
}

pub struct Dispatcher {
    rx: Receiver<DispatcherMessage>,
    nt_ctx: DispatcherNetworkCtx,
    islands: u32,
    sim_tx: Sender<Message>,
}

impl Dispatcher {
    pub fn new(
        rx: Receiver<DispatcherMessage>,
        nt_ctx: DispatcherNetworkCtx,
        islands: u32,
        sim_tx: Sender<Message>,
    ) -> Dispatcher {
        Dispatcher {
            rx,
            nt_ctx,
            islands,
            sim_tx,
        }
    }

    pub fn start(&self) {
        log::info!("Dispatcher started");
        let mut fin_sim = false;
        let mut confirmations = 0;
        let from = self.nt_ctx.nt_sett.host_ip.clone();
        self.sim_tx
            .send(Message::Ok)
            .expect("Error sending to sim_tx");
        while !fin_sim {
            let incoming = self.rx.try_iter();
            for msg in incoming {
                match msg {
                    DispatcherMessage::UnicastRandom(Message::Agent(_)) => {
                        let random_index = thread_rng().gen_range(0, self.nt_ctx.ip_table.len());
                        let (ip, port) = &self.nt_ctx.ip_table[random_index];
                        let key = format!("{}:{}", ip, port);

                        network::send_ps(
                            &self.nt_ctx.pub_sock,
                            key.clone(),
                            from.clone(),
                            msg.into(),
                        );
                    }
                    DispatcherMessage::Unicast(msg, addr) => {
                        let key = format!("{}:{}", addr.0, addr.1);
                        network::send_ps(&self.nt_ctx.pub_sock, key, from.clone(), msg)
                    }
                    DispatcherMessage::Broadcast(Message::Agent(_)) => {
                        let key = String::from(network::BROADCAST_KEY);
                        network::send_ps(&self.nt_ctx.pub_sock, key, from.clone(), msg.into())
                    }
                    DispatcherMessage::Broadcast(Message::Islands(island_ids)) => {
                        log::info!("ISLANDS MSG");
                        let key = String::from(network::BROADCAST_KEY);
                        network::send_ps(
                            &self.nt_ctx.pub_sock,
                            key.clone(),
                            from.clone(),
                            Message::Islands(island_ids.clone()),
                        );
                    }
                    DispatcherMessage::Broadcast(Message::Owners(_)) => {
                        log::info!("OWNERS MSG");
                        let key = String::from(network::BROADCAST_KEY);
                        network::send_ps(&self.nt_ctx.pub_sock, key, from.clone(), msg.into())
                    }
                    DispatcherMessage::Info(Message::HostReady) => {
                        network::send_rr(&self.nt_ctx.s_req_sock, from.clone(), Message::HostReady);
                        let (_, _) = network::recv_rr(&self.nt_ctx.s_req_sock);
                    }

                    DispatcherMessage::Info(Message::TurnDone) => {
                        confirmations += 1;
                        if confirmations == self.islands {
                            network::send_rr(
                                &self.nt_ctx.s_req_sock,
                                from.clone(),
                                Message::TurnDone,
                            );
                            let (_, _) = network::recv_rr(&self.nt_ctx.s_req_sock);
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
            DispatcherMessage::UnicastRandom(msg) => msg,
            DispatcherMessage::Unicast(msg, _addr) => msg,
            DispatcherMessage::Broadcast(msg) => msg,
            DispatcherMessage::Info(msg) => msg,
            DispatcherMessage::Server(msg) => msg,
        }
    }
}
