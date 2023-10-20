extern crate log;
use log::info;

extern crate config;
use config::ConfigError;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Server {
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: Server,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        //TODO find a way to inject this config file
        let mut settings = config::Config::default();

        settings
            .merge(config::File::with_name("cli/config/local"))
            .unwrap()
            // Add in settings from the environment (with a prefix of API)
            // Eg.. `API_DEBUG=1 ./target/app` would set the `debug` key
            .merge(config::Environment::with_prefix("API"))
            .unwrap();

        info!("Configuration has been loaded - [{:?}]", settings);
        settings.try_deserialize::<Settings>()
    }
}
