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
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::cell::RefCell;
use std::thread;
use uuid::Uuid;
use zmq::Socket;

use crate::address_book::AddressBook;
use crate::container::Container;
use crate::message::Message;
use crate::settings::AgentConfig;

fn main() -> Result<(), ConfigError> {
    init_logger();
    let context = zmq::Context::new();

    let settings = Settings::new()?;
    let simulation_dir_path = stats::create_simulation_dir(constants::STATS_DIR_NAME);
    let agent_config = Arc::new(settings.agent_config);
    let (txes, rxes) = create_channels(settings.islands);

    let mut island_ids = create_island_ids(settings.islands);
    //islands_number is always the main thread
    island_ids.push(Uuid::new_v4());

    stats::copy_simulation_settings(&simulation_dir_path);

    //Settings for sockets
    let is_coordinator = true;

    let coordinator_address = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let node1_address = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let worker_num = 1;

    //Same for every node in the cluster
    let router_port = 5555;

    //This port has to be different on each instance if working on localhost
    let publisher_port = 5561;

    //Should be a list of node addresses
    let node1_port = 5563;
    //let node2_port = ...
    //let node3_port = ...

    let router = context.socket(zmq::ROUTER).unwrap();
    let requester = context.socket(zmq::DEALER).unwrap();
    let publisher = context.socket(zmq::PUB).unwrap();
    let subscriber = context.socket(zmq::SUB).unwrap();

    log::debug!("Main thread is socket thread - beginning:");
    log::debug!("1/5 Initializing router/dealer sockets");
    if is_coordinator {
        //Set up sockets of the coordinator
        //First the router
        assert!(router
            .bind(&format!(
                "tcp://{}:{}",
                coordinator_address.to_string(),
                router_port.to_string()
            ))
            .is_ok());
    } else {
        //Set up the requester socket of a worker
        assert!(requester
            .connect(&format!(
                "tcp://{}:{}",
                coordinator_address.to_string(),
                router_port.to_string()
            ))
            .is_ok());
    }

    log::debug!("2/5 Initializing pub/sub sockets");
    //Publisher:
    publisher
        .bind(&format!(
            "tcp://{}:{}",
            coordinator_address.to_string(),
            publisher_port.to_string()
        ))
        .expect("failed binding publisher");

    //Subscribers: this should be in a loop according to how many workers there are
    subscriber
        .connect(&format!(
            "tcp://{}:{}",
            node1_address.to_string(),
            node1_port.to_string()
        ))
        .expect("failed connecting subscriber");

    //Each node should have some kind of id, also in config
    subscriber.set_subscribe(b"A").expect("failed subscribing");

    //This is a blocking call
    if is_coordinator {
        log::debug!("3/5 This is the leader, so waiting until all workers are ready...");
        let mut count = 0;
        while count != worker_num {
            let return_address = router.recv_msg(0).unwrap();
            let _null_del = router.recv_msg(0).unwrap();
            let data = router.recv_msg(0).unwrap();

            log::debug!("Got message: {}", data.as_str().unwrap());

            router.send(return_address, zmq::SNDMORE).unwrap();
            router.send(data, 0).unwrap();

            count += 1;
        }
    } else {
        log::debug!("3/5 This is a worker node, so sending READY message");
        //let id: Vec<u8> = vec![80, 69, 69, 82, 66]; //ASCII codes
        //requester.set_identity(&id);
        requester
            .send(zmq::Message::from(""), zmq::SNDMORE)
            .unwrap();
        requester.send("B is ready", 0).unwrap();

        let msg = requester.recv_msg(0).unwrap();;
        println!("Received: {}. Good to go", msg.as_str().unwrap());
    }

    log::debug!("4/5 Begin simulation");
    start_simulation(
        settings,
        agent_config,
        txes,
        rxes,
        island_ids,
        simulation_dir_path,
        subscriber,
        publisher
    );

    log::debug!("5/5 Enter polling loop, receive agents and send them to worker threads");


    Ok(())
}

fn start_simulation(
    settings: Settings,
    agent_config: Arc<AgentConfig>,
    txes: Vec<Sender<Message>>,
    mut rxes: Vec<Receiver<Message>>,
    island_ids: Vec<Uuid>,
    simulation_dir_path: String,
    subscriber: Socket,
    publisher: Socket
) {
    let mut threads = Vec::<thread::JoinHandle<_>>::new();

    for island_no in 0..settings.islands {
        let island_stats_dir_path =
            stats::create_island_stats_dir(&simulation_dir_path, &island_ids[island_no as usize]);

        let address_book = create_address_book(&txes, &mut rxes, &island_ids, island_no as usize);

        let mut container = Container::new(
            island_ids[island_no as usize],
            address_book,
            &functions::rastrigin,
            settings.container.agents_number,
            settings.turns,
            agent_config.clone(),
            island_stats_dir_path,
            island_ids[settings.islands as usize]
        );

        threads.push(thread::spawn(move || {
            container.run();
        }));
    }

    for thread in threads {
        thread.join().unwrap();
    }

    loop {
        //1. Check if anybody wants to migrate out from this node - check local tx
        let mut incoming = rxes[0].try_iter();
//        if incoming.next().is_none() == true {
//            println!("--------------no agents------------");
//        }
        let mut migrants_num = 0;
        for agent in incoming {
            match agent {
                Message::Agent(migrant) => {
                    migrants_num += 1;
                    println!("Migrant count:  {}", migrants_num);
                    let encoded: Vec<u8> = bincode::serialize(&migrant).unwrap();
                    publisher
                        .send("B", zmq::SNDMORE)
                        .expect("failed sending first envelope");
                    publisher
                        .send(encoded, 0)
                        .expect("failed sending first message");

                },
                _ => println!("MEANS END")
            }
        }


        println!("Also got here");
        //2. Check if anybody wants to migrate to this node - check the sub socket
        let mut items = [subscriber.as_poll_item(zmq::POLLIN)];
        zmq::poll(&mut items, -1).unwrap();

        if items[0].is_readable() {
            let message = subscriber.recv_msg(0).unwrap();
            //send agent to a thread
        }
    }


}

fn init_logger() {
    Logger::with_str("debug")
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
    //And at the end add the main thread channel
    let (tx_main, rx_main) = mpsc::channel();
    txes.push(tx_main);
    rxes.push(rx_main);
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
    island_no: usize,
) -> AddressBook {
    let mut addresses: HashMap<Uuid, (Sender<Message>, bool)> = HashMap::new();

    for (i, tx) in txes.iter().enumerate().take(txes.len()) {
        if i != island_no {
            addresses.insert(island_ids[i], (mpsc::Sender::clone(tx), true));
        }
    }
    let rx = rxes.remove(0);
    AddressBook::new(addresses, rx)
}
