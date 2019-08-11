use crate::Message;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

type State = bool;

pub struct AddressBook {
    pub addresses: HashMap<Uuid, (Sender<Message>, State)>,
    pub rx: Receiver<Message>,
    pub designated_island_id: Uuid,
}

impl AddressBook {
    pub fn new(
        addresses: HashMap<Uuid, (Sender<Message>, State)>,
        rx: Receiver<Message>,
        designated_island_id: Uuid,
    ) -> AddressBook {
        AddressBook {
            addresses,
            rx,
            designated_island_id,
        }
    }
}
