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

use config;
use config::ConfigError;
use flexi_logger::Logger;
use settings::Settings;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use rand::{thread_rng, Rng};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::{thread, env};
use uuid::Uuid;
use agent::Agent;
use zmq::Socket;

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
        .bind(&format!("tcp://{}:{}", 
                settings.network.host_ip.clone(), 
                settings.network.pub_port.to_string()
                )
        )
        .expect("failed binding pub");
}

fn subscribe(ips: &Vec<(IpAddr, Port)>, sub: &Socket, settings: &Settings) {
    ips.iter().for_each(|(ip, port)| {
        sub
            .connect(&format!("tcp://{}:{}", ip.to_string(), port.to_string()))
            .expect("failed connecting sub");
    });
    let private_sub_key = &format!("{}:{}", 
        settings.network.host_ip.to_string(), 
        settings.network.pub_port.to_string());
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
    publisher.send(START_SIMULATION_KEY, 0).expect("couldn't notify hosts to start sim");
}

fn send_ready_msg(req: &Socket, settings: &Settings) {
    req.send(format!("{}:{}", settings.network.host_ip, settings.network.pub_port).into_bytes(), zmq::SNDMORE).unwrap();
    req.send(&HOST_READY_MSG, 0).unwrap();
    let msg = req.recv_msg(0).unwrap();;
    log::info!("{}. Waiting for signal to start sim", msg.as_str().unwrap());
}

fn wait_for_signal(sub: &Socket){
    let msg = sub.recv_msg(0).expect("failed receiving signal to start sim");
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
    ips: Vec<(IpAddr, Port)>
) {
    let mut threads = Vec::<thread::JoinHandle<_>>::new();
    let (tx, rx) = mpsc::channel();

    for island_no in 0..settings.islands {
        let island_stats_dir_path =
            stats::create_island_stats_dir(&simulation_dir_path, &island_ids[island_no as usize]);

        let address_book = create_address_book(&txes, &mut rxes, &island_ids, island_no as i32, &network_tx, &tx);

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
    thread::spawn(move || {
        start_publisher_thread(network_rx, publisher, ips, settings, tx_clone);
    });
    
    let address_book = create_address_book(&txes, &mut rxes, &island_ids, -1, &network_tx, &tx);

    thread::spawn(move || {
        start_subscriber_thread(rx, subscriber, address_book);
    });
    
    for thread in threads {
        thread.join().unwrap();
    }
}

fn start_publisher_thread(network_rx: Receiver<Message>, publisher: Socket, ips: Vec<(IpAddr, Port)>, settings: Settings, tx: Sender<Message>) {
    loop {
            let mut incoming = network_rx.try_iter();
            let mut migrants_num = 0;
            for agent in incoming {
                match agent {
                    Message::Agent(migrant) => {
                        migrants_num += 1;
                        // println!("Migrant count:  {}", migrants_num);
                        let encoded: Vec<u8> = bincode::serialize(&migrant).unwrap();
                        // log::error!("{:#?}", ips);
                        let random_index = thread_rng().gen_range(0, ips.len());
                        let (ip, port) = ips.get(random_index).unwrap();
                        // log::error!("{}", port);
                        publisher
                            .send(&format!("{}:{}", ip.to_string(), port.to_string()), zmq::SNDMORE)
                            .expect("failed sending sub key");
                        
                        // only to know from which host is msg
                        publisher
                            .send(&format!("{}", settings.network.pub_port), zmq::SNDMORE) 
                            .expect("failed sending from msg");
                        
                        publisher
                            .send(encoded, 0)
                            .expect("failed sending msg");

                    },
                    Message::FinSim => {
                        tx.send(Message::FinSim).unwrap();
                        log::info!("Ending simulation in network sender thread");
                        break;
                    }
                    _ => println!("MEANS END")
                }
            }
        }
}

fn start_subscriber_thread(rx: Receiver<Message>, subscriber: Socket, address_book: AddressBook) {
    loop {
        // to na pewno jest za mało bo recv_msg niżej jest blokujące
        let messages = rx.try_iter();
        for mess in messages {
            match mess {
                Message::FinSim => {
                    log::info!("Ending simulation in network receiver thread");
                    break;
                },
                _ => log::info!("Received some message. ")
            }
        }

        let sub_key = subscriber
            .recv_msg(0)
            .expect("failed receiving sub key");
        
        let from = subscriber
            .recv_msg(0)
            .expect("failed receiving from msg");

        let message = subscriber
            .recv_msg(0)
            .expect("failed receiving msg");

        let decoded_agent: Agent = bincode::deserialize(&message[..]).unwrap();
        println!("Received agent, from {}", std::str::from_utf8(&from).unwrap());
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
    let rx = rxes.remove(0);
    AddressBook::new(addresses, mpsc::Sender::clone(network_thread), rx, mpsc::Sender::clone(sub_tx))
}
