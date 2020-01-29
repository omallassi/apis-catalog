extern crate log;
extern crate env_logger;
extern crate uuid;

use log::{debug,info};

use std::vec::Vec;
use openapiv3::OpenAPI;

use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web::{get, post};
use actix_web::web::Json;
use serde::{Deserialize, Serialize};
use actix_files::{NamedFile, Files};
use actix_web::{HttpRequest, Result};
use std::path::PathBuf;

use uuid::Uuid;

mod dao;
use dao::catalog;
use dao::repo_deployments;
use dao::repo_domains;
use dao::repo_envs;

use repo_deployments::{*};
use repo_domains::{*};
use repo_envs::{*};

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

    let mut all_apis = catalog::get_spec(API_CATALOG_PATH, &info.0);

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
struct Specs {
    specs: Vec<Spec>,
}

#[derive(Serialize, Deserialize)]
struct Spec {
    name: String,
    id: String,
}

#[get("/v1/specs")]
fn get_all_specs() -> HttpResponse {
    debug!("get_all_specs()");
    let mut specs = Specs{
        specs: Vec::new(),
    };

    let mut all_specs = catalog::list_specs(API_CATALOG_PATH);
    while let Some(spec) = all_specs.pop() {
        info!("Analysing file [{:?}]", spec.name);
        let short_path = &spec.name[API_CATALOG_DIR.len()..spec.name.len()];
        let spec = Spec {
            name: String::from(short_path),
            id: spec.id,
        };
        specs.specs.push(spec);
    }    
    
    HttpResponse::Ok().json(specs)
}

#[derive(Serialize, Deserialize, Debug)]
struct Deployment {
    api: String, 
    env: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Deployments {
    deployments: Vec<Deployment>
}

#[post("/v1/deployments")]
fn add_deployment(deployment: Json<Deployment>) -> HttpResponse {
    release(deployment.api.clone(), deployment.env.clone());

    HttpResponse::Ok().json("")
}

#[get("/v1/deployments")]
fn get_deployments() -> HttpResponse {
    let mut deployments = Deployments {
        deployments : Vec::new(),
    };

    let mut all_tuples: Vec<(String, String)> = match list_all_deployments() {
        Ok(all_tuples) => all_tuples, 
        Err(why) => { 
            debug!("No Deployments found");
            Vec::new()
        },
    };

    while let Some(tuple) = all_tuples.pop() {
        let deployment = Deployment {
            api: tuple.0,
            env: tuple.1,
        };        
        deployments.deployments.push(deployment);
    }

    HttpResponse::Ok().json(deployments)
}

fn get_deployments_for_api(path: web::Path<(String,)>) -> HttpResponse {
    let mut deployments = Deployments {
        deployments : Vec::new(),
    };

    let mut all_tuples: Vec<(String, String)> = match get_all_deployments_for_api(&path.0) {
        Ok(all_tuples) => all_tuples, 
        Err(why) => { 
            debug!("No Deployments found for api [{:?}]", &path.0);
            Vec::new()
        },
    };

    while let Some(tuple) = all_tuples.pop() {
        let deployment = Deployment {
            api: tuple.0,
            env: tuple.1,
        };        
        deployments.deployments.push(deployment);
    }

    HttpResponse::Ok().json(deployments)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Domain {
    pub name: String,
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Domains {
    pub domains: Vec<Domain>
}

#[get("/v1/domains")]
pub fn get_domains() -> HttpResponse {
    info!("get domains");
    let mut all_domains: Vec<DomainItem> = match list_all_domains() {
        Ok(all_domains) => all_domains, 
        Err(why) => { 
            panic!("Unable to get domains: {}", why);
        },
    };

    let mut domains = Vec::new();

    while let Some(domain) = all_domains.pop() {
        let domain = Domain {
            name: domain.name,
            id: domain.id,
        };
        domains.push(domain);
    }

    let domains_obj = Domains{
            domains: domains,
    };

    HttpResponse::Ok().json(domains_obj)
}


#[post("/v1/domains")]
pub fn create_domain(domain: Json<Domain>) -> HttpResponse {
    add_domain(&domain.name);
    HttpResponse::Ok().json("")
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Api {
    pub name: String, 
    pub domain_id: Uuid, 
    pub spec_ids: Vec<String>,
}

#[post("/v1/apis")]
pub fn create_api(api: Json<Api>) -> HttpResponse {
    info!("create api [{:?}]", api);

    HttpResponse::Ok().json("")
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Envs {
    pub envs: Vec<Env>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Env {
    pub id: Uuid, 
    pub name: String,
    pub description: String,
}

#[post("/v1/envs")]
pub fn create_env(env: Json<Env>) -> HttpResponse {
    info!("create env [{:?}]", env);
    add_env(&env.name, &env.description);

    HttpResponse::Ok().json("")
}

#[get("/v1/envs")]
pub fn list_env() -> HttpResponse {
    info!("list envs");

    let mut envs = Envs {
        envs: Vec::new(),
    };

    let mut all_tuples: Vec<EnvItem> = match list_all_envs() {
        Ok(all_tuples) => all_tuples, 
        Err(why) => { 
            debug!("No env found [{:?}]", why);
            Vec::new()
        },
    };

    while let Some(tuple) = all_tuples.pop() {
        let env = Env {
            id: tuple.id,
            name: tuple.name,
            description: tuple.description,
        };        
        envs.envs.push(env);
    }

    HttpResponse::Ok().json(envs)
}


/**
 * To server static pages
 */
async fn index(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = PathBuf::from("./static/index.html");
    Ok(NamedFile::open(path)?)
}

/**
 * 
 */
#[actix_rt::main]
async fn main() {
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .route("/v1/endpoints", web::get().to(get_endpoints))
            .service(web::resource("/v1/endpoints/{api}").route(web::get().to(get_endpoints)))
            .service(add_deployment)
            .service(get_deployments)
            .service(web::resource("/v1/deployments/{api}").route(web::get().to(get_deployments_for_api)))
            .service(get_domains)
            .service(create_domain)
            .service(get_all_specs)
            .service(create_api)
            .service(create_env)
            .service(list_env)
            .route("/static", web::get().to(index))
            .service(Files::new("/", "/Users/omallassi/code/rust/apis-catalog-web/build").index_file("index.html"))  // tmp can be replaced with relative "./ui/", 
    })
    .workers(4)
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .await;
}
