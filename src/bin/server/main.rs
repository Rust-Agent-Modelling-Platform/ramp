use rust_in_peace::message::Message;
use rust_in_peace::network::recv_rr;
use rust_in_peace::settings::ServerSettings;
use rust_in_peace::{network, utils};
use zmq::Socket;

const LOGGER_LEVEL: &str = "info";
const EXPECTED_ARGS_NUM: usize = 2;

fn main() {
    utils::init_logger(LOGGER_LEVEL);
    let args: Vec<String> = utils::parse_input_args(EXPECTED_ARGS_NUM);
    let settings_file_name = args[1].clone();
    let settings = load_settings(settings_file_name.clone());

    let context = zmq::Context::new();
    let rep_sock = context.socket(zmq::REP).unwrap();
    let pub_sock = context.socket(zmq::PUB).unwrap();

    network::bind_sock(&rep_sock, settings.ip.clone(), settings.rep_port);
    network::bind_sock(&pub_sock, settings.ip.clone(), settings.pub_port);

    let from = format!("{}:{}", settings.ip, settings.pub_port);
    log::info!("WAITING FOR HOSTS");
    wait_for_hosts(&rep_sock, from.clone(), settings.hosts);
    run(rep_sock, pub_sock, from, settings.hosts, settings.turns);
}

fn wait_for_hosts(rep_sock: &Socket, from: String, hosts: u32) {
    let mut host = 0;
    while host < hosts {
        let (recv_from, msg) = network::recv_rr(rep_sock);
        match msg {
            Message::HostReady => {
                host += 1;
                network::send_rr(rep_sock, from.clone(), Message::Ok);
                log::info!("{} {}", msg.as_string(), recv_from);
            }
            _ => log::warn!("Unexpected msg"),
        }
    }
}

fn run(rep_sock: Socket, pub_sock: Socket, from: String, hosts: u32, turns: u32) {
    log::info!("START SIM");
    let key = String::from(network::SERVER_INFO_KEY);
    let mut turn = 0;
    while turn < turns {
        log::info!("TURN {}", turn + 1);
        network::send_ps(
            &pub_sock,
            key.clone(),
            from.clone(),
            Message::NextTurn(turn + 1),
        );
        turn += 1;
        wait_for_confirmations(&rep_sock, from.clone(), hosts);
    }
    log::info!("FIN SIM");
    network::send_ps(&pub_sock, key, from.clone(), Message::FinSim);
}

fn wait_for_confirmations(rep_sock: &Socket, from: String, hosts: u32) {
    let mut confirmation = 0;
    while confirmation < hosts {
        let (_, msg) = recv_rr(rep_sock);
        match msg {
            Message::TurnDone => {
                confirmation += 1;
                network::send_rr(rep_sock, from.clone(), Message::Ok);
            }
            _ => log::warn!("Unexpected message {:#?}", msg),
        }
    }
}

fn load_settings(file_name: String) -> ServerSettings {
    ServerSettings::new(file_name).unwrap()
}
