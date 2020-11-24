use openapiv3::OpenAPI;

use actix_web::web::Json;
use actix_web::{get, post};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[path = "../dao/mod.rs"]
mod dao;
use dao::repo_apis::*;
use dao::repo_domains::*;

use log::{debug, error, info};

#[path = "../settings/mod.rs"]
mod settings;
use settings::Settings;

use uuid::Uuid;

lazy_static! {
    static ref SETTINGS: settings::Settings = Settings::new().unwrap();
}

/*
 * APIs & specs related APIs
 */

#[derive(Serialize, Deserialize)]
pub struct Endpoints {
    endpoints: Vec<Endpoint>,
}

#[derive(Serialize, Deserialize)]
pub struct Endpoint {
    name: String,
}

//#[get("/v1/endpoints/{api}")]
pub fn get_endpoints(info: web::Path<(String,)>) -> HttpResponse {
    let mut endpoints = Endpoints {
        endpoints: Vec::new(),
    };

    let mut all_apis = dao::catalog::get_spec(SETTINGS.catalog_path.as_str(), &info.0);

    while let Some(api) = all_apis.pop() {
        info!("Analysing file [{:?}]", api.path);

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

#[derive(Serialize, Deserialize)]
pub struct Specs {
    specs: Vec<Spec>,
}

#[derive(Serialize, Deserialize)]
pub struct Spec {
    name: String,
    title: String,
    version: String,
    description: String,
    id: String,
    audience: String,
}

#[get("/v1/specs")]
pub fn get_all_specs() -> HttpResponse {
    debug!("get_all_specs()");
    let mut specs = Specs { specs: Vec::new() };

    let mut all_specs = dao::catalog::list_specs(SETTINGS.catalog_path.as_str());
    while let Some(spec) = all_specs.pop() {
        info!("Analysing file [{:?}]", spec.path);
        let short_path =
            dao::catalog::get_spec_short_path(String::from(&SETTINGS.catalog_dir), &spec);
        let spec = Spec {
            name: String::from(short_path),
            id: spec.id,
            title: spec.api_spec.info.title,
            version: spec.api_spec.info.version,
            description: match spec.api_spec.info.description {
                Some(d) => d,
                None => String::from(""),
            },
            audience: spec.audience,
        };
        specs.specs.push(spec);
    }
    HttpResponse::Ok().json(specs)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Api {
    pub id: Uuid,
    pub name: String,
    pub tier: String,
    pub status: Status,
    pub domain_id: Uuid,
    pub domain_name: String,
    pub spec_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Status {
    VALIDATED,
    DEPRECATED,
    RETIRED,
    NONE,
}

//TODO I should be able to store the enum in DB but cannot figure out how - so back to basis
impl Status {
    fn as_str(&self) -> String {
        match *self {
            Status::VALIDATED => String::from("VALIDATED"),
            Status::DEPRECATED => String::from("DEPRECATED"),
            Status::RETIRED => String::from("RETIRED"),
            _ => String::from("NONE"),
        }
    }

    fn from_str(val: String) -> Status {
        match val.as_str() {
            "VALIDATED" => Status::VALIDATED,
            "DEPRECATED" => Status::DEPRECATED,
            "RETIRED" => Status::RETIRED,
            _ => Status::NONE,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Apis {
    pub apis: Vec<Api>,
}

#[post("/v1/apis")]
pub fn create_api(api: Json<Api>) -> HttpResponse {
    info!("create api [{:?}]", api);

    dao::repo_apis::add_api(&SETTINGS.database, &api.name, &api.domain_id).unwrap();

    HttpResponse::Ok().json("")
}

#[get("/v1/apis")]
pub fn list_all_apis() -> HttpResponse {
    info!("list all apis");

    let mut all_apis: Vec<ApiItem> = match dao::repo_apis::list_all_apis(&SETTINGS.database) {
        Ok(all_apis) => all_apis,
        Err(why) => {
            error!("Unable to get apis: {}", why);
            panic!();
        }
    };

    let mut apis = Vec::new();

    while let Some(api) = all_apis.pop() {
        //get domain related to this API
        let domain = match dao::repo_domains::get_domain(&SETTINGS.database, api.domain_id) {
            Ok(val) => val,
            Err(why) => {
                error!(
                    "Problem while getting domain [{}] for api [{}] - {}",
                    api.domain_id, api.id, why
                );

                let domain = DomainItem {
                    name: "N/A".to_string(),
                    id: Uuid::nil(),
                    description: "".to_string(),
                    owner: "".to_string(),
                };
                domain
            }
        };
        //
        let api = Api {
            name: api.name,
            id: api.id,
            tier: api.tier.name,
            status: Status::from_str(api.status),
            domain_id: domain.id,
            domain_name: domain.name,
            spec_ids: Vec::new(), //TODO
        };
        apis.push(api);
    }

    let apis_obj = Apis { apis: apis };

    HttpResponse::Ok().json(apis_obj)
}

pub fn get_api_by_id(path: web::Path<(String,)>) -> HttpResponse {
    info!("getting api for id [{:?}]", &path.0);
    let api = Uuid::parse_str(&path.0).unwrap();

    let api = dao::repo_apis::get_api_by_id(&SETTINGS.database, api).unwrap();

    let domain = dao::repo_domains::get_domain(&SETTINGS.database, api.domain_id).unwrap();

    let api = Api {
        id: api.id,
        name: api.name,
        tier: api.tier.name,
        status: Status::from_str(api.status),
        domain_id: domain.id,
        domain_name: domain.name,
        spec_ids: Vec::new(), //TODO
    };

    HttpResponse::Ok().json(api)
}

pub fn update_api_status_by_id(path: web::Path<(String,)>, status: Json<Status>) -> HttpResponse {
    //path: web::Path<(String,)>,
    //&path.0
    info!("updating api for id [{:?}]", &path.0);

    let status_item = StatusItem {
        api_id: Uuid::parse_str(&path.0).unwrap(),
        status: status.as_str(),
    };

    dao::repo_apis::update_api_status(&SETTINGS.database, status_item).unwrap();

    HttpResponse::Ok().json("")
}

pub fn update_api_tier_by_id(path: web::Path<(String,)>, tier: Json<String>) -> HttpResponse {
    //path: web::Path<(String,)>,
    //&path.0
    info!("updating api for id [{:?}] and tier [{}]", &path.0, tier);

    let api_id = Uuid::parse_str(&path.0).unwrap();
    let tier_id = Uuid::parse_str(tier.as_str()).unwrap();

    dao::repo_apis::update_api_tier(&SETTINGS.database, api_id, tier_id).unwrap();

    HttpResponse::Ok().json("")
}
