use std::env;
extern crate log;
use log::{info};

//extern crate env_logger;

extern crate config;
use config::ConfigError;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct StashConfig {
    pub base_uri: String,
    pub access_token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub bind_adress: String,
    pub static_resources_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Database {
    pub rusqlite_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DomainRepoType {
    pub domain_impl: String,
    pub domain_catalog_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SystemsAndLayers {
    pub systems_catalog_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Catalog {
    pub catalog_id: String, 
    pub catalog_name: String,
    pub catalog_path: String,
    pub catalog_dir: String,
    pub catalog_scm_clone_cmd: String,
    pub catalog_scm_pull_cmd: String,
    pub catalog_scm_clone: bool,
    pub catalog_http_base_uri: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Search {
    pub index_path: String,
}


#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub catalogs: Vec<Catalog>,
    pub stash_config: StashConfig,
    pub database: Database,
    pub server: Server,
    pub domain_repo_type: DomainRepoType,
    pub systems_and_layers: SystemsAndLayers,
    pub search: Search,
}

lazy_static! {
    pub static ref SETTINGS: Settings = Settings::new().unwrap();
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut settings = config::Config::default();

        let config_file_path = match env::var("API_SERVER_CONFIG_FILE") {
            Ok(var) => var,
            Err(_why) => String::from("server/config/local"),
        };

        // config_file_path = match fs::metadata(&config_file_path){
        //     Ok(_) => {
        //         info!("Will load Configuration from file - [{:?}]", config_file_path);
        //         config_file_path
        //     }, 
        //     Err(_e) => {
        //         error!("Unable to load config file [{:?}] - will load test file [server/config/test/test]", &config_file_path);
        //         String::from("server/config/test/test")
        //     }
        // };

        let mut config = settings.merge(config::File::with_name(config_file_path.as_str())).unwrap();
        config = config
            // Add in settings from the environment (with a prefix of API)
            // Eg.. `API_DEBUG=1 ./target/app` would set the `debug` key
            .merge(config::Environment::with_prefix("API"))
            .unwrap();
        
        info!("Configuration has been loaded - [{:?}]", settings);
        settings.try_deserialize::<Settings>()
    }
}

pub fn get_catalog_by_id(catalogs: &Vec<Catalog>, id: &str) -> Option<Catalog> {
    let mut val = None;
    for catalog in catalogs {
        if String::from(id) == catalog.catalog_id{
            val = Some(catalog);
            break;
        }
    };

    val.cloned()
}
