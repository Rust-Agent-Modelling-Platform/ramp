use crate::message::Message;
use crate::network;
use crate::network::DispatcherNetworkCtx;
use rand::{thread_rng, Rng};
use std::sync::mpsc::Receiver;

#[derive(Debug)]
pub enum DispatcherMessage {
    Random(Message),
    Broadcast(Message),
    Info(Message),
}

pub struct Dispatcher {
    rx: Receiver<DispatcherMessage>,
    nt_ctx: DispatcherNetworkCtx,
    islands: u32,
}

impl Dispatcher {
    pub fn new(
        rx: Receiver<DispatcherMessage>,
        nt_ctx: DispatcherNetworkCtx,
        islands: u32,
    ) -> Dispatcher {
        Dispatcher {
            rx,
            nt_ctx,
            islands,
        }
    }

    pub fn start(&self) {
        log::info!("Dispatcher started");
        let mut fin_sim = false;
        let mut confirmations = 0;
        let from = self.nt_ctx.nt_sett.host_ip.clone();
        while !fin_sim {
            let incoming = self.rx.try_iter();
            for msg in incoming {
                match msg {
                    DispatcherMessage::Random(Message::Agent(_)) => {
                        let random_index = thread_rng().gen_range(0, self.nt_ctx.ip_table.len());
                        let (ip, port) = self.nt_ctx.ip_table[random_index];
                        let key = format!("{}:{}", ip.to_string(), port.to_string());

                        network::send_ps(&self.nt_ctx.pub_sock, key, from.clone(), msg.into());
                    }
                    DispatcherMessage::Broadcast(Message::Agent(_)) => {
                        let key = String::from(network::BROADCAST_KEY);
                        network::send_ps(&self.nt_ctx.pub_sock, key, from.clone(), msg.into())
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
            DispatcherMessage::Random(msg) => msg,
            DispatcherMessage::Broadcast(msg) => msg,
            DispatcherMessage::Info(msg) => msg,
        }
    }
}
