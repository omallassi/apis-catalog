extern crate log;
extern crate env_logger;
use log::{info};

use std::vec::Vec;
use openapiv3::OpenAPI;

use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web::{get, post};
use actix_web::web::Json;
use serde::{Deserialize, Serialize};

mod catalog;
mod repo;
/**
 * 
 */
const API_CATALOG_PATH: &str = "/Users/omallassi/code/apis-catalog/"; //"/Users/omallassi/code/rust/apis-catalog/server/samples/"; //"/Users/omallassi/code/apis-catalog/";
const API_CATALOG_DIR: &str = "/Users/omallassi/code/apis-catalog/catalog/";


/**
 * 
 */
#[derive(Serialize, Deserialize)]
struct Endpoints {
    endpoints: Vec<Endpoint>,
}

#[derive(Serialize, Deserialize)]
struct Endpoint {
    name: String,
}

//#[get("/v1/endpoints/{api}")]
fn get_endpoints(info: web::Path<(String,)>) -> HttpResponse {
    
    let mut endpoints = Endpoints{
        endpoints: Vec::new(),
    };

    let mut all_apis = catalog::get_api(API_CATALOG_PATH, &info.0);

    while let Some(api) = all_apis.pop() {
        info!("Analysing file [{:?}]", api.name);

        let openapi: OpenAPI = api.api_spec;
        for val in openapi.paths.keys() {
            let endpoint = Endpoint {
                name: String::from(val),
            };
            endpoints.endpoints.push(endpoint);
        }
    }    
    
    HttpResponse::Ok().json(endpoints)
}

/**
 * 
 */
#[derive(Serialize, Deserialize)]
struct Apis {
    apis: Vec<Api>,
}

#[derive(Serialize, Deserialize)]
struct Api {
    name: String,
    id: String,
}

#[get("/v1/apis")]
fn get_apis() -> HttpResponse {
    
    let mut apis = Apis{
        apis: Vec::new(),
    };

    let mut all_apis = catalog::list_apis(API_CATALOG_PATH);
    while let Some(api) = all_apis.pop() {
        info!("Analysing file [{:?}]", api.name);
        let short_path = &api.name[API_CATALOG_DIR.len()..api.name.len()];
        let api = Api {
            name: String::from(short_path),
            id: api.id,
        };
        apis.apis.push(api);
    }    
    
    HttpResponse::Ok().json(apis)
}

#[derive(Serialize, Deserialize, Debug)]
struct Deployment {
    api: String, 
    env: String,
}

#[post("/v1/deployments")]
fn add_deployment(deployment: Json<Deployment>) -> HttpResponse {
    repo::release(deployment.api.clone(), deployment.env.clone());

    HttpResponse::Ok().json("")
}

#[get("/v1/deployments")]
fn get_deployments() -> HttpResponse {
    repo::list_all_deployments();

    HttpResponse::Ok().json("")
}

/**
 * 
 */
fn main() {
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            //.route("/v1/endpoints", web::get().to(get_endpoints))
            .service(web::resource("/v1/endpoints/{api}").route(web::get().to(get_endpoints)))
            .service(add_deployment)
            .service(get_deployments)
            .service(get_apis)
    })
    .workers(4)
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .unwrap();
}
