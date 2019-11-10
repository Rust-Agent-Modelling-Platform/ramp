use crate::message::Message;
use zmq::Socket;

type Key = String;
type From = String;

pub const COORD_INFO_KEY: &str = "COORD_INFO";
pub const SERVER_INFO_KEY: &str = "SERVER_INFO";
pub const BROADCAST_KEY: &str = "BROADCAST";

pub fn connect_sock(sock: &Socket, ip: String, port: u32) {
    let address = &format!("tcp://{}:{}", ip, port);
    assert!(sock.connect(address).is_ok());
}

pub fn bind_sock(sock: &Socket, ip: String, port: u32) {
    let endpoint = &format!("tcp://{}:{}", ip, port);
    assert!(sock.bind(endpoint).is_ok());
}

pub fn subscribe_sock(sock: &Socket, key: String) {
    sock.set_subscribe(key.as_bytes()).unwrap();
}

/// Sends ['Message'] in REQ-REP pattern. First is
/// sender identity and next is msg. Sender identity should
/// be its ip address.
pub fn send_rr(sock: &Socket, from: From, msg: Message) {
    let s_from = from.into_bytes();
    let s_msg = bincode::serialize(&msg).unwrap();
    sock.send(s_from, zmq::SNDMORE).unwrap();
    sock.send(s_msg, 0).unwrap();
}

/// Sends ['Message'] in PUB-SUB pattern. First is key, next is
/// sender identity and the last one is msg. Sender identity should
/// be its ip address.
pub fn send_ps(sock: &Socket, key: Key, from: From, msg: Message) {
    let s_key = key.into_bytes();
    let s_from = from.into_bytes();
    let s_msg = bincode::serialize(&msg).unwrap();
    sock.send(s_key, zmq::SNDMORE).unwrap();
    sock.send(s_from, zmq::SNDMORE).unwrap();
    sock.send(s_msg, 0).unwrap();
}

/// Receives ['Message'] in REQ-REP pattern. Analogous to ['send_rr'].
pub fn recv_rr(sock: &Socket) -> (From, Message) {
    let from = sock.recv_string(0).unwrap().unwrap();
    let msg = sock.recv_bytes(0).unwrap();
    let d_msg: Message = bincode::deserialize(&msg).unwrap();
    (from, d_msg)
}

/// Receives ['Message'] in PUB-SUB pattern. Analogous to ['send_ps'].
pub fn recv_ps(sock: &Socket) -> (Key, From, Message) {
    let key = sock.recv_string(0).unwrap().unwrap();
    let from = sock.recv_string(0).unwrap().unwrap();
    let msg = sock.recv_bytes(0).unwrap();
    let d_msg: Message = bincode::deserialize(&msg).unwrap();
    (key, from, d_msg)
}
