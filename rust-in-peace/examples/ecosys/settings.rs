use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct SimulationSettings {
    pub island_settings: IslandSettings,
    pub sheep_settings: SheepSettings,
    pub wolf_settings: WolfSettings,
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct IslandSettings {
    pub agents_number: u32,
    pub grass_interval: i32,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct SheepSettings {
    pub init_num: u32,
    pub init_energy: i64,
    pub reproduction_chance: f64,
    pub energy_gain: i64,
    pub energy_loss: i64,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct WolfSettings {
    pub init_num: u32,
    pub init_energy: i64,
    pub reproduction_chance: f64,
    pub energy_gain: i64,
    pub energy_loss: i64,
}

impl SimulationSettings {
    pub fn new(file_name: String) -> Result<Self, ConfigError> {
        let mut settings = Config::new();

        settings.merge(File::with_name(&file_name))?;
        settings.try_into()
    }
}
