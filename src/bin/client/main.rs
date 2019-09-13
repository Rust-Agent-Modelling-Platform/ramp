use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Barrier};
use std::{env, thread};

use config;
use config::ConfigError;
use flexi_logger::Logger;
use rand::{thread_rng, Rng};
use uuid::Uuid;
use zmq::Socket;

use rust_in_peace::address_book::AddressBook;
use rust_in_peace::island::Island;
use rust_in_peace::message::Message;
use rust_in_peace::settings::AgentConfig;
use rust_in_peace::settings::Settings;
use rust_in_peace::{constants, functions, stats};
use rust_in_peace::network;

type Port = u32;
const SERVER_INFO_KEY: &str = "SERVER_INFO";

fn main() -> Result<(), ConfigError> {
    init_logger();
    let args: Vec<String> = parse_input_args();
    let settings_file_name = args[1].clone();
    let settings = load_settings(settings_file_name.clone());
    let simulation_dir_path = stats::create_simulation_dir(constants::STATS_DIR_NAME);
    let agent_config = Arc::new(settings.agent_config);
    let (island_txes, island_rxes) = create_channels(settings.islands);
    let island_ids = create_island_ids(settings.islands);
    stats::copy_simulation_settings(&simulation_dir_path, settings_file_name.clone());

    let ips: Vec<(IpAddr, Port)> = parse_input_ips(&settings);
    let context = zmq::Context::new();
    let rep = context.socket(zmq::REP).unwrap();
    let req = context.socket(zmq::REQ).unwrap();
    let publisher = context.socket(zmq::PUB).unwrap();
    let sub = context.socket(zmq::SUB).unwrap();

    network::bind_sock(&publisher, settings.network.host_ip.clone(), settings.network.pub_port);
    connect(&sub, &ips, );
    subscribe(&sub, &settings);

    if settings.network.is_coordinator {
        let ip = settings.network.coordinator_ip.clone();
        let port = settings.network.coordinator_rep_port;
        network::bind_sock(&rep, ip, port);
        wait_for_hosts(&rep, &settings);
        notify_hosts(&publisher, &settings);
    } else {
        let ip = settings.network.coordinator_ip.clone();
        let port = settings.network.coordinator_rep_port;
        network::connect_sock(&req, ip, port);
        send_ready_msg(&req, &settings);
        wait_for_signal(&sub);
    }

    log::info!("Begin simulation");
    start_simulation(
        settings,
        agent_config,
        island_txes,
        island_rxes,
        island_ids,
        simulation_dir_path,
        sub,
        publisher,
        ips,
    );

    Ok(())
}

fn parse_input_args() -> Vec<String> {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 2);
    args
}

fn load_settings(file_name: String) -> Settings {
    Settings::new(file_name).unwrap()
}

fn parse_input_ips(settings: &Settings) -> Vec<(IpAddr, Port)> {
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

fn subscribe(sub: &Socket, settings: &Settings) {
    let private_sub_key = format!(
        "{}:{}",
        settings.network.host_ip.to_string(),
        settings.network.pub_port.to_string()
    );
    network::subscribe_sock(sub, private_sub_key);
    network::subscribe_sock(sub, String::from(SERVER_INFO_KEY));
}

fn wait_for_hosts(rep: &Socket, settings: &Settings) {
    let mut count = 0;
    while count != settings.network.hosts_num {
        let from = rep.recv_msg(0).unwrap();
        let msg = rep.recv_bytes(0).unwrap();

        let d_msg: Message = bincode::deserialize(&msg).unwrap();
        log::info!("{} {}", d_msg.into_string(), from.as_str().unwrap());

        let self_ip = &settings.network.host_ip;
        let self_port = settings.network.coordinator_rep_port;
        let from = format!("{}:{}",self_ip, self_port);
        let msg = Message::Ok;
        network::send1(rep, from, msg);
        count += 1;
    }
}

fn notify_hosts(publisher: &Socket, settings: &Settings) {
    log::info!("Notifying hosts");
    let key = String::from(SERVER_INFO_KEY);
    let from = format!("{}:{}", settings.network.host_ip, settings.network.pub_port);
    let msg = Message::StartSim;

    network::send2(publisher, key, from, msg);
}

fn send_ready_msg(req: &Socket, settings: &Settings) {
    let self_ip = &settings.network.host_ip;
    let self_port = settings.network.pub_port;
    let from = format!("{}:{}", self_ip, self_port);
    let msg = Message::HostReady;

    network::send1(req, from, msg);
    let (_, msg) = network::recv1(req);
    log::info!("{}. Waiting for signal to start sim", msg.into_string());
}

fn wait_for_signal(sub: &Socket) {
    let (_, _, msg) = network::recv2(sub);
    log::info!("{}", msg.into_string());
}

#[allow(clippy::too_many_arguments)]
fn start_simulation(
    settings: Settings,
    agent_config: Arc<AgentConfig>,
    island_txes: Vec<Sender<Message>>,
    mut island_rxes: Vec<Receiver<Message>>,
    island_ids: Vec<Uuid>,
    simulation_dir_path: String,
    subscriber: Socket,
    publisher: Socket,
    ips: Vec<(IpAddr, Port)>,
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
        let island_stats_dir_path =
            stats::create_island_stats_dir(&simulation_dir_path, &island_ids[island_no as usize]);

        let address_book = create_address_book(
            &island_txes,
            &mut island_rxes,
            &island_ids,
            island_no as i32,
            &pub_tx,
        );

        let mut island = Island::new(
            island_ids[island_no as usize],
            address_book,
            &functions::rastrigin,
            settings.island.agents_number,
            settings.turns,
            agent_config.clone(),
            island_stats_dir_path,
            islands_sync.clone(),
        );

        threads.push(thread::spawn(move || {
            island.run();
        }));
    }

    let sub_th_address_book =
        create_address_book(&island_txes, &mut island_rxes, &island_ids, -1, &pub_tx);
    thread::spawn(move || start_publisher_thread(pub_rx, publisher, ips, settings));
    thread::spawn(move || start_subscriber_thread(sub_rx, subscriber, sub_th_address_book));

    for thread in threads {
        thread.join().unwrap();
    }

    pub_tx.send(Message::FinSim).unwrap();
    sub_tx.send(Message::FinSim).unwrap();
}

fn start_publisher_thread(
    pub_rx: Receiver<Message>,
    publisher: Socket,
    ips: Vec<(IpAddr, Port)>,
    settings: Settings,
) {
    log::info!("Starting pub thread");
    let mut fin_sim = false;
    let self_ip = settings.network.host_ip;
    let self_port = settings.network.pub_port;
    while !fin_sim {
        let incoming = pub_rx.try_iter();
        for msg in incoming {
            match msg {
                Message::Agent(_) => {
                    let random_index = thread_rng().gen_range(0, ips.len());
                    let (ip, port) = ips[random_index];
                    let key = format!("{}:{}", ip.to_string(), port.to_string());
                    let from = format!("{}:{}", self_ip, self_port);

                    network::send2(&publisher, key, from, msg);
                }
                Message::FinSim => {
                    log::info!("Finishing simulation in pub thread");
                    fin_sim = true;
                    break;
                }
                _ => log::warn!("Unexpected msg in pub thread"),
            }
        }
    }
}

fn start_subscriber_thread(
    rx: Receiver<Message>,
    subscriber: Socket,
    mut address_book: AddressBook,
) {
    log::info!("Starting sub thread");
    let mut fin_sim = false;
    while !fin_sim {
        let incoming = rx.try_iter();
        for msg in incoming {
            match msg {
                Message::FinSim => {
                    log::info!("Ending simulation in network receiver thread");
                    fin_sim = true;
                    break;
                }
                _ => log::warn!("Unexpected message in sub thread"),
            }
        }

        //Next step: non-blocking check if there are any new agents waiting to be added to our system
        let mut items = [subscriber.as_poll_item(zmq::POLLIN)];
        zmq::poll(&mut items, -1).unwrap();
        if items[0].is_readable() {
            let (_, _, msg) = network::recv2(&subscriber);
            match address_book.send_to_rnd(msg) {
                Ok(()) => (),
                Err(e) => {
                    log::info!("{:?} (No more active islands in system)", e);
                }
            }
        }
    }
    log::info!("Finishing sub thread");
}

fn init_logger() {
    Logger::with_str("info")
        .format_for_stderr(flexi_logger::colored_default_format)
        .start()
        .unwrap();
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
