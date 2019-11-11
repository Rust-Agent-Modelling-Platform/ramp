use crate::message::Message;
use crate::settings::NetworkSettings;
use std::net::IpAddr;
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

//////////////////////////////////////////////////////////////////////////////
////                          Network Context                             ////
//////////////////////////////////////////////////////////////////////////////

type Port = u32;

pub struct NetworkCtx {
    pub private_key: String,
    pub ip_table: Vec<(IpAddr, Port)>,
    pub settings: NetworkSettings,
    pub req_sock: Socket,
    pub rep_sock: Socket,
    pub pub_sock: Socket,
    pub sub_sock: Socket,
    pub s_req_sock: Socket,
}

pub struct DispatcherNetworkCtx {
    pub nt_sett: NetworkSettings,
    pub ip_table: Vec<(IpAddr, Port)>,
    pub pub_sock: Socket,
    pub s_req_sock: Socket,
}

pub struct CollectorNetworkCtx {
    pub nt_sett: NetworkSettings,
    pub sub_sock: Socket,
}

impl NetworkCtx {
    pub fn new(settings: NetworkSettings) -> Self {
        let private_key = Self::create_private_key(&settings);
        let ip_table: Vec<(IpAddr, Port)> = Self::parse_input_ips(&settings);
        let context = zmq::Context::new();
        let rep_sock = context.socket(zmq::REP).unwrap();
        let req_sock = context.socket(zmq::REQ).unwrap();
        let s_req_sock = context.socket(zmq::REQ).unwrap();
        let pub_sock = context.socket(zmq::PUB).unwrap();
        let sub_sock = context.socket(zmq::SUB).unwrap();
        NetworkCtx {
            private_key,
            ip_table,
            settings,
            req_sock,
            rep_sock,
            pub_sock,
            sub_sock,
            s_req_sock,
        }
    }

    pub fn init(self) -> (DispatcherNetworkCtx, CollectorNetworkCtx) {
        let host_ip = self.settings.host_ip.clone();
        let host_pub_port = self.settings.pub_port;
        bind_sock(&self.pub_sock, host_ip.clone(), host_pub_port);
        self.connect();
        self.subscribe();

        if self.settings.global_sync.sync {
            let server_ip = self.settings.global_sync.server_ip.clone();
            let server_rep_port = self.settings.global_sync.server_rep_port;
            let server_pub_port = self.settings.global_sync.server_pub_port;
            connect_sock(&self.s_req_sock, server_ip.clone(), server_rep_port);
            connect_sock(&self.sub_sock, server_ip, server_pub_port);
            subscribe_sock(&self.sub_sock, String::from(SERVER_INFO_KEY));
            self.send_ready_msg(&self.s_req_sock);
        } else if self.settings.is_coordinator {
            let coord_ip = self.settings.coordinator_ip.clone();
            let coord_rep_port = self.settings.coordinator_rep_port;
            bind_sock(&self.rep_sock, coord_ip, coord_rep_port);
            self.wait_for_hosts();
            self.notify_hosts();
        } else {
            let coord_ip = self.settings.coordinator_ip.clone();
            let coord_rep_port = self.settings.coordinator_rep_port;
            connect_sock(&self.req_sock, coord_ip, coord_rep_port);
            self.send_ready_msg(&self.req_sock);
            self.wait_for_signal();
        }

        let dis_nt_ctx = DispatcherNetworkCtx {
            nt_sett: self.settings.clone(),
            ip_table: self.ip_table,
            pub_sock: self.pub_sock,
            s_req_sock: self.s_req_sock,
        };

        let coll_nt_ctx = CollectorNetworkCtx {
            nt_sett: self.settings.clone(),
            sub_sock: self.sub_sock,
        };

        (dis_nt_ctx, coll_nt_ctx)
    }

    fn create_private_key(settings: &NetworkSettings) -> String {
        format!(
            "{}:{}",
            settings.host_ip.to_string(),
            settings.pub_port.to_string()
        )
    }

    fn connect(&self) {
        self.ip_table
            .iter()
            .for_each(|(ip, port)| connect_sock(&self.sub_sock, ip.to_string(), *port));
    }

    fn subscribe(&self) {
        subscribe_sock(&self.sub_sock, self.private_key.clone());
        subscribe_sock(&self.sub_sock, String::from(COORD_INFO_KEY));
        subscribe_sock(&self.sub_sock, String::from(BROADCAST_KEY));
    }

    fn wait_for_hosts(&self) {
        let mut count = 0;
        while count != self.settings.hosts_num {
            let from = self.rep_sock.recv_msg(0).unwrap();
            let msg = self.rep_sock.recv_bytes(0).unwrap();

            let d_msg: Message = bincode::deserialize(&msg).unwrap();
            log::info!("{} {}", d_msg.as_string(), from.as_str().unwrap());

            let from = self.settings.host_ip.clone();
            let msg = Message::Ok;
            send_rr(&self.rep_sock, from, msg);
            count += 1;
        }
    }

    fn notify_hosts(&self) {
        log::info!("Notifying hosts");
        let key = String::from(COORD_INFO_KEY);
        let from = self.settings.host_ip.clone();
        let msg = Message::StartSim;

        send_ps(&self.pub_sock, key, from, msg);
    }

    fn send_ready_msg(&self, sock: &Socket) {
        log::info!("Sending host ready message");
        let from = self.settings.host_ip.clone();
        let msg = Message::HostReady;

        send_rr(sock, from, msg);
        let (_, msg) = recv_rr(sock);
        log::info!("{}", msg.as_string());
    }

    fn parse_input_ips(settings: &NetworkSettings) -> Vec<(IpAddr, Port)> {
        let mut ips: Vec<(IpAddr, Port)> = Vec::new();
        let ips_str: Vec<String> = settings.ips.clone();
        for address in ips_str {
            let split: Vec<&str> = address.split(':').collect();
            ips.push((split[0].parse().unwrap(), split[1].parse().unwrap()));
        }
        ips
    }

    fn wait_for_signal(&self) {
        log::info!("Waiting for signal to start sim");
        let (_, _, msg) = recv_ps(&self.sub_sock);
        log::info!("{}", msg.as_string());
    }
}
