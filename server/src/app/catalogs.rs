
use actix_web::{get, post, Responder};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

extern crate reqwest;
use crate::shared::settings::Catalog as RepoCatalog;
use crate::shared::settings::SETTINGS;

use log::{info, error};

#[derive(Serialize, Deserialize, Clone)]
pub struct Catalog {
    pub id: String,
    pub name: String,
    pub http_base_uri: String, 
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Catalogs {
    pub catalogs: Vec<Catalog>,
}

#[get("/v1/catalogs/{id}")]
pub async fn get_catalog_by_id(path: web::Path<String>) -> impl Responder{
    let id: String = path.into_inner();
    info!("get catalog for id [{:?}]", &id);

    let catalog_as_vec = &SETTINGS.catalogs;

    let catalog_as_map = catalog_as_vec.into_iter().map(|data| (data.catalog_id.clone(), data.clone())).collect::<HashMap<String, RepoCatalog>>();
    let curr_catalog = &catalog_as_map.get(&id).unwrap();

    let returned_catalog = self::Catalog {
        id: String::from(&curr_catalog.catalog_id),
        name: String::from(&curr_catalog.catalog_name),
        http_base_uri: String::from(&curr_catalog.catalog_http_base_uri)
    };

    HttpResponse::Ok().json(&returned_catalog)
}

#[get("/v1/catalogs")]
pub async fn get_all_catalog() -> impl Responder{
    info!("get all catalogs");

    let catalog_as_vec = &SETTINGS.catalogs;
    
    let mut returned_catalog: Vec<Catalog> = Vec::new();
    for cat in catalog_as_vec{
        returned_catalog.push(Catalog{ 
            id: String::from(&cat.catalog_id), 
            name: String::from(&cat.catalog_name), 
            http_base_uri: String::from(&cat.catalog_http_base_uri) });
    }

    HttpResponse::Ok().json(&returned_catalog)
}

#[post("/v1/catalogs/refresh")]
pub async fn refresh_all_catalogs() -> impl Responder{
    info!("refresh_all_catalogs");
    resfresh_caches_and_indexes(false);
    HttpResponse::Ok().json(())
}

pub fn resfresh_caches_and_indexes(init: bool) {
    crate::app::dao::catalog::refresh_catalogs(&SETTINGS.catalogs, init);
    let specs = crate::app::dao::catalog::list_specs(&SETTINGS.catalogs);
    match crate::app::dao::search::build_index(&SETTINGS.search.index_path, &specs){
        Ok(_results) => {
            //does nothing, logs have already been written
        },
        Err(e) => {
            error!("Error while indexing all specs - [{:?}]", e);
        }
    };
}