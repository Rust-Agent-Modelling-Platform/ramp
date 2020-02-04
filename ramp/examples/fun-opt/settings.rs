use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct SimulationSettings {
    pub island_settings: IslandSettings,
    pub agent_settings: AgentSettings,
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct IslandSettings {
    pub agents_number: u32,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct AgentSettings {
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

impl SimulationSettings {
    pub fn new(file_name: String) -> Result<Self, ConfigError> {
        let mut settings = Config::new();

        settings.merge(File::with_name(&file_name))?;
        settings.try_into()
    }
}
