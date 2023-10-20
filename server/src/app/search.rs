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
    pub limit: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Result {
    pub audience: String,
    pub domain: String,
    pub systems: Vec<String>,
    pub layer: String,
    pub path: String,
    pub operations: Vec<String>,
    pub catalog_id: String,
    pub spec_path: String,
}

#[post("/v1/search")]
pub async fn search_specs(query: Json<Query>) -> impl Responder{
    info!("search_specs [{:?}] with limit [{:?}]", query.query, query.limit);

    let search_results = search(&SETTINGS.search.index_path, String::from(&query.query), query.limit);
    let results = match search_results {
        Ok(results) => {
            let mut tmp = Vec::new();
            for result in results {
                let catalog_id = &result.catalog_id[0];
                let returned_catalog = get_catalog_by_id(&SETTINGS.catalogs, &catalog_id);

                let mut new_spec_path = String::from(&result.spec_path[0]);
                if let Some(catalog) = returned_catalog{
                    let tmp = crate::app::dao::catalog::extact_relative_path(&result.spec_path[0], &catalog.catalog_dir);
                    new_spec_path = String::from(tmp);
                }


                let systems_as_vec: Vec<String> = String::from(&result.systems[0]).split_whitespace().map(String::from).collect();
                let operations_as_vec: Vec<String> = String::from(&result.operations[0]).split_whitespace().map(String::from).collect();

                tmp.push(Result{
                    audience: String::from(&result.audience[0]),
                    domain: String::from(&result.domain[0]),
                    systems: systems_as_vec,
                    layer: String::from(&result.layer[0]),
                    path: String::from(&result.path[0]),
                    operations: operations_as_vec,
                    catalog_id: String::from(catalog_id),
                    spec_path: new_spec_path,
                });
            }
            //TODO to map to another struct from this layer 
            tmp
        }, 
        Err(e) => {
            error!("Error while searching for query [{:?}] - [{:?}]", query, e);
            Vec::new()
        }
    };

    HttpResponse::Ok().json(results)
}