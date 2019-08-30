use crate::Message;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

type State = bool;

pub struct AddressBook {
    pub addresses: HashMap<Uuid, (Sender<Message>, State)>,
    pub network_thread: Sender<Message>,
    pub rx: Receiver<Message>,
    pub sub_tx: Sender<Message>,
}

impl AddressBook {
    pub fn new(
        addresses: HashMap<Uuid, (Sender<Message>, State)>,
        network_thread: Sender<Message>,
        rx: Receiver<Message>,
        sub_tx: Sender<Message>,
    ) -> AddressBook {
        AddressBook {
            addresses,
            network_thread,
            rx,
            sub_tx,
        }
    }
}
