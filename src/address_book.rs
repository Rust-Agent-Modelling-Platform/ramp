use crate::Message;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

type State = bool;

pub struct AddressBook {
    pub addresses: HashMap<Uuid, (Sender<Message>, State)>,
    pub rx: Receiver<Message>,
}

impl AddressBook {
    pub fn new(
        addresses: HashMap<Uuid, (Sender<Message>, State)>,
        rx: Receiver<Message>,
    ) -> AddressBook {
        AddressBook { addresses, rx }
    }
}
