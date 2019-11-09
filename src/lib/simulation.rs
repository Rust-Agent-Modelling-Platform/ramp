use crate::collector::Collector;
use crate::dispatcher::{Dispatcher, DispatcherMessage};
use crate::island::Island;
use std::net::IpAddr;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Barrier};
use std::thread;

use uuid::Uuid;
use zmq::Socket;

use crate::address_book::AddressBook;
use crate::island::{IslandEnv, IslandFactory};
use crate::message::Message;
use crate::settings::ClientSettings;
use crate::{network, utils};
use std::time::Instant;

type Port = u32;

const LOGGER_LEVEL: &str = "info";
const EXPECTED_ARGS_NUM: usize = 3;

pub struct Simulation;

impl Simulation {
    pub fn start_simulation(factory: Box<dyn IslandFactory>) {
        utils::init_logger(LOGGER_LEVEL);
        let args: Vec<String> = utils::parse_input_args(EXPECTED_ARGS_NUM);
        let settings_file_name = args[1].clone();
        let settings = load_settings(settings_file_name.clone());
        let simulation_dir_path = utils::create_simulation_dir(&settings.stats_path.clone());
        utils::copy_simulation_settings(&simulation_dir_path, settings_file_name.clone());

        let ips: Vec<(IpAddr, Port)> = parse_input_ips(&settings);
        let context = zmq::Context::new();
        let rep_sock = context.socket(zmq::REP).unwrap();
        let req_sock = context.socket(zmq::REQ).unwrap();
        let s_req_sock = context.socket(zmq::REQ).unwrap();
        let pub_sock = context.socket(zmq::PUB).unwrap();
        let sub_sock = context.socket(zmq::SUB).unwrap();

        let host_ip = settings.network.host_ip.clone();
        let host_pub_port = settings.network.pub_port;
        network::bind_sock(&pub_sock, host_ip, host_pub_port);
        connect(&sub_sock, &ips);
        subscribe(&sub_sock, &settings);

        if settings.network.is_coordinator {
            let coord_ip = settings.network.coordinator_ip.clone();
            let coord_rep_port = settings.network.coordinator_rep_port;
            network::bind_sock(&rep_sock, coord_ip, coord_rep_port);
            wait_for_hosts(&rep_sock, &settings);
            notify_hosts(&pub_sock, &settings);
        } else {
            let coord_ip = settings.network.coordinator_ip.clone();
            let coord_rep_port = settings.network.coordinator_rep_port;
            network::connect_sock(&req_sock, coord_ip, coord_rep_port);
            send_ready_msg(&req_sock, &settings);
            wait_for_signal(&sub_sock);
        }

        if settings.network.global_sync.sync {
            let server_ip = settings.network.global_sync.server_ip.clone();
            let server_rep_port = settings.network.global_sync.server_rep_port;
            let server_pub_port = settings.network.global_sync.server_pub_port;
            network::connect_sock(&s_req_sock, server_ip.clone(), server_rep_port);
            network::connect_sock(&sub_sock, server_ip, server_pub_port);
            network::subscribe_sock(&sub_sock, String::from(network::SERVER_INFO_KEY));
            send_ready_msg(&s_req_sock, &settings);
        }

        log::info!("Begin simulation");
        start_simulation(
            settings,
            simulation_dir_path,
            sub_sock,
            pub_sock,
            ips,
            s_req_sock,
            factory,
        );
    }
}

fn load_settings(file_name: String) -> ClientSettings {
    ClientSettings::new(file_name).unwrap()
}

fn parse_input_ips(settings: &ClientSettings) -> Vec<(IpAddr, Port)> {
    let mut ips: Vec<(IpAddr, Port)> = Vec::new();
    let ips_str: Vec<String> = settings.network.ips.clone();
    for address in ips_str {
        let split: Vec<&str> = address.split(':').collect();
        ips.push((split[0].parse().unwrap(), split[1].parse().unwrap()));
    }
    ips
}

fn connect(sub: &Socket, ips: &[(IpAddr, Port)]) {
    ips.iter()
        .for_each(|(ip, port)| network::connect_sock(sub, ip.to_string(), *port));
}

fn subscribe(sub: &Socket, settings: &ClientSettings) {
    let private_sub_key = format!(
        "{}:{}",
        settings.network.host_ip.to_string(),
        settings.network.pub_port.to_string()
    );
    network::subscribe_sock(sub, private_sub_key);
    network::subscribe_sock(sub, String::from(network::COORD_INFO_KEY));
    network::subscribe_sock(sub, String::from(network::BROADCAST_KEY));
}

fn wait_for_hosts(rep: &Socket, settings: &ClientSettings) {
    let mut count = 0;
    while count != settings.network.hosts_num {
        let from = rep.recv_msg(0).unwrap();
        let msg = rep.recv_bytes(0).unwrap();

        let d_msg: Message = bincode::deserialize(&msg).unwrap();
        log::info!("{} {}", d_msg.as_string(), from.as_str().unwrap());

        let from = settings.network.host_ip.clone();
        let msg = Message::Ok;
        network::send_rr(rep, from, msg);
        count += 1;
    }
}

fn notify_hosts(pub_sock: &Socket, settings: &ClientSettings) {
    log::info!("Notifying hosts");
    let key = String::from(network::COORD_INFO_KEY);
    let from = settings.network.host_ip.clone();
    let msg = Message::StartSim;

    network::send_ps(pub_sock, key, from, msg);
}

fn send_ready_msg(req: &Socket, settings: &ClientSettings) {
    log::info!("Sending host ready message");
    let from = settings.network.host_ip.clone();
    let msg = Message::HostReady;

    network::send_rr(req, from, msg);
    let (_, msg) = network::recv_rr(req);
    log::info!("{}", msg.as_string());
}

fn wait_for_signal(sub: &Socket) {
    log::info!("Waiting for signal to start sim");
    let (_, _, msg) = network::recv_ps(sub);
    log::info!("{}", msg.as_string());
}

#[allow(clippy::too_many_arguments)]
fn start_simulation(
    settings: ClientSettings,
    simulation_dir_path: String,
    sub_sock: Socket,
    pub_sock: Socket,
    ips: Vec<(IpAddr, Port)>,
    srv_req_sock: Socket,
    factory: Box<dyn IslandFactory>,
) {
    let (island_txes, mut island_rxes) = create_channels(settings.islands);
    let island_ids = create_island_ids(settings.islands);
    let mut threads = Vec::<thread::JoinHandle<_>>::new();

    let (collector_tx, collector_rx) = mpsc::channel();
    let (dispatcher_tx, dispatcher_rx) = mpsc::channel();

    let islands_sync = if settings.islands_sync {
        Some(Arc::new(Barrier::new(settings.islands as usize)))
    } else {
        None
    };

    for island_no in 0..settings.islands {
        let turns = settings.turns;
        let island_id = island_ids[island_no as usize];
        let stats_dir_path = utils::create_island_stats_dir(&simulation_dir_path, &island_id);
        let island_sync = islands_sync.clone();

        let address_book = 
            create_address_book(&island_txes, &island_ids, island_no as i32, &dispatcher_tx);

        let island_rx = island_rxes.remove(0);
        let island_env = IslandEnv::new(address_book, stats_dir_path, Instant::now());
        let island = factory.create(island_id, island_env);
        let dispatcher_tx_cp = mpsc::Sender::clone(&dispatcher_tx);
        let th_handler = if settings.network.global_sync.sync {
            thread::spawn(move || {
                run_with_global_sync(island, island_rx, island_sync, dispatcher_tx_cp)
            })
        } else {
            thread::spawn(move || run(island, island_rx, turns, island_sync))
        };
        threads.push(th_handler);
    }

    let collector_address_book = AddressBook {
        dispatcher_tx: mpsc::Sender::clone(&dispatcher_tx),
        addresses: island_txes.clone(),
        islands: island_ids.clone(),
    };
    let settings_copy = settings.clone();
    thread::spawn(move || {
        Dispatcher::create(dispatcher_rx, pub_sock, ips, settings_copy, srv_req_sock).start()
    });
    thread::spawn(move || {
        Collector::create(collector_rx, sub_sock, collector_address_book).start()
    });

    for thread in threads {
        thread.join().unwrap();
    }

    if !settings.network.global_sync.sync {
        dispatcher_tx
            .send(DispatcherMessage::Info(Message::FinSim))
            .unwrap();
        collector_tx.send(Message::FinSim).unwrap();
    }
}

fn run_with_global_sync(
    mut island: Box<dyn Island>,
    island_rx: Receiver<Message>,
    island_sync: Option<Arc<Barrier>>,
    dispatcher_tx: Sender<DispatcherMessage>,
) {
    while let (true, turn, messages) = receive_messages_with_global_sync(&island_rx) {
        island.do_turn(turn, messages);
        island_sync.as_ref().map(|barrier| barrier.wait());
        dispatcher_tx.send(DispatcherMessage::Info(Message::TurnDone)).unwrap();
    }
    island.finish();
}

fn run(
    mut island: Box<dyn Island>,
    island_rx: Receiver<Message>,
    turns: u32,
    island_sync: Option<Arc<Barrier>>,
) {
    for turn in 0..turns {
        let messages = island_rx.try_iter().collect();
        island.do_turn(turn, messages);
        island_sync.as_ref().map(|barrier| barrier.wait());
    }
    island.finish();
}

type NextTurn = bool;
type Turn = u32;
fn receive_messages_with_global_sync(rx: &Receiver<Message>) -> (NextTurn, Turn, Vec<Message>) {
    let mut msg_queue = vec![];
    let mut next_turn = false;
    let mut fin_sim = false;
    let mut current_turn = 0;
    while !next_turn && !fin_sim {
        let messages = rx.try_iter();
        for msg in messages {
            match msg {
                Message::NextTurn(turn_number) => {
                    current_turn = turn_number;
                    next_turn = true
                }
                Message::FinSim => fin_sim = true,
                _ => msg_queue.push(msg),
            }
        }
    }
    (!fin_sim, current_turn as u32, msg_queue)
}

fn create_channels(islands_number: u32) -> (Vec<Sender<Message>>, Vec<Receiver<Message>>) {
    let mut txes = Vec::<Sender<Message>>::new();
    let mut rxes = Vec::<Receiver<Message>>::new();
    for _ in 0..islands_number {
        let (tx, rx) = mpsc::channel();
        txes.push(tx);
        rxes.push(rx);
    }
    (txes, rxes)
}

fn create_island_ids(islands_number: u32) -> Vec<Uuid> {
    let mut island_ids = Vec::<Uuid>::new();
    for _ in 0..islands_number {
        island_ids.push(Uuid::new_v4());
    }
    island_ids
}

fn create_address_book(
    txes: &Vec<Sender<Message>>,
    island_ids: &Vec<Uuid>,
    island_no: i32,
    dispatcher_tx: &Sender<DispatcherMessage>,
) -> AddressBook {
    let mut txes_cp = txes.clone();
    txes_cp.remove(island_no as usize);
    let mut island_ids_cp = island_ids.clone();
    island_ids_cp.remove(island_no as usize);

    AddressBook {
        dispatcher_tx: mpsc::Sender::clone(dispatcher_tx),
        addresses: txes_cp,
        islands: island_ids_cp,
    }
}
