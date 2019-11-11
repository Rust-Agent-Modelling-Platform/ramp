use crate::address_book::AddressBook;
use crate::dispatcher::DispatcherMessage;
use crate::message::Message;
use crate::metrics;
use crate::network;
use crate::network::CollectorNetworkCtx;
use std::sync::mpsc::Receiver;

pub struct Collector {
    rx: Receiver<Message>,
    nt_ctx: CollectorNetworkCtx,
    address_book: AddressBook,
    identity: String,
}

impl Collector {
    pub fn new(
        self_rx: Receiver<Message>,
        nt_ctx: CollectorNetworkCtx,
        address_book: AddressBook,
    ) -> Collector {
        let identity = nt_ctx.nt_sett.host_ip.clone();
        Collector {
            rx: self_rx,
            nt_ctx,
            address_book,
            identity,
        }
    }

    pub fn start(&mut self) {
        log::info!("Collector started");
        let mut fin_sim = false;
        while !fin_sim {
            let incoming = self.rx.try_iter();
            for msg in incoming {
                match msg {
                    Message::FinSim => {
                        log::info!("Finishing collector");
                        if self.address_book.send_to_all_local(msg).is_err() {
                            log::info!("Islands already finished");
                        }
                        fin_sim = true;
                        break;
                    }
                    _ => log::warn!("Unexpected message in collector {:#?}", msg),
                }
            }

            //Next step: non-blocking check if there are any new agents waiting to be added to our system
            let mut items = [self.nt_ctx.sub_sock.as_poll_item(zmq::POLLIN)];
            zmq::poll(&mut items, -1).unwrap();
            if items[0].is_readable() {
                let (_, from, msg) = network::recv_ps(&self.nt_ctx.sub_sock);
                metrics::inc_received_messages(
                    from.clone(),
                    self.identity.clone(),
                    String::from("200"),
                );
                match msg {
                    Message::NextTurn(_) => {
                        if self.address_book.send_to_all_local(msg).is_err() {
                            log::error!("No more active islands while sending NextTurn msg");
                        }
                    }
                    Message::FinSim => {
                        log::info!("Finishing collector");
                        if self.address_book.send_to_all_local(msg.clone()).is_err() {
                            log::error!("No more active islands while sending FinSim msg");
                        }
                        self.address_book
                            .dispatcher_tx
                            .send(DispatcherMessage::Info(msg))
                            .unwrap();
                        break;
                    }
                    Message::Agent(_) => {
                        if let Err(e) = self.address_book.send_to_rnd_local(msg) {
                            log::info!("{:?} (No more active islands in system)", e);
                        }
                    }
                    _ => log::warn!("Unexpected message in collector {:#?}", msg),
                }
            }
        }
        log::info!("Collector finished");
    }
}
