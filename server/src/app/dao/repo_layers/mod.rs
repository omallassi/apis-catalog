use log::{error, info};
use serde::{Deserialize, Serialize};



#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct SystemAndLayer {
    pub system: std::string::String,
    pub layer: std::string::String,
    pub description: std::string::String,
    #[serde(rename = "image")]
    pub config_image_link: std::string::String,
    #[serde(rename = "color")]
    pub config_color: std::string::String,
}

pub fn list_systems_and_layers(path: &str) -> Vec<SystemAndLayer> {
    let f = match std::fs::File::open(path){
        Ok(f) => {
            info!("has loaded system and layers catalog from Yaml file {:?}", &path);

            f
        }
        Err(err) => {
            error!("failed loading Yml Catalog from {:?} - {:?}", &path, err);
            panic!("failed loading Yml Catalog from {:?} - {:?}",&path, err);
        }
    };

    let yaml_struct = serde_yaml::from_reader(f);
    match yaml_struct {
        Ok(yaml) => yaml,
        Err(err) => {
            panic!("Unable to load Yaml Struct from file - {:?}", err);
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_list_systems_and_layers() {
        let mut path = std::path::PathBuf::new();
        path.push(env!("CARGO_MANIFEST_DIR"));
        path.push("./tests/data/systems-layers-catalog.yaml");

        println!("will use path {:?}", &path);

        let sut = super::list_systems_and_layers(path.into_os_string().into_string().unwrap().as_str());

        let expected_val : usize = 6;
        assert_eq!(&sut.len(), &expected_val);
        let item = &sut.get(0).unwrap();
        assert_ne!(item.system, "");
    
    }
}