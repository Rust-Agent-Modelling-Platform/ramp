use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize, Clone)]
pub struct ClientSettings {
    pub turns: u32,
    pub islands: u32,
    pub network: NetworkSettings,
    pub islands_sync: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NetworkSettings {
    pub is_coordinator: bool,
    pub hosts_num: u32,
    pub coordinator_ip: String,
    pub coordinator_rep_port: u32,
    pub coordinator_pub_port: u32,
    pub host_ip: String,
    pub pub_port: u32,
    pub metrics_port: u32,
    pub global_sync: GlobalSyncSettings,
    pub map: MapSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GlobalSyncSettings {
    pub sync: bool,
    pub server_ip: String,
    pub server_rep_port: u32,
    pub server_pub_port: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MapSettings {
    pub chunk_len: u64,
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
    pub metrics_port: u32,
}

impl ServerSettings {
    pub fn new(file_name: String) -> Result<Self, ConfigError> {
        let mut settings = Config::new();

        settings.merge(File::with_name(&file_name))?;
        settings.try_into()
    }
}
