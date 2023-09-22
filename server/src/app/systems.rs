use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use serde::ser::Serializer;
use serde::de::{Deserializer, Error as _};

use actix_web::{get, post, Responder};
use actix_web::{web, HttpResponse};

use crate::app::dao::repo_layers::*;
use crate::shared::settings::*;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct System {
    pub name : String,
}

impl Serialize for System {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{}", self.name))
    }
}

impl<'de> Deserialize<'de> for System {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let data = <&str>::deserialize(deserializer)?;
        let name: String = String::from(data);

        Ok(Self { name })
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub config_color: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Systems {
    pub systems: std::collections::HashMap<System, Vec<Layer>>,
}

#[get("/v1/systems")]
pub async fn get_all_systems() -> impl Responder {
    debug!("get_all_systems()");
    
    let all = self::get_all_layers_per_systems(&SETTINGS.systems_and_layers.systems_catalog_path);

    HttpResponse::Ok().json(Systems{systems: all})
}

fn get_all_layers_per_systems(path: &str) -> std::collections::HashMap<System, Vec<Layer>>{
    info!("get all layers per systems");

    let mut systems_layers: std::collections::HashMap<System, Vec<Layer>> = std::collections::HashMap::new();
    //get the flat list of systems / layers and create the map
    let flat_list_of_systems_layers = list_systems_and_layers(path);
    //
    for item in flat_list_of_systems_layers.iter() {
        let system = System {
            name: String::from(&item.system)
        };

        match (systems_layers.contains_key(&system)) {
            true => {
                let mut layers = systems_layers.get_mut(&system).unwrap();
                layers.push(Layer { name: String::from(&item.layer), config_color: String::from(&item.config_color) });


            },
            false => {
                //add the system and the layer
                let mut layers: Vec<Layer> = Vec::new();
                layers.push(Layer { name: String::from(&item.layer), config_color: String::from(&item.config_color) });

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
        let layers = sut.get(&super::System {name: String::from("A")});
        assert_eq!(layers.is_none(), false);
        let layers = sut.get(&super::System {name: String::from("A")}).unwrap();
        assert_eq!(layers.len(), 3);

        //
        let layers = sut.get(&super::System {name: String::from("B")});
        assert_eq!(layers.is_none(), false);
        let layers = sut.get(&super::System {name: String::from("B")}).unwrap();
        assert_eq!(layers.len(), 2);
    }
}