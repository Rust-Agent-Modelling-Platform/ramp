#[macro_use]
extern crate serde_derive;

mod action;
mod address_book;
mod agent;
mod constants;
mod container;
mod functions;
mod message;
mod settings;
mod stats;

use agent::Agent;
use config;
use config::ConfigError;
use flexi_logger::Logger;
use rand::{thread_rng, Rng};
use settings::Settings;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::{env, thread};
use uuid::Uuid;
use zmq::{Socket, Error};

use crate::address_book::AddressBook;
use crate::container::Container;
use crate::message::Message;
use crate::settings::AgentConfig;

type Port = u32;
const START_SIMULATION_KEY: &str = "START";
const HOST_READY_MSG: &str = "READY";

fn main() -> Result<(), ConfigError> {
    init_logger();
    let settings = load_settings();
    let simulation_dir_path = stats::create_simulation_dir(constants::STATS_DIR_NAME);
    let agent_config = Arc::new(settings.agent_config);
    let (txes, rxes) = create_channels(settings.islands);
    let mut island_ids = create_island_ids(settings.islands);
    let args: Vec<String> = env::args().collect();
    stats::copy_simulation_settings(&simulation_dir_path, args.get(1).unwrap().clone());

    let mut ips: Vec<(IpAddr, Port)> = parse_input_ips(&settings);
    let context = zmq::Context::new();
    let rep = context.socket(zmq::REP).unwrap();
    let req = context.socket(zmq::REQ).unwrap();
    let publisher = context.socket(zmq::PUB).unwrap();
    let sub = context.socket(zmq::SUB).unwrap();

    bind_publisher(&publisher, &settings);
    subscribe(&ips, &sub, &settings);

    if settings.network.is_coordinator {
        bind_rep_sock(&rep, &settings);
        wait_for_hosts(&rep, &settings);
        notify_hosts(&publisher);
    } else {
        connect_to_rep_sock(&req, &settings);
        send_ready_msg(&req, &settings);
        wait_for_signal(&sub);
    }

    let (network_tx, network_rx) = mpsc::channel();
    log::info!("Begin simulation");
    start_simulation(
        settings,
        agent_config,
        txes,
        rxes,
        island_ids,
        simulation_dir_path,
        sub,
        publisher,
        network_tx,
        network_rx,
        ips,
    );

    Ok(())
}

fn load_settings() -> Settings {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() == 2);
    Settings::new(args.get(1).unwrap().clone()).unwrap()
}

fn parse_input_ips(settings: &Settings) -> Vec<(IpAddr, Port)> {
    let mut ips: Vec<(IpAddr, Port)> = Vec::new();
    let ips_str: Vec<String> = settings.network.ips.clone();
    for address in ips_str {
        let mut split: Vec<&str> = address.split(":").collect();
        ips.push((split[0].parse().unwrap(), split[1].parse().unwrap()));
    }
    ips
}

fn bind_publisher(publisher: &Socket, settings: &Settings) {
    publisher
        .bind(&format!(
            "tcp://{}:{}",
            settings.network.host_ip.clone(),
            settings.network.pub_port.to_string()
        ))
        .expect("failed binding pub");
}

fn subscribe(ips: &Vec<(IpAddr, Port)>, sub: &Socket, settings: &Settings) {
    ips.iter().for_each(|(ip, port)| {
        sub.connect(&format!("tcp://{}:{}", ip.to_string(), port.to_string()))
            .expect("failed connecting sub");
    });
    let private_sub_key = &format!(
        "{}:{}",
        settings.network.host_ip.to_string(),
        settings.network.pub_port.to_string()
    );
    sub.set_subscribe(private_sub_key.as_bytes())
        .expect("failed seting sub key");
    sub.set_subscribe(START_SIMULATION_KEY.as_bytes())
        .expect("failed seting sub key");
}

fn bind_rep_sock(rep: &Socket, settings: &Settings) {
    assert!(rep
        .bind(&format!(
            "tcp://{}:{}",
            settings.network.host_ip.to_string(),
            settings.network.coordinator_rep_port.to_string()
        ))
        .is_ok());
}

fn connect_to_rep_sock(req: &Socket, settings: &Settings) {
    assert!(req
        .connect(&format!(
            "tcp://{}:{}",
            settings.network.coordinator_ip.to_string(),
            settings.network.coordinator_rep_port.to_string()
        ))
        .is_ok());
}

fn wait_for_hosts(rep: &Socket, settings: &Settings) {
    let mut count = 0;
    while count != settings.network.hosts_num {
        let from = rep.recv_msg(0).unwrap();
        let data = rep.recv_msg(0).unwrap();
        log::info!("{} {}", data.as_str().unwrap(), from.as_str().unwrap());
        rep.send("OK", 0).unwrap();
        count += 1;
    }
}

fn notify_hosts(publisher: &Socket) {
    log::info!("Notyfing hosts");
    publisher
        .send(START_SIMULATION_KEY, 0)
        .expect("couldn't notify hosts to start sim");
}

fn send_ready_msg(req: &Socket, settings: &Settings) {
    req.send(
        format!("{}:{}", settings.network.host_ip, settings.network.pub_port).into_bytes(),
        zmq::SNDMORE,
    )
    .unwrap();
    req.send(&HOST_READY_MSG, 0).unwrap();
    let msg = req.recv_msg(0).unwrap();;
    log::info!("{}. Waiting for signal to start sim", msg.as_str().unwrap());
}

fn wait_for_signal(sub: &Socket) {
    let msg = sub
        .recv_msg(0)
        .expect("failed receiving signal to start sim");
    log::info!("{}", std::str::from_utf8(&msg).unwrap());
}

fn start_simulation(
    settings: Settings,
    agent_config: Arc<AgentConfig>,
    txes: Vec<Sender<Message>>,
    mut rxes: Vec<Receiver<Message>>,
    island_ids: Vec<Uuid>,
    simulation_dir_path: String,
    subscriber: Socket,
    publisher: Socket,
    network_tx: Sender<Message>,
    network_rx: Receiver<Message>,
    ips: Vec<(IpAddr, Port)>,
) {
    let mut threads = Vec::<thread::JoinHandle<_>>::new();
    let (tx, rx) = mpsc::channel();

    for island_no in 0..settings.islands {
        let island_stats_dir_path =
            stats::create_island_stats_dir(&simulation_dir_path, &island_ids[island_no as usize]);

        let address_book = create_address_book(
            &txes,
            &mut rxes,
            &island_ids,
            island_no as i32,
            &network_tx,
            &tx,
        );

        let mut container = Container::new(
            island_ids[island_no as usize],
            address_book,
            &functions::rastrigin,
            settings.container.agents_number,
            settings.turns,
            agent_config.clone(),
            island_stats_dir_path,
        );

        threads.push(thread::spawn(move || {
            container.run();
        }));
    }

    let tx_clone = tx.clone();
    threads.push(thread::spawn(move || {
        start_publisher_thread(network_rx, publisher, ips, settings, tx_clone);
    }));

    let address_book = create_address_book(&txes, &mut rxes, &island_ids, -1, &network_tx, &tx);
    threads.push(thread::spawn(move || {
        start_subscriber_thread(rx, subscriber, address_book);
    }));

    for thread in threads {
        thread.join().unwrap();
    }
}

fn start_publisher_thread(
    network_rx: Receiver<Message>,
    publisher: Socket,
    ips: Vec<(IpAddr, Port)>,
    settings: Settings,
    tx: Sender<Message>,
) {
    loop {
        let incoming = network_rx.try_iter();
        for msg in incoming {
            match msg {
                // 1. Agent migrating form this node -> some other node
                Message::Agent(migrant) => {
                    let encoded: Vec<u8> = bincode::serialize(&migrant).unwrap();
                    let random_index = thread_rng().gen_range(0, ips.len());
                    let (ip, port) = ips.get(random_index).unwrap();

                    log::error!(
                        "Sending Agent{} to {}:{}",
                        &migrant.id.to_string()[..8],
                        &ip.to_string(),
                        &port.to_string()
                    );
                    publisher
                        .send(
                            &format!("{}:{}", ip.to_string(), port.to_string()),
                            zmq::SNDMORE,
                        )
                        .expect("failed sending sub key");
                    publisher
                        .send(&format!("{}", settings.network.pub_port), zmq::SNDMORE)
                        .expect("failed sending from msg");
                    publisher.send(encoded, 0).expect("failed sending msg");
                }

                // 2. End of the simulation
                Message::FinSim => {
                    tx.send(Message::FinSim).unwrap();
                    log::info!("Ending simulation in network sender thread");
                    break;
                }

                _ => log::error!("Unexpected message"),
            }
        }
    }
}

fn start_subscriber_thread(
    rx: Receiver<Message>,
    subscriber: Socket,
    mut address_book: AddressBook,
) {
    loop {
        let incoming = rx.try_iter();
        for msg in incoming {
            match msg {
                //1. Island informs that it has finished working and is no longer active
                Message::Fin(uuid) => {
                    log::error!("Island {} has finished working - updating information in sub_thread address_book", &uuid.to_string()[..8]);
                    address_book.addresses.get_mut(&uuid).unwrap().1 = false;
                }

                Message::FinSim => {
                    log::info!("Ending simulation in network receiver thread");
                    break;
                }
                _ => log::info!("Unexpected message in sub_thread"),
            }
        }

        //Next step: non-blocking check if there are any new agents waiting to be added to our system
        let mut items = [subscriber.as_poll_item(zmq::POLLIN)];
        zmq::poll(&mut items, -1).unwrap();
        if items[0].is_readable() {
            let sub_key = subscriber.recv_msg(0).expect("failed receiving sub key");
            let from = subscriber.recv_msg(0).expect("failed receiving from msg");
            let message = subscriber.recv_msg(0).expect("failed receiving msg");

            let decoded_agent: Agent = bincode::deserialize(&message[..]).expect("ERROR ERROR ERROR");

            log::error!(
                "Received Agent{} from {}",
                &decoded_agent.id.to_string()[..8],
                std::str::from_utf8(&from).unwrap()
            );

            //Get some random hashmap value that is active. If no more active - drop agent IDGAF
            match address_book.addresses.iter().find(|&x| (x.1).1) {
                Some((island_uuid, island_tx)) => {
                    //Send agent here
                    log::error!(
                        "Sending Agent{} to Island {}",
                        &decoded_agent.id.to_string()[..8],
                        &island_uuid.to_string()[..8]
                    );
                    match island_tx.0.send(Message::Agent(decoded_agent.into())) {
                        Ok(()) => log::error!("Successfully sent"),
                        Err(e) => log::error!("[Subscriber] INTERNAL SENDING UNSUCCESSFUL: {}", e),
                    }
                }
                None => {
                    println!("There are no more active islands - incoming migrant is being dropped")
                }
            }
        }
    }
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
    network_thread: &Sender<Message>,
    sub_tx: &Sender<Message>,
) -> AddressBook {
    let mut addresses: HashMap<Uuid, (Sender<Message>, bool)> = HashMap::new();
    for (i, tx) in txes.iter().enumerate().take(txes.len()) {
        if i != island_no as usize {
            addresses.insert(island_ids[i], (mpsc::Sender::clone(tx), true));
        }
    }
    if rxes.len() >= 1 {
        let rx = rxes.remove(0);
        AddressBook::new(
            addresses,
            mpsc::Sender::clone(network_thread),
            rx,
            mpsc::Sender::clone(sub_tx),
        )
    } else {
        //This is for the sub thread
        let (_, rx) = mpsc::channel();
        AddressBook::new(
            addresses,
            mpsc::Sender::clone(network_thread),
            rx,
            mpsc::Sender::clone(sub_tx),
        )
    }
}