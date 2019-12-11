use crate::dispatcher::{Addr, DispatcherMessage};
use rand::{thread_rng, Rng};
use std::sync::mpsc::Sender;

use uuid::Uuid;

use crate::message::Message;

#[derive(Debug)]
pub struct SendError<Message>(pub Message);

pub struct AddressBook {
    pub dispatcher_tx: Sender<DispatcherMessage>,
    pub addresses: Vec<Sender<Message>>,
    pub islands: Vec<Uuid>,
}

impl AddressBook {
    pub fn new(
        dispatcher_tx: Sender<DispatcherMessage>,
        addresses: Vec<Sender<Message>>,
        islands: Vec<Uuid>,
    ) -> AddressBook {
        AddressBook {
            dispatcher_tx,
            addresses,
            islands,
        }
    }

    pub fn send_to_rnd_local(&mut self, msg: Message) -> Result<(), SendError<Message>> {
        if self.addresses.is_empty() {
            return Err(SendError(msg));
        }
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

    pub fn send_to_local(
        &mut self,
        island_id: Uuid,
        msg: Message,
    ) -> Result<(), SendError<Message>> {
        let island = self.islands.iter().position(|&id| id == island_id).unwrap();
        match self.addresses.get(island) {
            Some(tx) => match tx.send(msg) {
                Ok(()) => Ok(()),
                Err(e) => Err(SendError(e.0)),
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

    pub fn send_to_global(&mut self, addr: Addr, msg: Message) {
        self.dispatcher_tx
            .send(DispatcherMessage::Unicast(msg, addr))
            .unwrap();
    }

    pub fn send_to_rnd_global(&mut self, msg: Message) {
        self.dispatcher_tx
            .send(DispatcherMessage::UnicastRandom(msg))
            .unwrap();
    }

    pub fn send_to_all_global(&mut self, msg: Message) {
        self.dispatcher_tx
            .send(DispatcherMessage::Broadcast(msg))
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::AddressBook;
    use crate::message::Message;
    use std::sync::mpsc;
    use uuid::Uuid;

    #[test]
    fn test_send_to_rnd_local() -> Result<(), ()> {
        let (dispatcher_tx, _dispatcher_rx) = mpsc::channel();
        let (tx1, rx1) = mpsc::channel();
        let addresses = vec![tx1];
        let islands = vec![Uuid::new_v4()];

        let mut address_book = AddressBook::new(dispatcher_tx, addresses, islands);
        address_book.send_to_rnd_local(Message::Ok).unwrap();
        if let Some(Message::Ok) = rx1.try_iter().next() {
            Ok(())
        } else {
            Err(())
        }
    }

    #[test]
    fn test_send_to_local() -> Result<(), ()> {
        let (dispatcher_tx, _dispatcher_rx) = mpsc::channel();
        let (tx1, rx1) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();
        let (tx3, rx3) = mpsc::channel();
        let addresses = vec![tx1, tx2, tx3];
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let islands = vec![id1, id2, id3];

        let mut address_book = AddressBook::new(dispatcher_tx, addresses, islands);
        address_book.send_to_local(id1, Message::Ok).unwrap();
        address_book.send_to_local(id3, Message::Ok).unwrap();

        let mut counter = 0;
        if let Some(Message::Ok) = rx1.try_iter().next() {
            counter += 1;
        }
        if let Some(Message::Ok) = rx3.try_iter().next() {
            counter += 1;
        }
        if let None = rx2.try_iter().next() {
            counter += 1;
        }

        if counter == 3 {
            Ok(())
        } else {
            Err(())
        }
    }

    #[test]
    fn test_send_to_all_local() -> Result<(), ()> {
        let (dispatcher_tx, _dispatcher_rx) = mpsc::channel();
        let (tx1, rx1) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();
        let addresses = vec![tx1, tx2];
        let islands = vec![Uuid::new_v4(), Uuid::new_v4()];

        let mut address_book = AddressBook::new(dispatcher_tx, addresses, islands);
        address_book.send_to_all_local(Message::Ok).unwrap();

        let mut counter = 0;
        if let Some(Message::Ok) = rx1.try_iter().next() {
            counter += 1;
        }
        if let Some(Message::Ok) = rx2.try_iter().next() {
            counter += 1;
        }

        if counter == 2 {
            Ok(())
        } else {
            Err(())
        }
    }
}
