extern crate log;
extern crate env_logger;
use log::{info, debug, warn, error};

use std::fs::File;
use std::io::Read;
use std::error::Error;
use std::vec::Vec;
use serde_yaml;
use openapiv3::OpenAPI;

use actix_web::{App, HttpResponse, HttpServer};
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

#[get("/v1/endpoints")]
fn get_endpoints() -> HttpResponse {
    
    let mut endpoints = Endpoints{
        endpoints: Vec::new(),
    };

    let mut all_endpoints = catalog::list_openapi_files(API_CATALOG_PATH);
    while let Some(top) = all_endpoints.pop() {
        info!("Analysing file [{}]", top);

        //top /Users/omallassi/code/apis-catalog/catalog/reference-data.reference-data/system-descriptor.yaml  
        //let data = include_str!("openapi-sample.yaml");
        let mut file = match File::open(&top) {
            Err(why) => panic!("couldn't open file [{}]", why.description()),
            Ok(file) => file,
        };

        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        //println!("{}", contents);
        if contents.starts_with("openapi") {
            let openapi: OpenAPI = serde_yaml::from_str(&contents).unwrap();

            //match openapi {
            //    Ok(openapi) => openapi,
            //    Err(why) => warn!("couldn't parse {}: {}", top,
            //                                           why.description()),
            //};

            for val in openapi.paths.keys() {
                let endpoint = Endpoint {
                    name: String::from(val),
                };
                endpoints.endpoints.push(endpoint);
            }
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
}

#[get("/v1/apis")]
fn get_apis() -> HttpResponse {
    
    let mut apis = Apis{
        apis: Vec::new(),
    };

    let mut all_apis = catalog::list_openapi_files(API_CATALOG_PATH);
    while let Some(top) = all_apis.pop() {
        info!("Analysing file [{}]", top);
        let short_path = &top[API_CATALOG_DIR.len()..top.len()];
        let api = Api {
            name: String::from(short_path),
        };
        apis.apis.push(api);
    }    
    
    HttpResponse::Ok().json(apis)
}

#[derive(Serialize, Deserialize, Debug)]
struct Release {
    api: String, 
    commit_id: String,
}

#[post("/v1/releases")]
fn add_release(release: Json<Release>) -> HttpResponse {
    repo::release(release.api.clone(), release.commit_id.clone());

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
            .service(get_endpoints)
            .service(add_release)
            .service(get_apis)
    })
    .workers(4)
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .unwrap();
}
