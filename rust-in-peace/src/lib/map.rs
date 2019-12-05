use crate::island::IslandEnv;
use crate::network::{Ip, Port};

use crate::message::Message;
use std::collections::HashMap;
use std::convert::TryInto;
use uuid::Uuid;

pub type Fragment = std::ops::Range<u64>;
pub type FragmentOwner = (Ip, Port, Uuid);
pub type MapOwners = HashMap<Fragment, FragmentOwner>;

#[derive(Debug, Clone)]
pub struct Map {
    pub owners: MapOwners,
    pub chunk_len: i64,
}

impl Map {
    pub fn new(chunk_len: i64, owners: MapOwners) -> Self {
        Self { chunk_len, owners }
    }
}

#[derive(Debug, Clone)]
pub struct MapInstance {
    pub map: Map,
    pub data: Vec<i32>,
    pub fragment_owner: FragmentOwner,
}

impl MapInstance {
    pub fn get_instance(island_env: &IslandEnv) -> Self {
        Self {
            data: Vec::with_capacity(
                (island_env.map.chunk_len * island_env.map.chunk_len) as usize,
            ),
            map: island_env.map.clone(),
            fragment_owner: island_env.fragment_owner.clone(),
        }
    }

    pub fn set(&mut self, island_env: &mut IslandEnv, x: i64, y: i64, val: i32) {
        let offset = self.pos_to_offset(x, y);
        let range = self
            .map
            .owners
            .keys()
            .find(|&r| r.contains(&offset.try_into().unwrap()))
            .unwrap();

        let (other_ip, other_port, other_island_id) = self.map.owners.get(range).unwrap();
        let (my_ip, _my_port, my_island_id) = &self.fragment_owner;

        if other_island_id.to_string() == my_island_id.to_string() {
            self.data[offset as usize] = val;
        } else if other_ip == my_ip {
            // TODO: Sending between islands on host
            island_env
                .send_to_local(*other_island_id, Message::MapSet(x, y, val))
                .expect("Error sending to local in map");
        } else {
            // TODO: Sending between hosts
            island_env.send_to_global((other_ip.clone(), *other_port), Message::MapSet(x, y, val));
        }
    }

    pub fn pos_to_offset(&self, x: i64, y: i64) -> i64 {
        y * self.map.chunk_len + x
    }

    pub fn offset_to_pos(&self, offset: i64) -> (i64, i64) {
        let x = offset % self.map.chunk_len;
        let y = offset / self.map.chunk_len;

        (x, y)
    }

    pub fn init_with_val(&mut self, val: i32) {
        for i in 0..self.map.chunk_len * self.map.chunk_len {
            self.data.insert(i as usize, val);
        }
    }

    pub fn get_my_range(&self) -> Fragment {
        let (_, _, my_island_id) = &self.fragment_owner;
        let range = self
            .map
            .owners
            .keys()
            .find(|&k| self.map.owners.get(k).unwrap().2 == *my_island_id)
            .unwrap();
        range.clone()
    }

    pub fn get_value(&self, pos: (i64, i64)) -> i32 {
        let offset = self.pos_to_offset(pos.0, pos.1);
        let range_start = self.get_my_range().start;
        let scale = self.map.chunk_len.pow(2);
        let local_index = if range_start == 0 {
            offset
        } else {
            offset - (range_start as i64 / scale) * scale
        };
        if self.data.get(local_index as usize) == None {
            -1
        } else {
            *self.data.get(local_index as usize).unwrap()
        }
    }

    pub fn update_value(&mut self, new_val: i32, pos: (i64, i64)) {
        let offset = self.pos_to_offset(pos.0, pos.1);
        let range_start = self.get_my_range().start;
        let scale = self.map.chunk_len.pow(2);
        let local_index = if range_start == 0 {
            offset
        } else {
            offset - (range_start as i64 / scale) * scale
        };
        log::info!("The local index for this position is {}", local_index);
        *self.data.get_mut(local_index as usize).unwrap() = new_val;
    }

    //    pub fn get_neighbourhood(&self, x: u64, y: u64) -> Vec<(u64, u64)> {
    //        unimplemented!();
    //    }
}
