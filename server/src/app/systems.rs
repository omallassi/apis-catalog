use std::collections::HashSet;

use log::{debug, info};
use serde::{Deserialize, Serialize};

use actix_web::{web, get, Responder};
use actix_web::HttpResponse;

use crate::app::dao::repo_layers::*;
use crate::shared::settings::*;

use super::dao::catalog;

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
    pub domains: Vec<String>,
}

#[get("/v1/systems/{system}/layers/{layer}")]
pub async fn get_all_domains_per_system_and_layer(path: web::Path<(String, String)>) -> impl Responder{
    let (system, layer) = path.into_inner();

    let returned_domains = self::get_domains_per_system_and_layer(&SETTINGS.catalog_path, &system, &layer);

    info!("get_all_domains_per_system_and_layer() for system {:?} and layer {:?} - got [{:?}] domains", &system, &layer, &returned_domains.len());

    let response = DomainsPerSystemAndLayer {
        system : String::from(&system),
        layer :  String::from(&layer),
        domains: returned_domains.into_iter().collect()
    };

    HttpResponse::Ok().json(response)

}

fn get_domains_per_system_and_layer(path: &str, system: &String, layer: &String) -> HashSet<String>{
    let mut domains = HashSet::new();

    let all_specs = catalog::list_specs(path);
    //loop over the list and check system and layer equality
    for spec in all_specs{
        match spec.systems.contains(&system.to_lowercase()){
            true => {
                match spec.layer.eq(&layer.to_lowercase()) {
                    true => {
                        domains.insert(spec.domain);
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

    domains

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

        match (systems_layers.contains_key(&system)) {
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
        let path_as_str = path.into_os_string().into_string().unwrap();

        let sut = super::get_domains_per_system_and_layer(&path_as_str, &String::from("bpaas"), &String::from("application"));
        assert_eq!(sut.len(), 1);
        assert_eq!(sut.iter().next().unwrap(), "/v1/audit/trails");
        //same test as above but check case 
        let sut = super::get_domains_per_system_and_layer(&path_as_str, &String::from("BPaas"), &String::from("application"));
        assert_eq!(sut.len(), 1);
        assert_eq!(sut.iter().next().unwrap(), "/v1/audit/trails");

        let sut = super::get_domains_per_system_and_layer(&path_as_str, &String::from("bpaas"), &String::from("functional"));
        assert_eq!(sut.len(), 0);
    }
}