use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct Container {
    pub agents_number: u32,
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct Settings {
    pub turns: u32,
    pub islands: u32,
    pub container: Container,
    pub agent_config: AgentConfig,
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

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut settings = Config::new();

        settings.merge(File::with_name("Settings"))?;
        settings.try_into()
    }
}
