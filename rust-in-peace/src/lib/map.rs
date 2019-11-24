use crate::network::{Ip, Port};
use crate::island::IslandEnv;

use uuid::Uuid;
use std::collections::HashMap;
use crate::message::Message;

pub type Fragment = std::ops::Range<u64>;
pub type FragmentOwner = (Ip, Port, Uuid);
pub type MapOwners = HashMap<Fragment, FragmentOwner>;

#[derive(Debug, Clone)]
pub struct Map {
    pub owners: MapOwners,
    chunk_len: u64
}

impl Map {
    pub fn new(chunk_len: u64, owners: MapOwners) -> Self {
        Self {
            chunk_len,
            owners
        }
    }
}

#[derive(Debug, Clone)]
pub struct MapInstance {
    pub map: Map,
    pub data: Vec<u64>,
    pub fragment_owner: FragmentOwner,
}

impl MapInstance {
    pub fn get_instance(island_env: &IslandEnv) -> Self {
        Self {
            data: Vec::with_capacity((island_env.map.chunk_len * island_env.map.chunk_len) as usize),
            map: island_env.map.clone(),
            fragment_owner: island_env.fragment_owner.clone(),
        }
    }

    pub fn set(&mut self, island_env: &mut IslandEnv, x: u64, y: u64, val: u64) {
        let offset = self.pos_to_offset(x, y);
        let range = self.map.owners.keys().find(|&r| r.contains(&offset)).unwrap();

        let (other_ip, other_port, other_island_id) = self.map.owners.get(range).unwrap();
        let (my_ip, my_port, my_island_id) = &self.fragment_owner;

        if other_island_id.to_string() == my_island_id.to_string() {
            self.data[offset as usize] = val;
        } else if other_ip == my_ip {
            // TODO: Sending between islands on host
            island_env.send_to_local(*other_island_id, Message::MapSet(x, y, val));
        } else {
            // TODO: Sending between hosts
            island_env.send_to_global((other_ip.clone(), other_port.clone()), Message::MapSet(x, y, val));
        }
    }

    fn pos_to_offset(&self, x: u64, y: u64) -> u64 {
        y * self.map.chunk_len + x
    }

    fn offset_to_pos(&self, offset: u64) -> (u64, u64) {
        let x = offset % self.map.chunk_len;
        let y = offset / self.map.chunk_len;

        (x, y)
    }

    pub fn get_neighbourhood(&self, x: u64, y: u64) -> Vec<(u64, u64)> {
        unimplemented!();
    }

    pub fn get_value(&self, x: u64, y: u64) -> u64 {
        unimplemented!();
    }

    pub fn update_value(&mut self, x: u64, y: u64) {
        unimplemented!();
    }

    pub fn get_range(&self) {
        unimplemented!();
    }

}