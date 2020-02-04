use crate::message::Message;
use crate::settings::NetworkSettings;
use zmq::Socket;

type Key = String;
type From = String;

pub const COORD_INFO_KEY: &str = "COORD_INFO";
pub const SERVER_INFO_KEY: &str = "SERVER_INFO";
pub const BROADCAST_KEY: &str = "BROADCAST";

pub fn connect_sock(sock: &Socket, ip: &str, port: u32) {
    let address = &format!("tcp://{}:{}", ip.to_string(), port);
    assert!(sock.connect(address).is_ok());
}

pub fn bind_sock(sock: &Socket, ip: String, port: u32) {
    println!("{}", ip);
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

pub type Ip = String;
pub type Port = u32;

pub struct NetworkCtx {
    pub private_key: String,
    pub settings: NetworkSettings,
    pub req_sock: Socket,
    pub rep_sock: Socket,
    pub pub_sock: Socket,
    pub sub_sock: Socket,
    pub s_req_sock: Socket,
}

pub struct DispatcherNetworkCtx {
    pub nt_sett: NetworkSettings,
    pub ip_table: Vec<(Ip, Port)>,
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
        let context = zmq::Context::new();
        let rep_sock = context.socket(zmq::REP).unwrap();
        let req_sock = context.socket(zmq::REQ).unwrap();
        let s_req_sock = context.socket(zmq::REQ).unwrap();
        let pub_sock = context.socket(zmq::PUB).unwrap();
        let sub_sock = context.socket(zmq::SUB).unwrap();
        NetworkCtx {
            private_key,
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
        self.subscribe();

        let mut ip_table;
        if self.settings.global_sync.sync {
            let server_ip = self.settings.global_sync.server_ip.clone();
            let server_rep_port = self.settings.global_sync.server_rep_port;
            let server_pub_port = self.settings.global_sync.server_pub_port;
            connect_sock(&self.s_req_sock, &server_ip, server_rep_port);
            connect_sock(&self.sub_sock, &server_ip, server_pub_port);
            subscribe_sock(&self.sub_sock, String::from(SERVER_INFO_KEY));
            self.send_hello_msg(&self.s_req_sock);
            ip_table = self.wait_for_ip_table();
            self.connect(&ip_table);
        } else if self.settings.is_coordinator {
            let coord_ip = self.settings.coordinator_ip.clone();
            let coord_rep_port = self.settings.coordinator_rep_port;
            let coord_pub_port = self.settings.coordinator_pub_port;
            bind_sock(&self.rep_sock, coord_ip.clone(), coord_rep_port);
            ip_table = wait_for_hosts(
                &self.rep_sock,
                &self.private_key,
                self.settings.hosts_num,
                false,
            );
            self.connect(&ip_table);
            ip_table.push((coord_ip, coord_pub_port));
            publish_ip_table(&self.pub_sock, &self.private_key, &ip_table);
            wait_for_confirmations(
                &self.rep_sock,
                &self.private_key,
                self.settings.hosts_num,
                false,
            );
            self.publish_start_sim();
        } else {
            let coord_ip = self.settings.coordinator_ip.clone();
            let coord_rep_port = self.settings.coordinator_rep_port;
            let coord_pub_port = self.settings.coordinator_pub_port;
            connect_sock(&self.req_sock, &coord_ip, coord_rep_port);
            connect_sock(&self.sub_sock, &coord_ip, coord_pub_port);
            self.send_hello_msg(&self.req_sock);
            ip_table = self.wait_for_ip_table();
            self.connect(&ip_table);
            self.send_ready_msg(&self.req_sock);
            self.wait_for_signal();
        }

        let dis_nt_ctx = DispatcherNetworkCtx {
            nt_sett: self.settings.clone(),
            ip_table,
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

    fn connect(&self, ip_table: &[(Ip, Port)]) {
        ip_table
            .iter()
            .for_each(|(ip, port)| connect_sock(&self.sub_sock, ip, *port));
    }

    fn subscribe(&self) {
        subscribe_sock(&self.sub_sock, self.private_key.clone());
        subscribe_sock(&self.sub_sock, String::from(COORD_INFO_KEY));
        subscribe_sock(&self.sub_sock, String::from(BROADCAST_KEY));
    }

    fn publish_start_sim(&self) {
        log::info!("Publishing start sim");
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

    fn send_hello_msg(&self, sock: &Socket) {
        log::info!("Sending hello message");
        let from = self.settings.host_ip.clone();
        let msg = Message::Hello(self.settings.host_ip.clone(), self.settings.pub_port);

        send_rr(sock, from, msg);
        let (_, msg) = recv_rr(sock);
        log::info!("{}", msg.as_string());
    }

    fn wait_for_ip_table(&self) -> Vec<(Ip, Port)> {
        log::info!("Waiting for ip table");
        let x;
        loop {
            let (_, _, msg) = recv_ps(&self.sub_sock);
            if let Message::IpTable(mut ip_table) = msg {
                ip_table.retain(|(ip, port)| {
                    *ip != self.settings.host_ip && *port != self.settings.pub_port
                });
                log::info!("Received Ip Table: {:#?}", ip_table);
                x = ip_table;
                break;
            }
        }
        x
    }

    fn wait_for_signal(&self) {
        log::info!("Waiting for signal to start sim");
        let (_, _, msg) = recv_ps(&self.sub_sock);
        log::info!("{}", msg.as_string());
    }
}

pub fn publish_ip_table(pub_sock: &Socket, identity: &str, ip_table: &[(Ip, Port)]) {
    log::info!("Publishing ip table");
    let key = String::from(COORD_INFO_KEY);
    let from = identity.to_string();
    let msg = Message::IpTable(ip_table.to_owned());

    send_ps(pub_sock, key, from, msg);
}

pub fn wait_for_hosts(
    rep_sock: &Socket,
    identity: &str,
    hosts: u32,
    is_server: bool,
) -> Vec<(Ip, Port)> {
    let mut host_count = hosts;
    if !is_server {
        host_count -= 1;
    }
    let mut ip_table = vec![];
    while ip_table.len() != host_count as usize {
        let (from, msg) = recv_rr(&rep_sock);
        log::info!("{} {}", msg.as_string(), from);
        match msg {
            Message::Hello(ip, port) => {
                ip_table.push((ip, port));
                send_rr(&rep_sock, identity.to_string(), Message::Ok);
            }
            _ => {
                log::warn!("Unexpected msg while waiting for hosts");
                send_rr(&rep_sock, identity.to_string(), Message::Err);
            }
        }
    }
    ip_table
}

pub fn wait_for_confirmations(rep_sock: &Socket, identity: &str, hosts: u32, is_server: bool) {
    log::info!("Waiting for confirmations");
    let mut host_count = hosts;
    if !is_server {
        host_count -= 1;
    }
    let mut count = 0;
    while count != host_count {
        let (from, msg) = recv_rr(rep_sock);
        match msg {
            Message::HostReady => {
                count += 1;
                log::info!("{} {}", msg.as_string(), from);
                send_rr(rep_sock, identity.to_string(), Message::Ok);
            }
            _ => {
                log::warn!("Unexpected msg while waiting for confirmations");
                send_rr(&rep_sock, identity.to_string(), Message::Err);
            }
        }
    }
}
