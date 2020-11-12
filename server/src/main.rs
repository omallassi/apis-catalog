extern crate log;
//extern crate env_logger;
extern crate uuid;

extern crate reqwest;
use reqwest::Client;

extern crate config;

use chrono::{DateTime, TimeZone, Utc};

use log::{debug, error, info};

use openapiv3::OpenAPI;
use std::vec::Vec;

use actix_files::{Files, NamedFile};
use actix_web::web::Json;
use actix_web::{get, post};
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};
use actix_web::{HttpRequest, Result};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use uuid::Uuid;

mod dao;
use dao::catalog;
use dao::repo_apis;
use dao::repo_deployments;
use dao::repo_domains;
use dao::repo_envs;
use dao::repo_metrics;

use catalog::*;
use repo_apis::*;
use repo_deployments::*;
use repo_domains::*;
use repo_envs::*;

mod settings;
use settings::Settings;

#[macro_use]
extern crate lazy_static;

extern crate histogram;
use histogram::Histogram;

use std::convert::TryFrom;

// // Scheduler, and trait for .seconds(), .minutes(), etc.
// use clokwerk::{Scheduler, TimeUnits};
// // Import week days and WeekDay
// use clokwerk::Interval::*;
// use std::time::Duration;
// use std::thread;

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
    let mut endpoints = Endpoints {
        endpoints: Vec::new(),
    };

    let mut all_apis = catalog::get_spec(SETTINGS.catalog_path.as_str(), &info.0);

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
    title: String,
    version: String,
    description: String,
    id: String,
    audience: String,
}

fn get_spec_short_path(spec: &SpecItem) -> &str {
    let short_path = &spec.path[SETTINGS.catalog_dir.as_str().len()..spec.path.len()];

    short_path
}

#[get("/v1/specs")]
fn get_all_specs() -> HttpResponse {
    debug!("get_all_specs()");
    let mut specs = Specs { specs: Vec::new() };

    let mut all_specs = catalog::list_specs(SETTINGS.catalog_path.as_str());
    while let Some(spec) = all_specs.pop() {
        info!("Analysing file [{:?}]", spec.path);
        let short_path = get_spec_short_path(&spec);
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
struct Deployment {
    api: String,
    env: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Deployments {
    deployments: Vec<Deployment>,
}

#[post("/v1/deployments")]
fn add_deployment(deployment: Json<Deployment>) -> HttpResponse {
    release(
        &SETTINGS.database,
        deployment.api.clone(),
        deployment.env.clone(),
    );

    HttpResponse::Ok().json("")
}

#[get("/v1/deployments")]
fn get_deployments() -> HttpResponse {
    let mut deployments = Deployments {
        deployments: Vec::new(),
    };

    let mut all_tuples: Vec<(String, String)> = match list_all_deployments(&SETTINGS.database) {
        Ok(all_tuples) => all_tuples,
        Err(why) => {
            debug!("No Deployments found");
            Vec::new()
        }
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
        deployments: Vec::new(),
    };

    let mut all_tuples: Vec<(String, String)> =
        match get_all_deployments_for_api(&SETTINGS.database, &path.0) {
            Ok(all_tuples) => all_tuples,
            Err(why) => {
                debug!("No Deployments found for api [{:?}]", &path.0);
                Vec::new()
            }
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
    pub description: String,
    pub owner: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Domains {
    pub domains: Vec<Domain>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    pub level: usize,
    pub name: String,
    pub parent: String,
    pub value: i32,
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.name == other.name && self.parent == other.parent && self.level == other.level
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut name = self.name.to_string();
        let key = name.push_str(self.parent.as_str());
        key.hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DomainError {
    pub spec_domain: String,
    pub spec_path: String,
    pub resources: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DomainErrors {
    pub errors: Vec<DomainError>,
}

#[get("/v1/domains/errors")]
pub fn get_domains_errors() -> HttpResponse {
    info!("get domains errors");

    //get all specs
    let all_specs: Vec<SpecItem> = catalog::list_specs(SETTINGS.catalog_path.as_str());
    //at this stage data = {"N/A - servers not specified": 11, "/v1/settlement/operational-arrangement": 8, "/v1/market-risk/scenarios": 10,....
    let data: std::collections::HashMap<String, usize> =
        catalog::get_endpoints_num_per_subdomain(&all_specs);

    //get all declared (and official) domains
    let all_domains: Vec<String> = match list_all_domains(&SETTINGS.database) {
        Ok(all_domains) => all_domains
            .iter()
            .map(|val| String::from(&val.name))
            .collect(),
        Err(why) => {
            panic!("Unable to get domains: {}", why);
        }
    };

    //make the check
    let mut errors: Vec<DomainError> = Vec::new();
    for spec in &all_specs {
        let short_path = get_spec_short_path(&spec);
        let spec_domain = &spec.domain;
        //will loop over all_domains to check if domains "match or not". contains() cannot work as the yml contains /v1 and not the domain
        let mut is_contained = false;
        for domain in &all_domains {
            match spec_domain.contains(domain.as_str()) {
                true => {
                    // info!(
                    //     "[{}] is contained - [{}] resources - spec [{}]",
                    //     &spec.domain,
                    //     data.get(&spec.domain).unwrap(),
                    //     short_path
                    // );
                    is_contained = true;
                    break;
                }
                false => {
                    // info!(
                    //     "[{}] is *not* contained - [{}] resources - spec [{}]",
                    //     &spec.domain,
                    //     data.get(&spec.domain).unwrap(),
                    //     short_path
                    // );
                    is_contained = false;
                }
            }
            debug!(
                "Matching [{}] with [{}] - is_contained [{}]",
                spec_domain, domain, is_contained
            );
        }

        if !is_contained {
            let error = DomainError {
                spec_domain: String::from(&spec.domain),
                spec_path: String::from(short_path),
                resources: *data.get(&spec.domain).unwrap(),
            };

            errors.push(error);
        }
    }

    //return the response
    let errors = DomainErrors { errors: errors };

    HttpResponse::Ok().json(errors)
}

#[get("/v1/domains/stats")]
pub fn get_domains_stats() -> HttpResponse {
    info!("get domains stats");

    let all_specs: Vec<SpecItem> = catalog::list_specs(SETTINGS.catalog_path.as_str());

    let data: std::collections::HashMap<String, usize> =
        catalog::get_endpoints_num_per_subdomain(&all_specs);

    //at this stage the  data structure contains
    //{"N/A - servers not specified": 11, "/v1/settlement/operational-arrangement": 8, "/v1/market-risk/scenarios": 10,....
    //and

    let mut response: std::collections::HashSet<Node> = std::collections::HashSet::new();
    //name of the node must be unique, so we might add an id to ensure this unicity
    let mut already_used_names: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    for (k, v) in data {
        let domains: Vec<&str> = k.split("/").collect();
        let domains_size = domains.len();
        //
        let mut parent = String::from("Global");
        let mut level: usize = 1;
        for domain_item in domains {
            // if !domain_item.eq_ignore_ascii_case("v1") {
            if !domain_item.is_empty() {
                let mut value = 0;
                if level == domains_size {
                    value = v;
                }
                //need to ensure name is unique across the tree (cf
                //https://developers.google.com/chart/interactive/docs/gallery/treemap)
                //so for each name, we check if it already exists and if the parent is the same.
                // if this is the same parent, then, it is the same node, if not, we happen a random number to the name to ensure unicity
                let name_str = {
                    let mut name_str = String::from(domain_item);
                    if already_used_names.contains_key(&String::from(&name_str)) {
                        //get the parent
                        let associated_parent =
                            already_used_names.get(&String::from(&name_str)).unwrap();

                        if *associated_parent == parent {
                        } else {
                            debug!(
                                "Same name {:?} for parent {:?} - generate a random number",
                                name_str, parent
                            );
                            name_str.push_str(" [");
                            name_str.push_str(&rand::random::<u16>().to_string());
                            name_str.push_str("]");
                        }
                    }

                    already_used_names.insert(String::from(&name_str), String::from(&parent));

                    name_str
                };

                let node = Node {
                    level: level,
                    name: String::from(&name_str),
                    parent: String::from(&parent),
                    value: value as i32,
                };

                response.insert(node);
                parent = String::from(&name_str);
            }
            // }
            level = level + 1;
        }
    }

    let response_as_vec: Vec<Node> = response.into_iter().collect();
    HttpResponse::Ok().json(response_as_vec)
}

#[get("/v1/domains")]
pub fn get_domains() -> HttpResponse {
    info!("get domains");
    let mut all_domains: Vec<DomainItem> = match list_all_domains(&SETTINGS.database) {
        Ok(all_domains) => all_domains,
        Err(why) => {
            panic!("Unable to get domains: {}", why);
        }
    };

    let mut domains = Vec::new();

    while let Some(domain) = all_domains.pop() {
        let domain = Domain {
            name: domain.name,
            id: domain.id,
            description: domain.description,
            owner: domain.owner,
        };
        domains.push(domain);
    }

    let domains_obj = Domains { domains: domains };

    HttpResponse::Ok().json(domains_obj)
}

#[post("/v1/domains")]
pub fn create_domain(domain: Json<Domain>) -> HttpResponse {
    let uuid = add_domain(
        &SETTINGS.database,
        &domain.name,
        &domain.description,
        &domain.owner,
    )
    .unwrap();

    HttpResponse::Created()
        .header("Location", format!("/v1/domains/{}", uuid))
        .finish()
}

pub fn delete_domain(path: web::Path<(String,)>) -> HttpResponse {
    //path: web::Path<(String,)>,
    //&path.0
    info!("deleting domain for id [{:?}]", &path.0);
    let id = Uuid::parse_str(&path.0).unwrap();

    //check if apis are related to this domain
    let response = match repo_apis::get_apis_per_domain_id(&SETTINGS.database, id) {
        Ok(api) => {
            if api.len() != 0 {
                error!("Domain [{}] has some APIs attached - cannot be deleted", id);
                HttpResponse::PreconditionFailed().json(format!(
                    "Domain [{}] has [{}] apis attached",
                    id,
                    api.len()
                ))
            } else {
                info!("No APIs related to domain [{}]", id);
                repo_domains::delete_domain(&SETTINGS.database, id).unwrap();
                HttpResponse::Ok().json("")
            }
        }
        Err(why) => {
            //if not, delete the domain

            error!("Error while deleting domain [{}] - [{:?}]", id, why);
            HttpResponse::BadRequest().json("Error while deleting domain")
        }
    };

    response
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tier {
    pub name: String,
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tiers {
    pub tiers: Vec<Tier>,
}

#[get("/v1/tiers")]
pub fn get_tiers() -> HttpResponse {
    info!("get tiers");
    let mut all_tiers: Vec<TierItem> = match list_all_tiers(&SETTINGS.database) {
        Ok(all_tiers) => all_tiers,
        Err(why) => {
            panic!("Unable to get domains: {}", why);
        }
    };

    let mut tiers = Vec::new();

    while let Some(tier) = all_tiers.pop() {
        let tier = Tier {
            name: tier.name,
            id: tier.id,
        };
        tiers.push(tier);
    }

    let tiers_obj = Tiers { tiers: tiers };

    HttpResponse::Ok().json(tiers_obj)
}

#[post("/v1/tiers")]
pub fn create_tier(tier: Json<Tier>) -> HttpResponse {
    let uuid = add_tier(&SETTINGS.database, &tier.name).unwrap();

    HttpResponse::Created()
        .header("Location", format!("/v1/tiers/{}", uuid))
        .finish()
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

    repo_apis::add_api(&SETTINGS.database, &api.name, &api.domain_id);

    HttpResponse::Ok().json("")
}

#[get("/v1/apis")]
pub fn list_all_apis() -> HttpResponse {
    info!("list all apis");

    let mut all_apis: Vec<ApiItem> = match repo_apis::list_all_apis(&SETTINGS.database) {
        Ok(all_apis) => all_apis,
        Err(why) => {
            error!("Unable to get apis: {}", why);
            panic!();
        }
    };

    let mut apis = Vec::new();

    while let Some(api) = all_apis.pop() {
        //get domain related to this API
        let domain = match repo_domains::get_domain(&SETTINGS.database, api.domain_id) {
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

fn get_api_by_id(path: web::Path<(String,)>) -> HttpResponse {
    info!("getting api for id [{:?}]", &path.0);
    let api = Uuid::parse_str(&path.0).unwrap();

    let api = repo_apis::get_api_by_id(&SETTINGS.database, api).unwrap();

    let domain = repo_domains::get_domain(&SETTINGS.database, api.domain_id).unwrap();

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

    repo_apis::update_api_status(&SETTINGS.database, status_item).unwrap();

    HttpResponse::Ok().json("")
}

pub fn update_api_tier_by_id(path: web::Path<(String,)>, tier: Json<String>) -> HttpResponse {
    //path: web::Path<(String,)>,
    //&path.0
    info!("updating api for id [{:?}] and tier [{}]", &path.0, tier);

    let api_id = Uuid::parse_str(&path.0).unwrap();
    let tier_id = Uuid::parse_str(tier.as_str()).unwrap();

    repo_apis::update_api_tier(&SETTINGS.database, api_id, tier_id);

    HttpResponse::Ok().json("")
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Envs {
    pub envs: Vec<Env>,
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
    add_env(&SETTINGS.database, &env.name, &env.description);

    HttpResponse::Ok().json("")
}

fn get_env(path: web::Path<(String,)>) -> HttpResponse {
    let env_id = Uuid::parse_str(&path.0).unwrap();

    let response = match repo_envs::get_env(&SETTINGS.database, env_id) {
        Ok(env) => {
            let returned_env = Env {
                id: env.id,
                name: env.name,
                description: env.description,
            };
            debug!("Got Env [{:?}]", returned_env);

            HttpResponse::Ok().json(returned_env)
        }
        Err(_) => {
            debug!("No Env found for api [{:?}]", &path.0);

            HttpResponse::NotFound().finish()
        }
    };

    response
}

#[get("/v1/envs")]
pub fn list_env() -> HttpResponse {
    info!("list envs");

    let mut envs = Envs { envs: Vec::new() };

    let mut all_tuples: Vec<EnvItem> = match list_all_envs(&SETTINGS.database) {
        Ok(all_tuples) => all_tuples,
        Err(why) => {
            debug!("No env found [{:?}]", why);
            Vec::new()
        }
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Metrics {
    pub pr_num: Vec<(DateTime<Utc>, i32)>,
    pub pr_ages: Vec<(DateTime<Utc>, i64, i64, i64, i64)>,
    pub endpoints_num: Vec<(DateTime<Utc>, i32)>, //Vec<(DateTime<Utc>, Option<String>, Option<String>, i32)>,
    pub zally_violations: Vec<(DateTime<Utc>, std::collections::HashMap<i64, usize>)>,
    pub endpoints_num_per_audience: Vec<(DateTime<Utc>, std::collections::HashMap<String, usize>)>,
}

#[get("/v1/metrics")]
pub fn get_all_metrics() -> HttpResponse {
    info!("get all metrics");

    let pr_num_timeseries: Vec<(DateTime<Utc>, i32)> =
        match repo_metrics::get_metrics_pull_requests_number(&SETTINGS.database) {
            Ok(val) => val.points,
            Err(why) => {
                error!(
                    "Error while getting get_metrics_pull_requests_number [{}]",
                    why
                );
                Vec::new()
            }
        };

    let pr_ages_timeseries: Vec<(DateTime<Utc>, i64, i64, i64, i64)> =
        match repo_metrics::get_metrics_pull_requests_ages(&SETTINGS.database) {
            Ok(val) => val.points,
            Err(why) => {
                error!(
                    "Error while getting get_metrics_pull_requests_ages [{}]",
                    why
                );
                Vec::new()
            }
        };

    let endpoints_number: Vec<(DateTime<Utc>, i32)> =
        match repo_metrics::get_metrics_endpoints_number(&SETTINGS.database) {
            Ok(val) => val.points,
            Err(why) => {
                error!("Error while getting get_metrics_endpoints_number [{}]", why);
                Vec::new()
            }
        };

    let zally_ignore_timeseries: Vec<(DateTime<Utc>, std::collections::HashMap<i64, usize>)> =
        match repo_metrics::get_metrics_zally_ignore(&SETTINGS.database) {
            Ok(val) => val.points,
            Err(why) => {
                error!("Error while getting get_metrics_zally_ignore [{}]", why);
                Vec::new()
            }
        };
    let endpoints_audience_number: Vec<(DateTime<Utc>, std::collections::HashMap<String, usize>)> =
        match repo_metrics::get_metrics_endpoints_per_audience(&SETTINGS.database) {
            Ok(val) => val.points,
            Err(why) => {
                error!(
                    "Error while getting get_metrics_endpoints_per_audience [{}]",
                    why
                );
                Vec::new()
            }
        };

    //will combine PR informations with metrics
    let merged_prs: Vec<PullRequest> = get_pull_requests("MERGED").values;
    let merged_prs: Vec<(DateTime<Utc>, PullRequest)> = merged_prs
        .into_iter()
        .map(|val| {
            let dt = chrono::Utc.timestamp(val.closed_epoch.unwrap() / 1000, 0);
            (dt, val)
        })
        .collect();
    //
    // let endpoints_num: Vec<(DateTime<Utc>, i32)> = endpoints_number.points;
    let mut endpoints_num_incl_pr: Vec<(DateTime<Utc>, Option<String>, Option<String>, i32)> =
        Vec::new();
    for tuple in &endpoints_number {
        let date: DateTime<Utc> = tuple.0;
        for pr in &merged_prs {
            match date
                .format("%Y-%m-%d")
                .to_string()
                .starts_with(pr.0.format("%Y-%m-%d").to_string().as_str())
            {
                true => {
                    let annotation = format!(
                        "id: {}, title: {}, author: {}",
                        pr.1.id, pr.1.title, pr.1.author.user.email_address,
                    );
                    endpoints_num_incl_pr.push((
                        date,
                        Some(pr.1.id.to_string()),
                        Some(annotation),
                        tuple.1,
                    ));
                    break;
                }
                false => endpoints_num_incl_pr.push((date, None, None, tuple.1)),
            }
        }
    }

    //
    let metrics = Metrics {
        pr_num: pr_num_timeseries,
        pr_ages: pr_ages_timeseries,
        endpoints_num: endpoints_number,
        endpoints_num_per_audience: endpoints_audience_number,
        zally_violations: zally_ignore_timeseries,
    };

    HttpResponse::Ok().json(metrics)
}

#[derive(Serialize, Deserialize, Debug)]
struct PullRequests {
    size: i32,
    limit: i32,
    #[serde(rename(serialize = "isLastPage", deserialize = "isLastPage"))]
    is_last_page: bool,
    values: Vec<PullRequest>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PullRequest {
    id: i32,
    version: i32,
    title: String,
    state: String,
    #[serde(rename(serialize = "createdDate", deserialize = "createdDate"))]
    created_epoch: u64,
    #[serde(rename(serialize = "closedDate", deserialize = "closedDate"))]
    closed_epoch: Option<i64>,
    author: Author,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Author {
    user: User,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    #[serde(rename(serialize = "displayName", deserialize = "displayName"))]
    display_name: String,
    #[serde(rename(serialize = "emailAddress", deserialize = "emailAddress"))]
    email_address: String,
}

#[get("/v1/metrics/pull-requests")]
pub fn get_oldest_pr() -> HttpResponse {
    let limit = 3;
    info!("get oldest pull-request");
    let pull_requests: PullRequests = get_pull_requests("OPEN");

    let current_epoch = std::time::SystemTime::now();
    let current_epoch = current_epoch.duration_since(std::time::UNIX_EPOCH).unwrap();
    let current_epoch = current_epoch.as_secs();
    //
    let mut pull_requests: Vec<_> = pull_requests
        .values
        .iter()
        .map(|val| {
            let created_epoch_in_sec = val.created_epoch / 1000;
            if current_epoch < created_epoch_in_sec {
                //val.created_epoch is in ms
                error!(
                    "Cannot compute epoch elapse as current epoch [{}] < obtained epoch [{}]",
                    current_epoch, val.created_epoch
                );
            }
            let delta: u64 = current_epoch - created_epoch_in_sec;

            (val, delta)
        })
        .collect();
    pull_requests.sort_by(|a, b| a.1.cmp(&b.1).reverse());
    let pull_requests: Vec<_> = pull_requests.iter().map(|val| val.0).take(limit).collect();

    //
    HttpResponse::Ok().json(pull_requests)
}

#[get("/v1/metrics/merged-pull-requests")]
pub fn get_merged_pr() -> HttpResponse {
    info!("get merged pull-request");
    let pull_requests: PullRequests = get_pull_requests("MERGED");

    let pull_requests: Vec<_> = pull_requests.values;
    //
    HttpResponse::Ok().json(pull_requests)
}

fn get_pull_requests(status: &str) -> PullRequests {
    let access_token = SETTINGS.stash_config.access_token.clone();
    let client = Client::new();

    let url = format!(
        "{}/pull-requests?state={}&limit=1000",
        SETTINGS.stash_config.base_uri, status
    );
    let mut resp = client
        .get(url.as_str())
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .unwrap();

    debug!("Calling {} - got HTTP Status {:?}", url, resp.status());
    //TODO manage unwrap withe resp.status().is_success() or is_server_error()
    let pull_requests: PullRequests = resp.json().unwrap();

    pull_requests
}

#[post("/v1/metrics/refresh")]
pub fn refresh_metrics() -> HttpResponse {
    info!("refresh metrics");
    catalog::refresh_git_repo(&SETTINGS.catalog_path);
    //
    let pull_requests: PullRequests = get_pull_requests("OPEN");

    //keep metric pr_num
    let metrics = get_metrics_pull_requests_number(&pull_requests);
    repo_metrics::save_metrics_pull_requests_number(&SETTINGS.database, metrics.0, metrics.1)
        .unwrap();
    //keep metric pr_age
    let current_epoch = std::time::SystemTime::now();
    let current_epoch = current_epoch.duration_since(std::time::UNIX_EPOCH).unwrap();
    let metrics = get_metrics_pull_requests_ages_stats(&pull_requests, current_epoch.as_secs());
    repo_metrics::save_metrics_pull_requests_ages(
        &SETTINGS.database,
        metrics.0,
        isize::try_from(metrics.1).unwrap(),
        isize::try_from(metrics.2).unwrap(),
        isize::try_from(metrics.3).unwrap(),
        isize::try_from(metrics.4).unwrap(),
    )
    .unwrap();

    //get # of endpoints
    let all_specs: Vec<SpecItem> = catalog::list_specs(SETTINGS.catalog_path.as_str());

    let all_specs_paths: Vec<String> = all_specs.iter().map(|val| val.path.to_string()).collect();
    info!(
        "List of retrieved and parsed OpenAPI Specifications [{:?}]",
        all_specs_paths
    );

    let len = &all_specs.len();
    let metrics = get_metrics_endpoints_num(&all_specs);
    info!(
        "Parsed [{}] specifications and got a total of [{:?}] paths",
        len, &metrics
    );
    repo_metrics::save_metrics_endpoints_num(&SETTINGS.database, metrics.0, metrics.1).unwrap();

    //save metrics zally_ignore
    let stats = catalog::get_zally_ignore(&all_specs);
    repo_metrics::save_metrics_zally_ignore(&SETTINGS.database, Utc::now(), stats).unwrap();

    //save metrics endpoints_num_per audience
    let stats = catalog::get_endpoints_num_per_audience(&all_specs);
    repo_metrics::save_metrics_endpoints_num_per_audience(&SETTINGS.database, Utc::now(), stats)
        .unwrap();
    //
    HttpResponse::Ok().json(pull_requests.size)
}

fn get_metrics_pull_requests_number(pull_requests: &PullRequests) -> (DateTime<Utc>, i32) {
    (Utc::now(), pull_requests.size)
}

fn get_metrics_pull_requests_ages_stats(
    pull_requests: &PullRequests,
    current_epoch: u64,
) -> (DateTime<Utc>, u64, u64, u64, u64) {
    let mut histogram = Histogram::new();
    //
    let elapse_in_ms: Vec<_> = pull_requests
        .values
        .iter()
        .map(|val| {
            let created_epoch_in_sec = val.created_epoch / 1000;
            if current_epoch < created_epoch_in_sec {
                //val.created_epoch is in ms
                error!(
                    "Cannot compute epoch elapse as current epoch [{}] < obtained epoch [{}]",
                    current_epoch, val.created_epoch
                );
            }
            let delta: u64 = current_epoch - created_epoch_in_sec;
            //TODO clean unwrap
            histogram.increment(delta / 86400).unwrap(); //keep values in days

            delta
        })
        .collect();

    debug!(
        "Got Percentiles: p0: {} days p50: {} days p100: {} days mean: {} days",
        histogram.percentile(0.0).unwrap(),
        histogram.percentile(50.0).unwrap(),
        histogram.percentile(100.0).unwrap(),
        histogram.mean().unwrap(),
    );

    (
        Utc::now(),
        histogram.percentile(0.0).unwrap(),
        histogram.percentile(50.0).unwrap(),
        histogram.percentile(100.0).unwrap(),
        histogram.mean().unwrap(),
    )
}

//TODO move this method into catalog/mod.rs
fn get_metrics_endpoints_num(all_specs: &Vec<SpecItem>) -> (DateTime<Utc>, i32) {
    let endpoints_per_spec: Vec<_> = all_specs
        .iter()
        .map(|spec| {
            let num = spec.api_spec.paths.len();
            debug!("# of paths - spec [{:?}] got [{:?}] paths", spec.path, num);

            num
        })
        .collect();

    let total: i32 = endpoints_per_spec.iter().sum::<usize>() as i32;
    info!(
        "# of paths - per spec [{:?}] - and total # of paths [{}]",
        &endpoints_per_spec, &total
    );

    (Utc::now(), total)
}

/**
 * To server static pages
 */
async fn index(_req: HttpRequest) -> Result<NamedFile> {
    let mut path: PathBuf = PathBuf::from(&SETTINGS.server.static_resources_path);
    path.push("index.html");
    Ok(NamedFile::open(path)?)
}

lazy_static! {
    static ref SETTINGS: settings::Settings = Settings::new().unwrap();
}

/**
 *
 */
#[actix_rt::main]
async fn main() {
    //TODO std::env::set_var("RUST_LOG", "actix_web=info");
    //env_logger::init();

    // let colors = fern::colors::ColoredLevelConfig::new()
    //     .debug(fern::colors::Color::Blue)
    //     .info(fern::colors::Color::Green)
    //     .warn(fern::colors::Color::Yellow)
    //     .error(fern::colors::Color::Red);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}] - [{}] - [{}] - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                //colors.color(record.level()),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("logs.log").unwrap())
        .apply()
        .unwrap();

    // Create a new scheduler for Utc
    // let mut scheduler = Scheduler::new();
    // // Add some tasks to it
    // scheduler.every(Weekday).at("23:30").run(|| {
    //     let client =  Client::new();
    //     client.post(format!("http://{}/v1/metrics/refresh", &SETTINGS.server.bind_adress).as_str()).send().unwrap();
    // });

    // scheduler.every(10.seconds()).run(|| println!("Periodic task"));
    // let _thread_handle = scheduler.watch_thread(Duration::from_millis(100));

    //start HTTP Server
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .route("/v1/endpoints", web::get().to(get_endpoints))
            .service(web::resource("/v1/endpoints/{api}").route(web::get().to(get_endpoints))) //TODO rework url
            .service(add_deployment)
            .service(get_deployments)
            .service(
                web::resource("/v1/deployments/{api}")
                    .route(web::get().to(get_deployments_for_api)),
            ) //TODO rework url
            .service(get_domains)
            .service(get_domains_stats)
            .service(create_domain)
            .service(get_domains_errors)
            .service(
                web::scope("/v1/domains")
                    .service(web::resource("/{id}").route(web::delete().to(delete_domain))),
            )
            .service(get_all_specs)
            .service(create_api)
            .service(list_all_apis)
            .service(
                web::scope("/v1/apis")
                    .service(web::resource("/{api}").route(web::get().to(get_api_by_id)))
                    .service(
                        web::resource("/{api}/status")
                            .route(web::post().to(update_api_status_by_id)),
                    )
                    .service(
                        web::resource("/{api}/tier").route(web::post().to(update_api_tier_by_id)),
                    ),
            )
            .service(create_env)
            .service(list_env)
            .service(create_tier)
            .service(get_tiers)
            .service(web::resource("/v1/envs/{id}").route(web::get().to(get_env)))
            .service(get_all_metrics)
            .service(get_oldest_pr)
            .service(get_merged_pr)
            .service(refresh_metrics)
            .route("/static", web::get().to(index))
            .route("/", web::get().to(index))
            .route("/domains", web::get().to(index))
            .route("/apis", web::get().to(index))
            .route("/envs", web::get().to(index))
            .service(
                Files::new("/", &SETTINGS.server.static_resources_path).index_file("index.html"),
            )
    })
    .workers(4)
    .bind(&SETTINGS.server.bind_adress)
    .unwrap()
    .run()
    .await
    .unwrap();
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    #[test]
    fn test_metrics_get_pr_number() {
        let response = r#"{"size":2,"limit":2,"isLastPage":false,"values":[{"id":57,"version":14,"title":"XXX API for currencies.","description":"XXX (partial) API.\nOnly exposes currencies list","state":"OPEN","open":true,"closed":false,"createdDate":1582305198106,"updatedDate":1585062047626,"fromRef":{"id":"refs/heads/xxx","displayId":"xxx","latestCommit":"486e8c0b301114fcbfc53bdb4e4884765c7122db","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"toRef":{"id":"refs/heads/master","displayId":"master","latestCommit":"eb26e8472c9beb4da8779b9783a2bbb68f176af1","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"locked":false,"author":{"user":{"name":"","emailAddress":"...","id":2811,"displayName":"W","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"AUTHOR","approved":false,"status":"UNAPPROVED"},"reviewers":[{"user":{"name":"","emailAddress":"...","id":1504,"displayName":"L","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"....","id":2511,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":2083,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"-ci","emailAddress":".....","id":8003,"displayName":"jenkins-ci","active":true,"slug":"-ci","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"486e8c0b301114fcbfc53bdb4e4884765c7122db","role":"REVIEWER","approved":true,"status":"APPROVED"},{"user":{"name":"","emailAddress":"....","id":1283,"displayName":"W","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":".....","id":4304,"displayName":"L","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"}],"participants":[],"properties":{"mergeResult":{"outcome":"CLEAN","current":true},"resolvedTaskCount":0,"commentCount":10,"openTaskCount":0},"links":{"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/pull-requests/57"}]}},{"id":1,"version":93,"title":"Marketdata","description":"* Add 3 yamls about APIs for Service [MDS (w/ interpolation)] described under wiki https://my_wiki","state":"OPEN","open":true,"closed":false,"createdDate":1551955373000,"updatedDate":1582726600363,"fromRef":{"id":"refs/heads/marketdata","displayId":"marketdata","latestCommit":"3947e71bd4e152d6f1b93b63232b32d09fa5562e","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"toRef":{"id":"refs/heads/master","displayId":"master","latestCommit":"eb26e8472c9beb4da8779b9783a2bbb68f176af1","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"locked":false,"author":{"user":{"name":"","emailAddress":"...","id":4215,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"AUTHOR","approved":false,"status":"UNAPPROVED"},"reviewers":[{"user":{"name":"","emailAddress":"....","id":1283,"displayName":"W","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":435,"displayName":"B","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"0fe3dff0f1a9415d35bddf0ffc004da155e5c26e","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":4436,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":3070,"displayName":"S","active":true,"slug":"dsubtil","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"..","id":2511,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"0fe3dff0f1a9415d35bddf0ffc004da155e5c26e","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"..","id":2842,"displayName":"E","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"6106a3ea81bd9fbbed4a7ccf694f572745040297","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"d","emailAddress":"...","id":2083,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"26d762f1c1242d3f2c29a328526154c13c923077","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"-ci","emailAddress":".....","id":8003,"displayName":"jenkins-ci","active":true,"slug":"-ci","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"3947e71bd4e152d6f1b93b63232b32d09fa5562e","role":"REVIEWER","approved":true,"status":"APPROVED"}],"participants":[{"user":{"name":"","emailAddress":"...","id":1857,"displayName":"S","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"J....","id":3941,"displayName":"C","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"....","id":784,"displayName":"e","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/us"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"......","id":1483,"displayName":"S","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":".....","id":2862,"displayName":"S","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"}],"properties":{"mergeResult":{"outcome":"CLEAN","current":true},"resolvedTaskCount":1,"commentCount":86,"openTaskCount":1},"links":{"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/pull-requests/1"}]}}],"start":0,"nextPageStart":2}"#;
        let pull_requests: super::PullRequests = serde_json::from_str(response).unwrap();

        let metrics = super::get_metrics_pull_requests_number(&pull_requests);
        assert_eq!(2, metrics.1);
    }

    #[test]
    fn test_metrics_get_pr_ages() {
        let response = r#"{"size":2,"limit":2,"isLastPage":false,"values":[{"id":57,"version":14,"title":"XXX API for currencies.","description":"XXX (partial) API.\nOnly exposes currencies list","state":"OPEN","open":true,"closed":false,"createdDate":1582305198106,"updatedDate":1585062047626,"fromRef":{"id":"refs/heads/xxx","displayId":"xxx","latestCommit":"486e8c0b301114fcbfc53bdb4e4884765c7122db","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"toRef":{"id":"refs/heads/master","displayId":"master","latestCommit":"eb26e8472c9beb4da8779b9783a2bbb68f176af1","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"locked":false,"author":{"user":{"name":"","emailAddress":"...","id":2811,"displayName":"W","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"AUTHOR","approved":false,"status":"UNAPPROVED"},"reviewers":[{"user":{"name":"","emailAddress":"...","id":1504,"displayName":"L","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"....","id":2511,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":2083,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"-ci","emailAddress":".....","id":8003,"displayName":"jenkins-ci","active":true,"slug":"-ci","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"486e8c0b301114fcbfc53bdb4e4884765c7122db","role":"REVIEWER","approved":true,"status":"APPROVED"},{"user":{"name":"","emailAddress":"....","id":1283,"displayName":"W","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":".....","id":4304,"displayName":"L","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"}],"participants":[],"properties":{"mergeResult":{"outcome":"CLEAN","current":true},"resolvedTaskCount":0,"commentCount":10,"openTaskCount":0},"links":{"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/pull-requests/57"}]}},{"id":1,"version":93,"title":"Marketdata","description":"* Add 3 yamls about APIs for Service [MDS (w/ interpolation)] described under wiki https://my_wiki","state":"OPEN","open":true,"closed":false,"createdDate":1551955373000,"updatedDate":1582726600363,"fromRef":{"id":"refs/heads/marketdata","displayId":"marketdata","latestCommit":"3947e71bd4e152d6f1b93b63232b32d09fa5562e","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"toRef":{"id":"refs/heads/master","displayId":"master","latestCommit":"eb26e8472c9beb4da8779b9783a2bbb68f176af1","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"locked":false,"author":{"user":{"name":"","emailAddress":"...","id":4215,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"AUTHOR","approved":false,"status":"UNAPPROVED"},"reviewers":[{"user":{"name":"","emailAddress":"....","id":1283,"displayName":"W","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":435,"displayName":"B","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"0fe3dff0f1a9415d35bddf0ffc004da155e5c26e","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":4436,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":3070,"displayName":"S","active":true,"slug":"dsubtil","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"..","id":2511,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"0fe3dff0f1a9415d35bddf0ffc004da155e5c26e","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"..","id":2842,"displayName":"E","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"6106a3ea81bd9fbbed4a7ccf694f572745040297","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"d","emailAddress":"...","id":2083,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"26d762f1c1242d3f2c29a328526154c13c923077","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"-ci","emailAddress":".....","id":8003,"displayName":"jenkins-ci","active":true,"slug":"-ci","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"3947e71bd4e152d6f1b93b63232b32d09fa5562e","role":"REVIEWER","approved":true,"status":"APPROVED"}],"participants":[{"user":{"name":"","emailAddress":"...","id":1857,"displayName":"S","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"J....","id":3941,"displayName":"C","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"....","id":784,"displayName":"e","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/us"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"......","id":1483,"displayName":"S","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":".....","id":2862,"displayName":"S","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"}],"properties":{"mergeResult":{"outcome":"CLEAN","current":true},"resolvedTaskCount":1,"commentCount":86,"openTaskCount":1},"links":{"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/pull-requests/1"}]}}],"start":0,"nextPageStart":2}"#;
        let pull_requests: super::PullRequests = serde_json::from_str(response).unwrap();
        let current_epoch = std::time::SystemTime::now();
        //fix date
        let dt = DateTime::parse_from_rfc2822("Sun, 29 Mar 2020 20:36:29 +0000").unwrap();
        let metrics = super::get_metrics_pull_requests_ages_stats(
            &pull_requests,
            dt.timestamp() as u64, /*current_epoch.as_secs()*/
        );

        assert_eq!(37, metrics.1);
        assert_eq!(388, metrics.2);
        assert_eq!(388, metrics.3);
        assert_eq!(213, metrics.4);
    }
}
