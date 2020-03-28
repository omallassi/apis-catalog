extern crate log;
use log::{debug,info};

extern crate env_logger;

extern crate config;
use config::{ConfigError, Config, File, Environment};

use serde::{Deserialize};

#[derive(Debug, Deserialize)]
pub struct StashConfig {
    pub base_uri: String, 
    pub user: String, 
    pub pwd: String,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub rusqlite_path: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub catalog_path: String,
    pub catalog_dir: String,
    pub stash_config: StashConfig,
    pub database: Database,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        //TODO find a way to inject this config file
        let mut settings = config::Config::default();
    
        settings
            .merge(config::File::with_name("server/config/local")).unwrap()
            // Add in settings from the environment (with a prefix of API)
            // Eg.. `API_DEBUG=1 ./target/app` would set the `debug` key
            .merge(config::Environment::with_prefix("API")).unwrap();

        info!("Configuration has been loaded - [{:?}]", settings);
        settings.try_into()
    }
}