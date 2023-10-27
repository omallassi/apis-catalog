use std::collections::{HashSet, HashMap};

use log::{debug, info};
use serde::{Deserialize, Serialize};

use actix_web::{web, get, Responder};
use actix_web::HttpResponse;

use crate::app::dao::repo_layers::*;
use crate::app::dao::catalog::*;
use crate::shared::settings::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct System {
    pub name : String,
    pub layers : Vec<Layer>,
}

impl std::hash::Hash for System {
    fn hash<H>(&self, state: &mut H) where H: std::hash::Hasher,
    {
        self.name.hash(state);
    }
}

impl PartialEq for System {
    fn eq(&self, other: &System) -> bool {
        self.name == other.name
    }
}

impl Eq for System {}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub description: std::string::String,
    #[serde(rename = "image")]
    pub config_image_link: std::string::String,
    #[serde(rename = "color")]
    pub config_color: std::string::String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Systems {
    pub systems: Vec<System>,
}

#[get("/v1/systems")]
pub async fn get_all_systems() -> impl Responder {
    info!("get_all_systems()");
    
    let all = self::get_all_layers_per_systems(&SETTINGS.systems_and_layers.systems_catalog_path);
    let mut all_as_vec: Vec<System> = Vec::new();
    for (k, v) in all.iter() {
        all_as_vec.push( System{name: String::from(&k.name), layers: v.clone()} );
    }

    HttpResponse::Ok().json(Systems{systems: all_as_vec})
}


#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DomainsPerSystemAndLayer {
    pub system: String,
    pub layer: String, 
    pub domains: Vec<Domain>,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Domain {
    pub name: String, 
    pub specs: Vec<Spec>
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Spec {
    pub catalog_id: String,
    pub spec_path: String,
}

#[get("/v1/systems/{system}/layers/{layer}")]
pub async fn get_all_domains_per_system_and_layer(path: web::Path<(String, String)>) -> impl Responder{
    let (system, layer) = path.into_inner();
    

    let returned_domains = self::get_domains_per_system_and_layer(&SETTINGS.catalogs, &system, &layer);
    let returned_domain_length = &returned_domains.len();
    let mut returned_domains_as_vec: Vec<Domain> = returned_domains.into_iter().collect();
    returned_domains_as_vec.sort_by( |a, b| {
        a.name.cmp(&b.name)
    });

    info!("get_all_domains_per_system_and_layer() for system {:?} and layer {:?} - got [{:?}] domains", &system, &layer, returned_domain_length);

    let response = DomainsPerSystemAndLayer {
        system : String::from(&system),
        layer :  String::from(&layer),
        domains: returned_domains_as_vec
    };

    HttpResponse::Ok().json(response)
}

fn get_domains_per_system_and_layer(catalogs: &Vec<Catalog>, system: &String, layer: &String) -> HashSet<Domain>{
    let mut domains: HashSet<Domain> = HashSet::new();

    let mut specs_per_domain: HashMap<String, HashSet<(String, String)>> = HashMap::new();

    let all_specs = list_specs(&catalogs);
    //loop over the list and check system and layer equality
    for spec in all_specs{
        match spec.systems.contains(&system.to_lowercase()){
            true => {
                match spec.layer.eq(&layer.to_lowercase()) {
                    true => {
                        debug!("spec [{:?}] matches system [{:?}] *and* layer [{:?}]", &spec.path, &system, &layer);

                        let spec_domain = String::from(spec.domain);
                        match specs_per_domain.get(&spec_domain) {
                            Some(related_specs) => {
                                let mut specs = related_specs.clone();
                                specs.insert( (shorten_spec_path(spec.path, catalogs, String::from(&spec.catalog_id)), String::from(&spec.catalog_id)) );
                                specs_per_domain.insert(String::from(&spec_domain), specs);
                            },
                            None => {
                                let mut related_specs = HashSet::new();
                                related_specs.insert( (shorten_spec_path(spec.path, catalogs, String::from(&spec.catalog_id)), String::from(&spec.catalog_id)) );
                                specs_per_domain.insert(String::from(&spec_domain), related_specs);
                            }
                        }                
                    }, 
                    false => {
                        debug!("spec [{:?}] matches system [{:?}] but not layer [{:?}]", spec.path, system, layer);
                    }
                }
            
            }, 
            false => {
                debug!("spec [{:?}] does not belong to system [{:?}] and layer [{:?}]", spec.path, system, layer);
            }
        }
    }

    info!("Domain # from all catalogs, system [{:?}] and layer [{:?}] - [{:?}]", &system, &layer, &domains.len());

    for (domain_name, specs) in specs_per_domain {
            //create the vec of spec
        let mut specs_object: Vec<Spec> = Vec::new();
        for item in specs {
            specs_object.push(Spec{ spec_path: item.0, catalog_id: item.1 })
        }
        
        domains.insert( Domain {
            name: String::from(domain_name),
            specs: specs_object
        }
    );
    }

    domains

}


fn shorten_spec_path (spec_path: String, catalogs: &Vec<Catalog>, catalog_id: String) -> String {
    let returned_catalog = get_catalog_by_id(&catalogs, &catalog_id);
    let mut short_spec_path = String::from(&spec_path);
    if let Some(catalog) = returned_catalog{
        let tmp = extact_relative_path(&spec_path, &catalog.catalog_dir);
        short_spec_path = String::from(tmp);
    }

    short_spec_path
}

fn get_all_layers_per_systems(path: &str) -> std::collections::HashMap<System, Vec<Layer>>{
    info!("get all layers per systems");

    let mut systems_layers: std::collections::HashMap<System, Vec<Layer>> = std::collections::HashMap::new();
    //get the flat list of systems / layers and create the map
    let flat_list_of_systems_layers = list_systems_and_layers(path);
    //
    for item in flat_list_of_systems_layers.iter() {
        let system = System {
            name: String::from(&item.system),
            layers: Vec::new(),
        };

        match systems_layers.contains_key(&system) {
            true => {
                let layers = systems_layers.get_mut(&system).unwrap();
                layers.push(Layer { 
                    name: String::from(&item.layer), 
                    description: String::from(&item.description),
                    config_color: String::from(&item.config_color),
                    config_image_link: String::from(&item.config_image_link)
                });


            },
            false => {
                //add the system and the layer
                let mut layers: Vec<Layer> = Vec::new();
                layers.push(Layer { 
                    name: String::from(&item.layer), 
                    description: String::from(&item.description),
                    config_color: String::from(&item.config_color),
                    config_image_link: String::from(&item.config_image_link)
                });

                systems_layers.insert(system, layers);
            }
        }
    }

    systems_layers
}


#[cfg(test)]
mod tests {
    use crate::shared::settings::*;

    #[test]
    fn test_get_all_layres_per_systems() {
        let mut path = std::path::PathBuf::new();
        path.push(env!("CARGO_MANIFEST_DIR"));
        path.push("./tests/data/systems-layers-catalog.yaml");

        println!("will use path {:?}", &path);

        let sut = super::get_all_layers_per_systems(path.into_os_string().into_string().unwrap().as_str());

        let expected_val : usize = 3;
        assert_eq!(sut.len(), expected_val);
        //
        let layers = sut.get(&super::System {name: String::from("A"), layers: Vec::new()});
        assert_eq!(layers.is_none(), false);
        let layers = sut.get(&super::System {name: String::from("A"), layers: Vec::new()}).unwrap();
        assert_eq!(layers.len(), 3);

        //
        let layers = sut.get(&super::System {name: String::from("B"), layers: Vec::new()});
        assert_eq!(layers.is_none(), false);
        let layers = sut.get(&super::System {name: String::from("B"), layers: Vec::new()}).unwrap();
        assert_eq!(layers.len(), 2);
    }

    #[test]
    fn test_get_domain_per_system_and_layer(){
        let mut path = std::path::PathBuf::new();
        path.push(env!("CARGO_MANIFEST_DIR"));
        path.push("./tests/data/catalog/");

        let catalog = Catalog{
            catalog_id: String::from("uuid"),
            catalog_name: String::from("name"), 
            catalog_dir: String::from("not used here"),
            catalog_include_glob_pattern: vec![String::from("**/*.yaml")],
            catalog_scm_clone_cmd: String::from("not used here"), 
            catalog_scm_pull_cmd: String::from("not used here"),
            catalog_path: path.into_os_string().into_string().unwrap(),
            catalog_scm_clone: false,
            catalog_http_base_uri: String::from("not used here")
        };
        let mut catalogs = Vec::new();
        catalogs.push(catalog);

        let sut = super::get_domains_per_system_and_layer( &catalogs, &String::from("bpaas"), &String::from("application"));
        assert_eq!(sut.len(), 1);
        assert_eq!(sut.iter().next().unwrap().name, "/v1/audit/trails");
        //same test as above but check case 
        let sut = super::get_domains_per_system_and_layer(&catalogs, &String::from("BPaas"), &String::from("application"));
        assert_eq!(sut.len(), 1);
        assert_eq!(sut.iter().next().unwrap().name, "/v1/audit/trails");

        let sut = super::get_domains_per_system_and_layer(&catalogs, &String::from("bpaas"), &String::from("functional"));
        assert_eq!(sut.len(), 0);
    }
}