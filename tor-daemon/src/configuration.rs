use config::{Config, ConfigError, Environment};

#[derive(serde::Deserialize)]
pub struct Configuration {
    pub virtual_port: u16,
    pub target_address: String,
    pub target_port: u16,
}

impl Configuration {
    pub fn load() -> Result<Configuration, ConfigError> {
        let mut config = Config::default();
        config.merge(Environment::with_prefix("APP").separator("__"))?;
        config.try_into()
    }
}
