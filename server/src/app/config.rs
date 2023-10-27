
use actix_web::{get, Responder};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

extern crate reqwest;
use crate::shared::settings::SETTINGS;

use log::info;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub api_doc_url: String, 
    pub pact_url: String,
    pub stash_base_url: String, 
    pub beta: bool
}

#[get("/v1/config")]
pub async fn get_config_for_ui() -> impl Responder{
    info!("get config");
    let config = Config{
        api_doc_url: SETTINGS.ui_config.api_doc_url.to_string(),
        pact_url: SETTINGS.ui_config.pact_url.to_string(),
        stash_base_url: SETTINGS.ui_config.stash_base_url.to_string(),
        beta: SETTINGS.ui_config.beta
    };

    HttpResponse::Ok().json(&config)
}