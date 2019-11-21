use crate::collector::Collector;
use crate::dispatcher::{Dispatcher, DispatcherMessage};
use crate::island::Island;
use crate::network::CollectorNetworkCtx;
use crate::network::DispatcherNetworkCtx;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Barrier};
use std::thread;

use uuid::Uuid;

use crate::address_book::AddressBook;
use crate::island::{IslandEnv, IslandFactory};
use crate::map::{MapOwners, Fragment, FragmentOwner, Map};
use crate::message::Message;
use crate::metrics::MetricHub;
use crate::network::NetworkCtx;
use crate::settings::ClientSettings;
use crate::{metrics, utils, network};
use std::time::Instant;
use std::collections::HashMap;

const LOGGER_LEVEL: &str = "info";
const EXPECTED_ARGS_NUM: usize = 3;

pub struct Simulation;

impl Simulation {
    pub fn start_simulation(factory: Box<dyn IslandFactory>, metrics: MetricHub) {
        utils::init_logger(LOGGER_LEVEL);
        let args: Vec<String> = utils::parse_input_args(EXPECTED_ARGS_NUM);
        let settings_file_name = args[1].clone();
        let settings = load_settings(settings_file_name.clone());

        log::info!("Initializing simulation");
        let nt_settings = settings.network.clone();
        let nt_ctx = NetworkCtx::new(nt_settings.clone());
        let (dis_nt_ctx, coll_nt_ctx) = nt_ctx.init();

        let metrics_port = settings.network.metrics_port;
        let metrics_addr = format!("{}:{}", settings.network.host_ip.clone(), metrics_port);
        thread::spawn(move || metrics::start_server(metrics_addr));
        start(
            settings,
            dis_nt_ctx,
            coll_nt_ctx,
            factory,
            Arc::new(metrics),
        );
    }
}

fn load_settings(file_name: String) -> ClientSettings {
    ClientSettings::new(file_name).unwrap()
}

#[allow(clippy::too_many_arguments)]
fn start(
    settings: ClientSettings,
    dis_nt_ctx: DispatcherNetworkCtx,
    coll_nt_ctx: CollectorNetworkCtx,
    factory: Box<dyn IslandFactory>,
    metrics: Arc<MetricHub>,
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

    let islands = settings.islands;
    let is_coordinator = dis_nt_ctx.nt_sett.is_coordinator;
    let host_ip = dis_nt_ctx.nt_sett.host_ip.clone();
    let host_port = dis_nt_ctx.nt_sett.pub_port;
    let mut ip_table = dis_nt_ctx.ip_table.clone();

    let coll_address_book = AddressBook {
        dispatcher_tx: mpsc::Sender::clone(&dispatcher_tx),
        addresses: island_txes.clone(),
        islands: island_ids.clone(),
    };

    thread::spawn(move || Dispatcher::new(dispatcher_rx, dis_nt_ctx, islands).start());

    let mut map_owners: MapOwners = HashMap::new();
    if !is_coordinator {
        dispatcher_tx.send(DispatcherMessage::Broadcast(Message::Islands(island_ids.clone()))).unwrap();

        loop {
            let (_, _, msg) = network::recv_ps(&coll_nt_ctx.sub_sock);
            if let Message::Owners(owners) = msg {
                map_owners = owners;
                break;
            }
        }
    } else {
        ip_table.push((host_ip, host_port));

        map_owners = create_map_owners(&coll_nt_ctx.sub_sock, island_ids.clone(), settings.network.hosts_num, ip_table, settings.network.map.chunk_len);
        dispatcher_tx.send(DispatcherMessage::Broadcast(Message::Owners(map_owners.clone()))).unwrap();
    }

    thread::spawn(move || Collector::new(collector_rx, coll_nt_ctx, coll_address_book).start());

    let islands = settings.islands;
    for island_no in 0..islands {
        let turns = settings.turns;
        let island_id = island_ids[island_no as usize];
        let island_sync = islands_sync.clone();

        let address_book =
            create_address_book(&island_txes, &island_ids, island_no as i32, &dispatcher_tx);

        let island_rx = island_rxes.remove(0);

        let map = Map::new(settings.network.map.chunk_len, map_owners.clone());
        let fragment_owner: FragmentOwner = (settings.network.host_ip.clone(), settings.network.pub_port, island_id);

        let island_env = IslandEnv::new(address_book, Arc::clone(&metrics), Instant::now());
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
    island.on_start();
    while let (true, turn, messages) = receive_messages_with_global_sync(&island_rx) {
        island.do_turn(turn, messages);
        island_sync.as_ref().map(|barrier| barrier.wait());
        dispatcher_tx
            .send(DispatcherMessage::Info(Message::TurnDone))
            .unwrap();
    }
    island.on_finish();
}

fn run(
    mut island: Box<dyn Island>,
    island_rx: Receiver<Message>,
    turns: u32,
    island_sync: Option<Arc<Barrier>>,
) {
    island.on_start();
    for turn in 0..turns {
        let messages = island_rx.try_iter().collect();
        island.do_turn(turn, messages);
        island_sync.as_ref().map(|barrier| barrier.wait());
    }
    island.on_finish();
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
    txes: &[Sender<Message>],
    island_ids: &[Uuid],
    island_no: i32,
    dispatcher_tx: &Sender<DispatcherMessage>,
) -> AddressBook {
    let mut txes_cp = txes.to_owned();
    txes_cp.remove(island_no as usize);
    let mut island_ids_cp = island_ids.to_owned();
    island_ids_cp.remove(island_no as usize);

    AddressBook {
        dispatcher_tx: mpsc::Sender::clone(dispatcher_tx),
        addresses: txes_cp,
        islands: island_ids_cp,
    }
}
