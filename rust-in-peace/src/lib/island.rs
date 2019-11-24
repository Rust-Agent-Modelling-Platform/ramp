use crate::address_book::SendError;
use crate::message::Message;
use std::sync::Arc;

use uuid::Uuid;

use crate::address_book::AddressBook;
use crate::metrics::MetricHub;
use std::time::Instant;

pub struct IslandEnv {
    address_book: AddressBook,
    pub metric_hub: Arc<MetricHub>,
    pub start_time: Instant,
}

impl IslandEnv {
    pub fn new(
        address_book: AddressBook,
        metric_hub: Arc<MetricHub>,
        start_time: Instant,
    ) -> IslandEnv {
        IslandEnv {
            address_book,
            metric_hub,
            start_time,
        }
    }

    pub fn send_to_rnd_local(&mut self, msg: Message) -> Result<(), SendError<Message>> {
        self.address_book.send_to_rnd_local(msg)
    }

    pub fn send_to_all_local(&mut self, msg: Message) -> Result<(), SendError<Message>> {
        self.address_book.send_to_all_local(msg)
    }

    pub fn send_to_rnd_global(&mut self, msg: Message) {
        self.address_book.send_to_rnd_global(msg);
    }

    pub fn send_to_all_global(&mut self, msg: Message) {
        self.address_book.send_to_all_global(msg);
    }

    pub fn get_active_islands_number(&self) -> i32 {
        self.address_book.islands.len() as i32
    }
}

pub trait Island: Send {
    fn on_start(&mut self);

    fn do_turn(&mut self, turn_number: u32, messages: Vec<Message>);

    fn on_finish(&mut self);
}

pub trait IslandFactory {
    fn create(&self, island_id: Uuid, island_env: IslandEnv) -> Box<dyn Island>;
}
