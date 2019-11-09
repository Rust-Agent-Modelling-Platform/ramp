use crate::dispatcher::DispatcherMessage;
use rand::{thread_rng, Rng};
use std::sync::mpsc::{Receiver, Sender};

use uuid::Uuid;

use crate::message::Message;

#[derive(Debug)]
pub struct SendError<Message>(pub Message);

pub struct AddressBook {
    pub self_rx: Receiver<Message>,
    pub dispatcher_tx: Sender<DispatcherMessage>,
    pub addresses: Vec<Sender<Message>>,
    pub islands: Vec<Uuid>,
}

impl AddressBook {
    pub fn new(
        self_rx: Receiver<Message>,
        dispatcher_tx: Sender<DispatcherMessage>,
        addresses: Vec<Sender<Message>>,
        islands: Vec<Uuid>,
    ) -> AddressBook {
        AddressBook {
            self_rx,
            dispatcher_tx,
            addresses,
            islands,
        }
    }

    pub fn receive_messages(&self) -> Vec<Message> {
        self.self_rx.try_iter().collect()
    }

    pub fn send_to_rnd_local(&mut self, msg: Message) -> Result<(), SendError<Message>> {
        let island = thread_rng().gen_range(0, self.addresses.len());
        match self.addresses.get(island) {
            Some(tx) => match tx.send(msg) {
                Ok(()) => Ok(()),
                Err(e) => {
                    self.addresses.remove(island);
                    self.islands.remove(island);
                    self.send_to_rnd_local(e.0)
                }
            },
            None => Err(SendError(msg)),
        }
    }

    pub fn send_to_all_local(&mut self, msg: Message) -> Result<(), SendError<Message>> {
        let mut counter = 0;
        let mut id_to_remove = vec![];

        for i in 0..self.addresses.len() {
            if let Some(tx) = self.addresses.get(i) {
                match tx.send(msg.clone()) {
                    Ok(()) => counter += 1,
                    Err(_) => id_to_remove.push(i),
                }
            }
        }

        id_to_remove.iter().for_each(|index| {
            self.addresses.remove(*index);
            self.islands.remove(*index);
        });

        if counter == 0 {
            Err(SendError(msg))
        } else {
            Ok(())
        }
    }

    pub fn send_to_rnd_global(&mut self, msg: Message) {
        self.dispatcher_tx.send(DispatcherMessage::Random(msg)).unwrap();
    }

    pub fn send_to_all_global(&mut self, msg: Message) {
        self.dispatcher_tx.send(DispatcherMessage::Broadcast(msg)).unwrap();
    }
}
