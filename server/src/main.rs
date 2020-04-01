extern crate log;
extern crate env_logger;
extern crate uuid;

extern crate reqwest;
use reqwest::{Client, Response};

extern crate config;

use chrono::{DateTime, Utc};

use log::{debug,info, error};

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
use dao::repo_apis;
use dao::repo_metrics;

use repo_deployments::{*};
use repo_domains::{*};
use repo_envs::{*};
use repo_apis::{*};
use repo_metrics::{*};
use catalog::{*};

mod settings;
use settings::Settings;

#[macro_use]
extern crate lazy_static;

extern crate histogram;
use histogram::Histogram;

use std::convert::TryFrom;


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

    let mut all_apis = catalog::get_spec(SETTINGS.catalog_path.as_str(), &info.0);

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

    let mut all_specs = catalog::list_specs(SETTINGS.catalog_path.as_str());
    while let Some(spec) = all_specs.pop() {
        info!("Analysing file [{:?}]", spec.name);
        let short_path = &spec.name[SETTINGS.catalog_dir.as_str().len()..spec.name.len()];
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
    release(&SETTINGS.database, deployment.api.clone(), deployment.env.clone());

    HttpResponse::Ok().json("")
}

#[get("/v1/deployments")]
fn get_deployments() -> HttpResponse {
    let mut deployments = Deployments {
        deployments : Vec::new(),
    };

    let mut all_tuples: Vec<(String, String)> = match list_all_deployments(&SETTINGS.database) {
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

    let mut all_tuples: Vec<(String, String)> = match get_all_deployments_for_api(&SETTINGS.database, &path.0) {
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
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Domains {
    pub domains: Vec<Domain>
}

#[get("/v1/domains")]
pub fn get_domains() -> HttpResponse {
    info!("get domains");
    let mut all_domains: Vec<DomainItem> = match list_all_domains(&SETTINGS.database) {
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
            description: domain.description,
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
    let uuid = add_domain(&SETTINGS.database, &domain.name, &domain.description).unwrap();

    HttpResponse::Created().header("Location", format!("/v1/domains/{}", uuid)).finish()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Api {
    pub id: Uuid,
    pub name: String, 
    pub domain_id: Uuid, 
    pub domain_name: String,
    pub spec_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Apis {
    pub apis: Vec<Api>
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
            panic!("Unable to get apis: {}", why);
        },
    };

    let mut apis = Vec::new();

    while let Some(api) = all_apis.pop() {
        //get domain related to this API
        let domain = repo_domains::get_domain(&SETTINGS.database, api.domain_id).unwrap();
        //
        let api = Api {
            name: api.name,
            id: api.id,
            domain_id: domain.id, 
            domain_name: domain.name,
            spec_ids: Vec::new(), //TODO
        };
        apis.push(api);
    }

    let apis_obj = Apis{
            apis: apis,
    };

    HttpResponse::Ok().json(apis_obj)
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
    add_env(&SETTINGS.database, &env.name, &env.description);

    HttpResponse::Ok().json("")
}

#[get("/v1/envs")]
pub fn list_env() -> HttpResponse {
    info!("list envs");

    let mut envs = Envs {
        envs: Vec::new(),
    };

    let mut all_tuples: Vec<EnvItem> = match list_all_envs(&SETTINGS.database) {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Metrics {
    pub pr_num: Vec<(DateTime<Utc>, i32)>,
    pub pr_ages: Vec<(DateTime<Utc>, i64, i64, i64, i64)>,
    pub endpoints_num: Vec<(DateTime<Utc>, i32)>,
}

#[get("/v1/metrics")]
pub fn get_all_metrics() -> HttpResponse {
    info!("get all metrics");
    
    let pr_num_timeseries: TimeSeries = repo_metrics::get_metrics_pull_requests_number(&SETTINGS.database).unwrap();
    let pr_ages_timeseries: TupleTimeSeries = repo_metrics::get_metrics_pull_requests_ages(&SETTINGS.database).unwrap();
    let endpoints_number: TimeSeries = repo_metrics::get_metrics_endpoints_number(&SETTINGS.database).unwrap();
    //
    let metrics = Metrics {
        pr_num: pr_num_timeseries.points,
        pr_ages: pr_ages_timeseries.points,
        endpoints_num : endpoints_number.points,
    };

    HttpResponse::Ok().json(metrics)
}

#[derive(Serialize, Deserialize, Debug)]
struct PullRequests {
    size: i32,
    limit: i32,
    #[serde(rename(serialize = "isLastPage", deserialize = "isLastPage"))]
    is_last_page: bool,
    values: Vec<PullRequest>
}

#[derive(Serialize, Deserialize, Debug)]
struct PullRequest {
    id: i32,
    version: i32,
    title: String, 
    state: String, 
    #[serde(rename(serialize = "createdDate", deserialize = "createdDate"))]
    created_epoch: u64,
}

#[derive(Deserialize, Debug)]
pub struct Info {
    username: Option<String>, 
    password: Option<String>,
}

#[post("/v1/metrics/refresh")]
pub fn refresh_metrics(info: web::Query<Info>) -> HttpResponse {
    info!("refresh metrics");

    let mut username = SETTINGS.stash_config.user.clone();
    match &info.username  {
        Some(name) => {
            username = name.to_string();
            debug!("Will use settings in url for username");
        }, 
        None => debug!("Will use settings in config for username"),
    };
    let mut pwd = SETTINGS.stash_config.pwd.clone();
    match &info.password  {
        Some(name) => {
            pwd = name.to_string();
            debug!("Will use settings in url for password");
        }, 
        None => debug!("Will use settings in config for password"),
    };

    let client =  Client::new();
    
    let url = format!("{}/pull-requests?state=OPEN&limit=1000", SETTINGS.stash_config.base_uri);
    let mut resp = client.get(url.as_str())
        .basic_auth(username, Some(pwd))
        .send().unwrap();

    debug!("HTTP Status {:?}", resp.status());

    let pull_requests: PullRequests =  resp.json().unwrap();

    debug!("body: {:?}", pull_requests);

    //keep metric pr_num
    let metrics = get_metrics_pull_requests_number(&pull_requests);
    repo_metrics::save_metrics_pull_requests_number(&SETTINGS.database, metrics.0, metrics.1);
    //keep metric pr_age
    let current_epoch = std::time::SystemTime::now();
    let current_epoch = current_epoch.duration_since(std::time::UNIX_EPOCH).unwrap();
    let metrics = get_metrics_pull_requests_ages(&pull_requests, current_epoch.as_secs());
    repo_metrics::save_metrics_pull_requests_ages(&SETTINGS.database, metrics.0, 
        isize::try_from(metrics.1).unwrap(), 
        isize::try_from(metrics.2).unwrap(),
        isize::try_from(metrics.3).unwrap(),
        isize::try_from(metrics.4).unwrap()
    );
    //get # of endpoints
    let all_specs : Vec<SpecItem> = catalog::list_specs(SETTINGS.catalog_path.as_str());
    let len = &all_specs.len();
    let metrics = get_metrics_endpoints_num(all_specs);
    debug!("Got [{}] specifications and [{:?}] endpoints", len, &metrics);
    repo_metrics::save_metrics_endpoints_num(&SETTINGS.database, metrics.0, metrics.1);
    //
    HttpResponse::Ok().json(pull_requests.size)
}

fn get_metrics_pull_requests_number(pull_requests: &PullRequests) -> (DateTime<Utc>, i32) {
    (Utc::now(), pull_requests.size)
}

fn get_metrics_pull_requests_ages(pull_requests: &PullRequests, current_epoch: u64) -> (DateTime<Utc>, u64, u64, u64, u64) {
    let mut histogram = Histogram::new();
    //
    let elapse_in_ms : Vec<_> = pull_requests.values.iter().map( |val|  {
        let created_epoch_in_sec = val.created_epoch/1000;
        if current_epoch < created_epoch_in_sec { //val.created_epoch is in ms
            error!("Cannot compute epoch elapse as current epoch [{}] < obtained epoch [{}]", current_epoch, val.created_epoch);
        }
        let delta : u64 = (current_epoch - created_epoch_in_sec);
        histogram.increment(delta / 86400); //keep values in days

        delta
    } ).collect();

    debug!("Got Percentiles: p0: {} days p50: {} days p100: {} days mean: {} days",
        histogram.percentile(0.0).unwrap(),
        histogram.percentile(50.0).unwrap(),
        histogram.percentile(100.0).unwrap(),
        histogram.mean().unwrap(),
    );

    (Utc::now(), histogram.percentile(0.0).unwrap(), histogram.percentile(50.0).unwrap(), histogram.percentile(100.0).unwrap(), histogram.mean().unwrap())
}

fn get_metrics_endpoints_num(all_specs : Vec<SpecItem>) -> (DateTime<Utc>, i32) {
    let endpoints_per_spec : Vec<_> = all_specs.iter().map(|spec| {
            spec.api_spec.paths.len()
        }).collect();

    let total: i32 = endpoints_per_spec.iter().sum::<usize>() as i32;
    debug!("Got # of endpoints per spec [{:?}] - and total # of endpoints [{}]", &endpoints_per_spec, &total);

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
    static ref SETTINGS : settings::Settings = Settings::new().unwrap();
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
            .service(list_all_apis)
            .service(create_env)
            .service(list_env)
            .service(get_all_metrics)
            .service(refresh_metrics)
            .route("/static", web::get().to(index))
            .route("/", web::get().to(index))
            .route("/domains", web::get().to(index))
            .route("/apis", web::get().to(index))
            .route("/env", web::get().to(index))
            .service(Files::new("/", &SETTINGS.server.static_resources_path).index_file("index.html"))
    })
    .workers(4)
    .bind(&SETTINGS.server.bind_adress)  
    .unwrap()
    .run()
    .await;
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime};

    #[test]
    fn test_metrics_get_pr_number() {
        let response = r#"{"size":2,"limit":2,"isLastPage":false,"values":[{"id":57,"version":14,"title":"XXX API for currencies.","description":"XXX (partial) API.\nOnly exposes currencies list","state":"OPEN","open":true,"closed":false,"createdDate":1582305198106,"updatedDate":1585062047626,"fromRef":{"id":"refs/heads/xxx","displayId":"xxx","latestCommit":"486e8c0b301114fcbfc53bdb4e4884765c7122db","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"toRef":{"id":"refs/heads/master","displayId":"master","latestCommit":"eb26e8472c9beb4da8779b9783a2bbb68f176af1","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"locked":false,"author":{"user":{"name":"","emailAddress":"...","id":2811,"displayName":"W","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"AUTHOR","approved":false,"status":"UNAPPROVED"},"reviewers":[{"user":{"name":"","emailAddress":"...","id":1504,"displayName":"L","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"....","id":2511,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":2083,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"-ci","emailAddress":".....","id":8003,"displayName":"jenkins-ci","active":true,"slug":"-ci","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"486e8c0b301114fcbfc53bdb4e4884765c7122db","role":"REVIEWER","approved":true,"status":"APPROVED"},{"user":{"name":"","emailAddress":"....","id":1283,"displayName":"W","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":".....","id":4304,"displayName":"L","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"}],"participants":[],"properties":{"mergeResult":{"outcome":"CLEAN","current":true},"resolvedTaskCount":0,"commentCount":10,"openTaskCount":0},"links":{"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/pull-requests/57"}]}},{"id":1,"version":93,"title":"Marketdata","description":"* Add 3 yamls about APIs for Service [MDS (w/ interpolation)] described under wiki https://my_wiki","state":"OPEN","open":true,"closed":false,"createdDate":1551955373000,"updatedDate":1582726600363,"fromRef":{"id":"refs/heads/marketdata","displayId":"marketdata","latestCommit":"3947e71bd4e152d6f1b93b63232b32d09fa5562e","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"toRef":{"id":"refs/heads/master","displayId":"master","latestCommit":"eb26e8472c9beb4da8779b9783a2bbb68f176af1","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"locked":false,"author":{"user":{"name":"","emailAddress":"...","id":4215,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"AUTHOR","approved":false,"status":"UNAPPROVED"},"reviewers":[{"user":{"name":"","emailAddress":"....","id":1283,"displayName":"W","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":435,"displayName":"B","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"0fe3dff0f1a9415d35bddf0ffc004da155e5c26e","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":4436,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":3070,"displayName":"S","active":true,"slug":"dsubtil","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"..","id":2511,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"0fe3dff0f1a9415d35bddf0ffc004da155e5c26e","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"..","id":2842,"displayName":"E","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"6106a3ea81bd9fbbed4a7ccf694f572745040297","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"d","emailAddress":"...","id":2083,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"26d762f1c1242d3f2c29a328526154c13c923077","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"-ci","emailAddress":".....","id":8003,"displayName":"jenkins-ci","active":true,"slug":"-ci","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"3947e71bd4e152d6f1b93b63232b32d09fa5562e","role":"REVIEWER","approved":true,"status":"APPROVED"}],"participants":[{"user":{"name":"","emailAddress":"...","id":1857,"displayName":"S","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"J....","id":3941,"displayName":"C","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"....","id":784,"displayName":"e","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/us"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"......","id":1483,"displayName":"S","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":".....","id":2862,"displayName":"S","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"}],"properties":{"mergeResult":{"outcome":"CLEAN","current":true},"resolvedTaskCount":1,"commentCount":86,"openTaskCount":1},"links":{"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/pull-requests/1"}]}}],"start":0,"nextPageStart":2}"#;
        let pull_requests : super::PullRequests = serde_json::from_str(response).unwrap();

        let metrics = super::get_metrics_pull_requests_number(&pull_requests);
        assert_eq!(2, metrics.1);
    }

    #[test]
    fn test_metrics_get_pr_ages() {
        let response = r#"{"size":2,"limit":2,"isLastPage":false,"values":[{"id":57,"version":14,"title":"XXX API for currencies.","description":"XXX (partial) API.\nOnly exposes currencies list","state":"OPEN","open":true,"closed":false,"createdDate":1582305198106,"updatedDate":1585062047626,"fromRef":{"id":"refs/heads/xxx","displayId":"xxx","latestCommit":"486e8c0b301114fcbfc53bdb4e4884765c7122db","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"toRef":{"id":"refs/heads/master","displayId":"master","latestCommit":"eb26e8472c9beb4da8779b9783a2bbb68f176af1","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"locked":false,"author":{"user":{"name":"","emailAddress":"...","id":2811,"displayName":"W","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"AUTHOR","approved":false,"status":"UNAPPROVED"},"reviewers":[{"user":{"name":"","emailAddress":"...","id":1504,"displayName":"L","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"....","id":2511,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":2083,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"-ci","emailAddress":".....","id":8003,"displayName":"jenkins-ci","active":true,"slug":"-ci","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"486e8c0b301114fcbfc53bdb4e4884765c7122db","role":"REVIEWER","approved":true,"status":"APPROVED"},{"user":{"name":"","emailAddress":"....","id":1283,"displayName":"W","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":".....","id":4304,"displayName":"L","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"}],"participants":[],"properties":{"mergeResult":{"outcome":"CLEAN","current":true},"resolvedTaskCount":0,"commentCount":10,"openTaskCount":0},"links":{"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/pull-requests/57"}]}},{"id":1,"version":93,"title":"Marketdata","description":"* Add 3 yamls about APIs for Service [MDS (w/ interpolation)] described under wiki https://my_wiki","state":"OPEN","open":true,"closed":false,"createdDate":1551955373000,"updatedDate":1582726600363,"fromRef":{"id":"refs/heads/marketdata","displayId":"marketdata","latestCommit":"3947e71bd4e152d6f1b93b63232b32d09fa5562e","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"toRef":{"id":"refs/heads/master","displayId":"master","latestCommit":"eb26e8472c9beb4da8779b9783a2bbb68f176af1","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"locked":false,"author":{"user":{"name":"","emailAddress":"...","id":4215,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"AUTHOR","approved":false,"status":"UNAPPROVED"},"reviewers":[{"user":{"name":"","emailAddress":"....","id":1283,"displayName":"W","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":435,"displayName":"B","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"0fe3dff0f1a9415d35bddf0ffc004da155e5c26e","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":4436,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":3070,"displayName":"S","active":true,"slug":"dsubtil","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"..","id":2511,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"0fe3dff0f1a9415d35bddf0ffc004da155e5c26e","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"..","id":2842,"displayName":"E","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"6106a3ea81bd9fbbed4a7ccf694f572745040297","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"d","emailAddress":"...","id":2083,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"26d762f1c1242d3f2c29a328526154c13c923077","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"-ci","emailAddress":".....","id":8003,"displayName":"jenkins-ci","active":true,"slug":"-ci","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"3947e71bd4e152d6f1b93b63232b32d09fa5562e","role":"REVIEWER","approved":true,"status":"APPROVED"}],"participants":[{"user":{"name":"","emailAddress":"...","id":1857,"displayName":"S","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"J....","id":3941,"displayName":"C","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"....","id":784,"displayName":"e","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/us"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"......","id":1483,"displayName":"S","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":".....","id":2862,"displayName":"S","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"}],"properties":{"mergeResult":{"outcome":"CLEAN","current":true},"resolvedTaskCount":1,"commentCount":86,"openTaskCount":1},"links":{"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/pull-requests/1"}]}}],"start":0,"nextPageStart":2}"#;
        let pull_requests : super::PullRequests = serde_json::from_str(response).unwrap();
        let current_epoch = std::time::SystemTime::now();
        //fix date
        let dt = DateTime::parse_from_rfc2822("Sun, 29 Mar 2020 20:36:29 +0000").unwrap();
        let metrics = super::get_metrics_pull_requests_ages(&pull_requests, dt.timestamp() as u64 /*current_epoch.as_secs()*/);

        assert_eq!(37, metrics.1);
        assert_eq!(388, metrics.2);
        assert_eq!(388, metrics.3);
        assert_eq!(213, metrics.4);
    }
}
