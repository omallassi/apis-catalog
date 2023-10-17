use actix_web::{post, Responder};
use actix_web::{HttpResponse};
use actix_web::web::Json;
use serde::{Deserialize, Serialize};
use crate::shared::settings::*;
use crate::app::dao::search::*;

use log::{info, error};


#[derive(Serialize, Deserialize, Debug)]
pub struct Query {
    pub query: String,
}

#[post("/v1/search")]
pub async fn search_specs(query: Json<Query>) -> impl Responder{
    info!("search_specs [{:?}]", query.query);

    let search_results = search(&SETTINGS.search.index_path, String::from(&query.query), 10000);
    let results = match search_results {
        Ok(results) => {
            //TODO to map to another struct from this layer 
            results
        }, 
        Err(e) => {
            error!("Error while searching for query [{:?}] - [{:?}]", query, e);
            Vec::new()
        }
    };

    HttpResponse::Ok().json(results)
}