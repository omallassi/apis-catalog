use actix_web::web::Json;
use actix_web::{get, post};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use std::hash::{Hash, Hasher};

#[path = "../dao/mod.rs"]
mod dao;
use dao::catalog::*;
use dao::repo_domains::*;

#[path = "./apis.rs"]
mod apis;

use log::{debug, error, info};

#[path = "../settings/mod.rs"]
mod settings;
use settings::Settings;

use uuid::Uuid;

lazy_static! {
    static ref SETTINGS: settings::Settings = Settings::new().unwrap();
}

/*
 * domain related APIs
 */

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
    let all_specs: Vec<SpecItem> = dao::catalog::list_specs(SETTINGS.catalog_path.as_str());
    //at this stage data = {"N/A - servers not specified": 11, "/v1/settlement/operational-arrangement": 8, "/v1/market-risk/scenarios": 10,....
    let data: std::collections::HashMap<String, usize> =
        dao::catalog::get_endpoints_num_per_subdomain(&all_specs);

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
        let short_path =
            dao::catalog::get_spec_short_path(String::from(&SETTINGS.catalog_dir), &spec);
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

    let all_specs: Vec<SpecItem> = dao::catalog::list_specs(SETTINGS.catalog_path.as_str());

    let data: std::collections::HashMap<String, usize> =
        dao::catalog::get_endpoints_num_per_subdomain(&all_specs);

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
    let response = match dao::repo_apis::get_apis_per_domain_id(&SETTINGS.database, id) {
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
                dao::repo_domains::delete_domain(&SETTINGS.database, id).unwrap();
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
