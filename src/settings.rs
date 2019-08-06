use config::{ConfigError, Config, File};

#[derive(Debug, Deserialize)]
pub struct Container {
    pub agents_number: u32,
    pub max_agents_number: usize,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub iterations: u32,
    pub container: Container,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut settings = Config::new();

        settings.merge(File::with_name("Settings"))?;
        settings.try_into()
    }
}