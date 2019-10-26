use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Barrier};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use config;
use config::ConfigError;
use rand::{Rng, thread_rng};
use uuid::Uuid;
use zmq::Socket;

use crate::{constants, network, utils};
use crate::address_book::AddressBook;
use crate::island::{IslandEnv, IslandFactory};
use crate::message::Message;
use crate::settings::{AgentConfig, ClientSettings};

type Port = u32;

const LOGGER_LEVEL: &str = "info";
const EXPECTED_ARGS_NUM: usize = 2;

pub struct Simulation;

impl Simulation {
    pub fn start_simulation(factory: Box<dyn IslandFactory>) {
        utils::init_logger(LOGGER_LEVEL);
        let args: Vec<String> = utils::parse_input_args(EXPECTED_ARGS_NUM);
        let settings_file_name = args[1].clone();
        let settings = load_settings(settings_file_name.clone());
        let simulation_dir_path = utils::create_simulation_dir(constants::STATS_DIR_NAME);
        let (island_txes, island_rxes) = create_channels(settings.islands);
        let island_ids = create_island_ids(settings.islands);
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
            island_txes,
            island_rxes,
            island_ids,
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

fn notify_hosts(publisher: &Socket, settings: &ClientSettings) {
    log::info!("Notifying hosts");
    let key = String::from(network::COORD_INFO_KEY);
    let from = settings.network.host_ip.clone();
    let msg = Message::StartSim;

    network::send_ps(publisher, key, from, msg);
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
    island_txes: Vec<Sender<Message>>,
    mut island_rxes: Vec<Receiver<Message>>,
    island_ids: Vec<Uuid>,
    simulation_dir_path: String,
    subscriber: Socket,
    publisher: Socket,
    ips: Vec<(IpAddr, Port)>,
    s_req: Socket,
    factory: Box<dyn IslandFactory>,
) {
    let mut threads = Vec::<thread::JoinHandle<_>>::new();

    let (sub_tx, sub_rx) = mpsc::channel();
    let (pub_tx, pub_rx) = mpsc::channel();

    let islands_sync = if settings.islands_sync {
        Some(Arc::new(Barrier::new(settings.islands as usize)))
    } else {
        None
    };

    for island_no in 0..settings.islands {
        let stats_dir_path = utils::create_island_stats_dir(&simulation_dir_path, &island_ids[island_no as usize]);

        let address_book = create_address_book(
            &island_txes,
            &mut island_rxes,
            &island_ids,
            island_no as i32,
            &pub_tx,
        );

        let island_env = IslandEnv {
            address_book,
            stats_dir_path,
            islands_sync: islands_sync.clone(),
        };

        let island_id = island_ids[island_no as usize];
        let mut island = factory.create(island_id, island_env);

        let th_handler = if settings.network.global_sync.sync {
            thread::spawn(move || island.run_with_global_sync())
        } else {
            thread::spawn(move || island.run())
        };
        threads.push(th_handler);
    }

    let sub_th_address_book =
        create_address_book(&island_txes, &mut island_rxes, &island_ids, -1, &pub_tx);
    let settings_copy = settings.clone();
    thread::spawn(move || start_sender_thread(pub_rx, publisher, ips, settings_copy, s_req));
    thread::spawn(move || start_receiver_thread(sub_rx, subscriber, sub_th_address_book));

    for thread in threads {
        thread.join().unwrap();
    }

    if !settings.network.global_sync.sync {
        pub_tx.send(Message::FinSim).unwrap();
        sub_tx.send(Message::FinSim).unwrap();
    }
}

fn start_sender_thread(
    pub_rx: Receiver<Message>,
    publisher: Socket,
    ips: Vec<(IpAddr, Port)>,
    settings: ClientSettings,
    s_req: Socket,
) {
    log::info!("Starting sender thread");
    let mut fin_sim = false;
    let mut confirmations = 0;
    let from = settings.network.host_ip;
    while !fin_sim {
        let incoming = pub_rx.try_iter();
        for msg in incoming {
            match msg {
                Message::Agent(_) => {
                    let random_index = thread_rng().gen_range(0, ips.len());
                    let (ip, port) = ips[random_index];
                    let key = format!("{}:{}", ip.to_string(), port.to_string());

                    network::send_ps(&publisher, key, from.clone(), msg);
                }
                Message::TurnDone => {
                    confirmations += 1;
                    if confirmations == settings.islands {
                        network::send_rr(&s_req, from.clone(), Message::TurnDone);
                        let (_, _) = network::recv_rr(&s_req);
                        confirmations = 0;
                    }
                }
                Message::FinSim => {
                    log::info!("Finishing simulation in sender thread");
                    fin_sim = true;
                    break;
                }
                _ => log::warn!("Unexpected msg in sender thread {:#?}", msg),
            }
        }
    }
    log::info!("Sender thread finished")
}

fn start_receiver_thread(rx: Receiver<Message>, sub_sock: Socket, mut address_book: AddressBook) {
    log::info!("Starting receiver thread");
    let mut fin_sim = false;
    while !fin_sim {
        let incoming = rx.try_iter();
        for msg in incoming {
            match msg {
                Message::FinSim => {
                    log::info!("Finishing simulation in receiver thread");
                    address_book.send_to_all(msg);
                    fin_sim = true;
                    break;
                }
                _ => log::warn!("Unexpected message in receiver thread {:#?}", msg),
            }
        }

        //Next step: non-blocking check if there are any new agents waiting to be added to our system
        let mut items = [sub_sock.as_poll_item(zmq::POLLIN)];
        zmq::poll(&mut items, -1).unwrap();
        if items[0].is_readable() {
            let (_, _, msg) = network::recv_ps(&sub_sock);
            match msg {
                Message::NextTurn(_) => {
                    address_book.send_to_all(msg);
                }
                Message::FinSim => {
                    log::info!("Finishing simulation in receiver thread");
                    address_book.send_to_all(msg.clone());
                    address_book.pub_tx.send(msg).unwrap();
                    break;
                }
                Message::Agent(_) => match address_book.send_to_rnd(msg) {
                    Ok(()) => (),
                    Err(e) => {
                        log::info!("{:?} (No more active islands in system)", e);
                    }
                },
                _ => log::warn!("Unexpected message in receiver thread {:#?}", msg),
            }
        }
    }
    log::info!("Receiver thread finished");
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
    txes: &[Sender<Message>],
    rxes: &mut Vec<Receiver<Message>>,
    island_ids: &[Uuid],
    island_no: i32,
    pub_tx: &Sender<Message>,
) -> AddressBook {
    let mut addresses: HashMap<Uuid, (Sender<Message>, bool)> = HashMap::new();
    for (i, tx) in txes.iter().enumerate().take(txes.len()) {
        if i != island_no as usize {
            addresses.insert(island_ids[i], (mpsc::Sender::clone(tx), true));
        }
    }
    if !rxes.is_empty() {
        let rx = rxes.remove(0);
        AddressBook::new(rx, addresses, mpsc::Sender::clone(pub_tx))
    } else {
        //This is for the sub thread
        let (_, rx) = mpsc::channel();
        AddressBook::new(rx, addresses, mpsc::Sender::clone(pub_tx))
    }
}