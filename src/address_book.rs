use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

use uuid::Uuid;

use crate::Message;

type State = bool;

#[derive(Debug)]
pub struct SendError<Message>(pub Message);

pub struct AddressBook {
    pub self_rx: Receiver<Message>,
    pub addresses: HashMap<Uuid, (Sender<Message>, State)>,
    pub pub_rx: Sender<Message>,
}

impl AddressBook {
    pub fn new(
        self_rx: Receiver<Message>,
        addresses: HashMap<Uuid, (Sender<Message>, State)>,
        pub_rx: Sender<Message>,
    ) -> AddressBook {
        AddressBook {
            self_rx,
            addresses,
            pub_rx,
        }
    }

    /// Tries to send [`Message`] to random island. If no island is active
    /// [`SendError`] with [`Message`] will be returned.
    pub fn send_to_rnd(&mut self, msg: Message) -> Result<(), SendError<Message>> {
        match self
            .addresses
            .iter_mut()
            .find(|&(_, (_, mut state))| state)
        {
            Some((_island_uuid, (tx, state))) => match tx.send(msg) {
                Ok(()) => Ok(()),
                Err(e) => {
                    *state = false;
                    self.send_to_rnd(e.0)
                }
            },
            None => Err(SendError(msg)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender};

    use uuid::Uuid;

    use crate::address_book::{AddressBook, State};
    use crate::message::Message;

    #[test]
    fn send_to_rnd_when_there_is_active_island_stress() -> Result<(), String> {
        let (tx, rx) = mpsc::channel();
        let mut addresses: HashMap<Uuid, (Sender<Message>, State)> = HashMap::new();
        addresses.insert(Uuid::new_v4(), (tx.clone(), true));
        let mut address_book: AddressBook = AddressBook {
            self_rx: rx,
            addresses,
            pub_rx: tx.clone(),
        };
        match address_book.send_to_rnd(Message::FinSim) {
            Ok(()) => Ok(()),
            Err(_) => Err(String::from("send_to_rnd")),
        }
    }

    #[test]
    fn send_to_rnd_when_there_is_no_active_island_stress() -> Result<(), String> {
        let (tx, rx) = mpsc::channel();
        let mut addresses: HashMap<Uuid, (Sender<Message>, State)> = HashMap::new();
        addresses.insert(Uuid::new_v4(), (tx.clone(), false));
        let mut address_book: AddressBook = AddressBook {
            self_rx: rx,
            addresses,
            pub_rx: tx.clone(),
        };
        match address_book.send_to_rnd(Message::FinSim) {
            Ok(()) => Err(String::from("send_to_rnd")),
            Err(_) => Ok(()),
        }
    }

    #[test]
    fn send_to_rnd_island_state_update_stress() -> Result<(), String> {
        let (tx, rx) = mpsc::channel();
        let (_, rx_stub) = mpsc::channel();
        let mut addresses: HashMap<Uuid, (Sender<Message>, State)> = HashMap::new();
        addresses.insert(Uuid::new_v4(), (tx.clone(), true));
        let mut address_book: AddressBook = AddressBook {
            self_rx: rx_stub,
            addresses,
            pub_rx: tx.clone(),
        };
        drop(rx);
        match address_book.send_to_rnd(Message::FinSim) {
            Ok(()) => Err(String::from("send_to_rnd")),
            Err(_) => Ok(()),
        }
    }
}
