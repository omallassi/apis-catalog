use log::{debug, error, info};


#[path = "./dao/mod.rs"]
mod dao;
use dao::repo_layers::*;


pub fn get_all_layers_per_systems(path: &str) -> std::collections::HashMap<String, Vec<String>>{
    info!("get all layers per systems");

    let mut systems_layers: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    //get the flat list of systems / layers and create the map
    let flat_list_of_systems_layers = list_systems_and_layers(path);
    for item in flat_list_of_systems_layers.iter() {
        let system = String::from(&item.system);

        match (systems_layers.contains_key(&system)) {
            true => {
                print!("true");
            },
            false => {
                print!("false");
            }
        }

        let layers: Vec<String> = Vec::new();


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

        let expected_val : usize = 2;
        assert_eq!(sut.len(), expected_val);
    
    }
}