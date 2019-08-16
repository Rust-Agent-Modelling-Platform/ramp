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
    pub agent: Agent
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct Agent {
    pub genotype_dim: i32,
    pub mutation_rate: f64,
    pub minimum: bool,
    pub procreation_prob: i32,
    pub procreation_penalty: f64,
    pub meeting_penalty: i32
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut settings = Config::new();

        settings.merge(File::with_name("Settings"))?;
        settings.try_into()
    }
}
