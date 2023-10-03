use actix_web::web::Json;
use actix_web::{get, post, Responder};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use std::hash::{Hash, Hasher};

use crate::app::dao::repo_domains::*;
use crate::app::dao::catalog::*;
use crate::shared::settings::*;

use log::{debug, warn, error, info};

use uuid::Uuid;
use std::collections::HashSet;

use crate::app::dao::catalog;


/*
 * domain related APIs
 */

#[derive(Serialize, Deserialize, Debug)]
pub struct Domain {
    pub name: String,
    pub id: Uuid,
    pub description: String,
    pub owner: String,
    pub is_empty: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Domains {
    pub is_read_only: bool,
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
    pub spec_catalog_id: String,
    pub spec_path: String,
    pub resources: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DomainErrors {
    pub errors: Vec<DomainError>,
}

#[get("/v1/catalogs/all/domains")]
pub async fn get_all_domains_for_all_catalogs() -> impl Responder {
    info!("get all domains ");

    let mut domains = HashSet::new();

    let all_specs = catalog::list_specs(&SETTINGS.catalogs);
    //loop over the list and check system and layer equality
    for spec in all_specs{
        domains.insert(String::from(spec.domain));
    }

    info!("Domain # from all catalogs [{:?}]", &domains.len());

    HttpResponse::Ok().json(domains)
}

#[get("/v1/domains/errors")]
pub async fn get_domains_errors() -> impl Responder {
    info!("get domains errors");

    //get all specs
    let all_specs: Vec<SpecItem> = list_specs(&SETTINGS.catalogs);
    //at this stage data = {"N/A - servers not specified": 11, "/v1/settlement/operational-arrangement": 8, "/v1/market-risk/scenarios": 10,....

    let data: std::collections::HashMap<String, usize> = get_endpoints_num_per_subdomain(&all_specs);

    //get all declared (and official) domains
    let repo_domains_dao = crate::app::dao::repo_domains::DomainImplFactory::get_impl();
    let all_domains: Vec<String> = match repo_domains_dao.list_all_domains(&SETTINGS.database) {
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
        let short_path = get_spec_short_path(String::from(&spec.catalog_dir), &spec);
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
                spec_catalog_id: String::from(&spec.catalog_id),
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
pub async fn get_domains_stats() -> impl Responder {
    info!("get domains stats");

    let all_specs: Vec<SpecItem> = list_specs(&SETTINGS.catalogs);

    let data: std::collections::HashMap<String, usize> = get_endpoints_num_per_subdomain(&all_specs);

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
pub async fn get_domains() -> impl Responder {
    info!("get domains");

    let repo_domains_dao = crate::app::dao::repo_domains::DomainImplFactory::get_impl();
    let mut all_domains: Vec<DomainItem> =
        match repo_domains_dao.list_all_domains(&SETTINGS.database) {
            Ok(all_domains) => all_domains,
            Err(why) => {
                panic!("Unable to get domains: {}", why);
            }
        };


    let all_specs: Vec<SpecItem> = list_specs(&SETTINGS.catalogs);
    let non_emtpy_domains: std::collections::HashMap<String, usize> = get_endpoints_num_per_subdomain(&all_specs);

    //TODO : crappy!! find a way to better handle the /v1 - maybe relying on servers attr in OAI is not the right way and having dedicated OAI attributes would be easier / proper (alos taking into sonsiderations code generation plugins)
    let mut cleaned_non_empty_domain : std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (key, value) in &non_emtpy_domains {
        match &key.starts_with("/v"){
            true => {
                cleaned_non_empty_domain.insert(key[3..key.len()].to_string(), *value);
            },
            false => {
                warn!("found a domain that does not follow conventions - [{:?}]", &key);
            }
        }
    }
    //end TODO

    let mut domains = Vec::new();

    while let Some(domain) = all_domains.pop() {
        let is_empty : bool = ! cleaned_non_empty_domain.contains_key(&domain.name); 

        let domain = Domain {
            name: domain.name,
            id: domain.id,
            description: domain.description,
            owner: domain.owner,
            is_empty: is_empty, 
        };
        domains.push(domain);
    }

    let domains_obj = Domains { is_read_only: crate::app::dao::repo_domains::DomainImplFactory::is_read_only(), domains: domains };

    HttpResponse::Ok().json(domains_obj)
}

#[post("/v1/domains")]
pub async fn create_domain(domain: Json<Domain>) -> impl Responder {
    let repo_domains_dao = crate::app::dao::repo_domains::DomainImplFactory::get_impl();

    let uuid = repo_domains_dao
        .add_domain(
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

pub async fn delete_domain(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    info!("deleting domain for id [{:?}]", id);
    let id = Uuid::parse_str(&id).unwrap();

    //check if apis are related to this domain
    let response = match crate::app::dao::repo_apis::get_apis_per_domain_id(&SETTINGS.database, id) {
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
                let repo_domains_dao = crate::app::dao::repo_domains::DomainImplFactory::get_impl();
                repo_domains_dao
                    .delete_domain(&SETTINGS.database, id)
                    .unwrap();
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

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn test_domains_method() {
        
//         assert_eq!(2, 3);
//     }
// }