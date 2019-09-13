use zmq::Socket;
use crate::message::Message;

type Key = String;
type From = String;

pub fn connect_sock(sock: &Socket, ip: String, port: u32) {
    let address = &format!("tcp://{}:{}",ip, port);
    assert!(sock.connect(address).is_ok());
}

pub fn bind_sock(sock: &Socket, ip: String, port: u32) {
    let endpoint = &format!("tcp://{}:{}", ip, port);
    assert!(sock.bind(endpoint).is_ok());
}

pub fn subscribe_sock(sock: &Socket, key: String) {
    sock.set_subscribe(key.as_bytes()).unwrap();
}

pub fn send1(sock: &Socket, from: From, msg: Message) {
    let s_from = from.into_bytes();
    let s_msg = bincode::serialize(&msg).unwrap();
    sock.send(s_from, zmq::SNDMORE).unwrap();
    sock.send(s_msg, 0).unwrap();
}

pub fn send2(sock: &Socket, key: Key, from: From, msg: Message) {
    let s_key = key.into_bytes();
    let s_from = from.into_bytes();
    let s_msg = bincode::serialize(&msg).unwrap();
    sock.send(s_key, zmq::SNDMORE).unwrap();
    sock.send(s_from, zmq::SNDMORE).unwrap();
    sock.send(s_msg, 0).unwrap();
}

pub fn recv1(sock: &Socket) -> (From, Message) {
    let from = sock.recv_string(0).unwrap().unwrap();
    let msg = sock.recv_bytes(0).unwrap();
    let d_msg: Message = bincode::deserialize(&msg).unwrap();
    (from, d_msg)
}

pub fn recv2(sock: &Socket) -> (Key, From, Message) {
    let key = sock.recv_string(0).unwrap().unwrap();
    let from = sock.recv_string(0).unwrap().unwrap();
    let msg = sock.recv_bytes(0).unwrap();
    let d_msg: Message = bincode::deserialize(&msg).unwrap();
    (key, from, d_msg)
}