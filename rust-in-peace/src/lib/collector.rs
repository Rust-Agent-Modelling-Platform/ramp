use crate::address_book::AddressBook;
use crate::dispatcher::DispatcherMessage;
use crate::message::Message;
use crate::metrics;
use crate::network;
use std::sync::mpsc::Receiver;
use zmq::Socket;

pub struct Collector {
    rx: Receiver<Message>,
    sub_sock: Socket,
    address_book: AddressBook,
    identity: String,
}

impl Collector {
    pub fn create(
        self_rx: Receiver<Message>,
        sub_sock: Socket,
        address_book: AddressBook,
        identity: String,
    ) -> Collector {
        Collector {
            rx: self_rx,
            sub_sock,
            address_book,
            identity,
        }
    }

    pub fn start(&mut self) {
        log::info!("Starting receiver thread");
        let mut fin_sim = false;
        while !fin_sim {
            let incoming = self.rx.try_iter();
            for msg in incoming {
                match msg {
                    Message::FinSim => {
                        log::info!("Finishing simulation in receiver thread");
                        if self.address_book.send_to_all_local(msg).is_err() {
                            log::info!("Ilands already finished");
                        }
                        fin_sim = true;
                        break;
                    }
                    _ => log::warn!("Unexpected message in receiver thread {:#?}", msg),
                }
            }

            //Next step: non-blocking check if there are any new agents waiting to be added to our system
            let mut items = [self.sub_sock.as_poll_item(zmq::POLLIN)];
            zmq::poll(&mut items, -1).unwrap();
            if items[0].is_readable() {
                let (_, from, msg) = network::recv_ps(&self.sub_sock);
                metrics::inc_received_messages(
                    from.clone(),
                    self.identity.clone(),
                    String::from("200"),
                );
                match msg {
                    Message::NextTurn(_) => {
                        if self.address_book.send_to_all_local(msg).is_err() {
                            log::error!("No more active ilands while sending NextTurn msg");
                        }
                    }
                    Message::FinSim => {
                        log::info!("Finishing simulation in receiver thread");
                        if self.address_book.send_to_all_local(msg.clone()).is_err() {
                            log::error!("No more active ilands while sending FinSim msg");
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
                    _ => log::warn!("Unexpected message in receiver thread {:#?}", msg),
                }
            }
        }
        log::info!("Receiver thread finished");
    }
}
