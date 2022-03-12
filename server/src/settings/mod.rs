use std::env;
extern crate log;
use log::info;

//extern crate env_logger;

extern crate config;
use config::ConfigError;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StashConfig {
    pub base_uri: String,
    pub access_token: String,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub bind_adress: String,
    pub static_resources_path: String,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub rusqlite_path: String,
}

#[derive(Debug, Deserialize)]
pub struct DomainRepoType {
    pub domain_impl: String,
    pub domain_catalog_path: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub catalog_path: String,
    pub catalog_dir: String,
    pub stash_config: StashConfig,
    pub database: Database,
    pub server: Server,
    pub domain_repo_type: DomainRepoType,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut settings = config::Config::default();

        let config_file_path = match env::var("API_SERVER_CONFIG_FILE") {
            Ok(var) => var,
            Err(_why) => String::from("server/config/local"),
        };

        info!("Will load Configiuration from file - [{:?}]", config_file_path);

        settings
            .merge(config::File::with_name(config_file_path.as_str()))
            .unwrap()
            // Add in settings from the environment (with a prefix of API)
            // Eg.. `API_DEBUG=1 ./target/app` would set the `debug` key
            .merge(config::Environment::with_prefix("API"))
            .unwrap();

        info!("Configuration has been loaded - [{:?}]", settings);
        settings.try_into()
    }
}
