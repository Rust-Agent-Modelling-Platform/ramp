use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize, Clone)]
pub struct ClientSettings {
    pub turns: u32,
    pub islands: u32,
    pub network: Network,
    pub island: Island,
    pub islands_sync: bool,
    pub agent_config: AgentConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Network {
    pub is_coordinator: bool,
    pub hosts_num: u32,
    pub coordinator_ip: String,
    pub coordinator_rep_port: u32,
    pub host_ip: String,
    pub pub_port: u32,
    pub ips: Vec<String>,
    pub global_sync: GlobalSync,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GlobalSync {
    pub sync: bool,
    pub server_ip: String,
    pub server_rep_port: u32,
    pub server_pub_port: u32,
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct Island {
    pub agents_number: u32,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct AgentConfig {
    pub genotype_dim: i32,
    pub initial_energy: i32,
    pub minimum: bool,
    pub mutation_rate: f64,
    pub procreation_prob: i32,
    pub procreation_penalty: f64,
    pub meeting_penalty: i32,
    pub lower_bound: f64,
    pub upper_bound: f64,
}

impl ClientSettings {
    pub fn new(file_name: String) -> Result<Self, ConfigError> {
        let mut settings = Config::new();

        settings.merge(File::with_name(&file_name))?;
        settings.try_into()
    }
}

#[derive(Debug, Deserialize)]
pub struct ServerSettings {
    pub hosts: u32,
    pub turns: u32,
    pub ip: String,
    pub rep_port: u32,
    pub pub_port: u32,
}

impl ServerSettings {
    pub fn new(file_name: String) -> Result<Self, ConfigError> {
        let mut settings = Config::new();

        settings.merge(File::with_name(&file_name))?;
        settings.try_into()
    }
}
