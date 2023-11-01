use openapiv3::OpenAPI;
use log::warn;
use super::{handlers::{Method, Path}, DEFAULT_SYSTEM_LAYER};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct SpecItem {
    // pub spec_type: SpecType,
    pub spec_handler: OpenAPI, //TODO should be private again
    path: std::string::String,
    catalog_id: String,
    catalog_dir: String,
}

impl SpecItem {

    pub fn new(path: std::string::String, catalog_id: String, catalog_dir: String, spec: OpenAPI) -> SpecItem {
        let spec = SpecItem{
            path: path.clone(), 
            catalog_id: catalog_id.clone(),
            catalog_dir: catalog_dir.clone(),
            spec_handler: spec,
        };

        spec
    }

    pub fn get_file_path(&self) -> &str {
        &self.path
    }

    pub fn get_catalog_id(&self) -> &str {
        &self.catalog_id
    }

    pub fn get_catalog_dir(&self) -> &str {
        &self.catalog_dir
    }

    pub fn get_version(&self) -> &str {
        &self.spec_handler.info.version
    }

    pub fn get_title(&self) -> &str {
        &self.spec_handler.info.title
    }

    pub fn get_description(&self) -> &str {
        let description = match &self.spec_handler.info.description {
            Some(d) => d,
            None => "",
        };

        &description
    }

    pub fn get_paths_len(&self) -> usize {
        * &self.spec_handler.paths.paths.len()
    }

    pub fn get_paths(&self) -> Vec<Path> {
        let mut all_paths = Vec::new();

        let paths = &self.spec_handler.paths;
        for (path_value, path_item) in paths.iter() {
            match path_item.as_item() {
                Some(item) => {
                    //need to get the http method fro the PathItem
                    let http_methods: [(&str, &Option<openapiv3::Operation>); 7] = [
                        ("GET", &item.get),
                        ("POST", &item.post),
                        ("PUT", &item.put),
                        ("DELETE", &item.delete),
                        ("OPTIONS", &item.options),
                        ("HEAD", &item.head),
                        ("PATCH", &item.patch),
                    ];


                    let mut all_methods = Vec::new();

                    for (method, operation_option) in &http_methods {
                        if let Some(ref ope) = operation_option {
                            let mut ope_summary = String::from("");
                            let mut ope_description = String::from("");
                            let mut ope_method = String::from("");

                            ope_summary.push_str( ope.summary.clone().unwrap_or("N/A".to_string()).as_str() );
                            ope_description.push_str( ope.description.clone().unwrap_or("N/A".to_string()).as_str()  );
                            ope_method.push_str( * method );

                            all_methods.push(Method{
                                method: String::from(* method),
                                description: ope_description, 
                                summary: ope_summary
                            });
                        }
                    }

                    all_paths.push(Path { path: String::from(path_value), methods: all_methods })
                }
                None => {
                    warn!("No path found for spec {:?}", &self.path);
                }
            }
        }

        all_paths
    }

    pub fn get_audience(&self) -> String {
        let audience:String  = match self.spec_handler.info.extensions.get("x-audience"){
            Some(aud) => String::from(aud.as_str().unwrap()),
            None => String::from(DEFAULT_SYSTEM_LAYER),
        };
    
        audience
    }

    pub fn get_api_id(&self) -> String {
        let api_id: String = match self.spec_handler.info.extensions.get("x-api-id"){ // as specified https://opensource.zalando.com/restful-api-guidelines/#215
            Some(id)=> String::from(id.as_str().unwrap()),
            None => String::from("0"),
        };
    
        api_id
    }
    
    pub fn get_layer(&self) -> String {
        let layer:String  = match self.spec_handler.extensions.get("x-layer"){
            Some(layer) => String::from(layer.as_str().unwrap()),
            None => String::from(DEFAULT_SYSTEM_LAYER),
        };
    
        layer.to_lowercase()
    }
    
    pub fn get_systems(&self) -> Vec<String> {
        
        let systems = match self.spec_handler.extensions.get("x-systems"){
            Some(systems) => {
                let mut returned_systems: Vec<String> = Vec::new();
                for system in systems.as_array().unwrap(){
                    //did not find a way to use into_iter().collect::Vec<String>>
                    returned_systems.push(String::from(system.as_str().unwrap()).to_lowercase());
                }
    
                returned_systems
            },
            None => {
                let mut systems: Vec<String> = Vec::new();
                systems.push(String::from(DEFAULT_SYSTEM_LAYER));        
    
                systems
            }
        };
    
        systems
    }

    pub fn get_domain(spec: &OpenAPI) -> &str {
        let base_url = match &spec.servers.is_empty() {
            true => "NA - servers attribute not specified",
            false => {
                //TODO can do better
                //base_url could have the following form http://baseurl/v1/xva-management/xva
                //will extract http://baseurl and keep the rest
                lazy_static! {
                    static ref RE: Regex = Regex::new(r"(http[s]?://[a-z]*)(.*)").unwrap();
                }
    
                if let Some(cap) = RE.captures(&spec.servers[0].url) {
                    cap.get(2).unwrap().as_str()
                } else {
                    &spec.servers[0].url
                }
            }
        };
    
        base_url
    }

    pub fn get_spec_short_path(spec: &SpecItem) -> &str {
        let catalog_dir_srt = &spec.catalog_dir;
        let short_path = SpecItem::extact_relative_path(&spec.path, &catalog_dir_srt);
    
        short_path
    }
    
    pub fn extact_relative_path<'a>(spec_path: &'a String, catalog_dir_srt: &'a String) -> &'a str {
        let catalog_dir = catalog_dir_srt.as_str().len();
        let len = spec_path.len();
    
        let short_path = &spec_path[ catalog_dir..len ];
        
        short_path
    }
    
}

#[cfg(test)]
pub mod tests {
    use crate::app::dao::catalog::spec::*;

    #[test]
    fn test_get_spec_short_path() {
        let openapi_spec = openapiv3::OpenAPI {
            openapi: "3.0.0".to_string(),
            info: openapiv3::Info {
                title: "My API".to_string(),
                version: "1.0.0".to_string(),
                ..Default::default()
            },
            paths: Default::default(),
            ..Default::default()
        };

        let spec = SpecItem {
            // spec_type: super::SpecType::OpenApi,
            path: String::from("/home/catalog/code/openapi-specifications/specifications/manual-tasks/openapi.yaml"), 
            spec_handler: openapi_spec, 
            catalog_id: String::from("not used here"),
            catalog_dir: String::from("/home/catalog/")
        };

        let sut = super::SpecItem::get_spec_short_path(&spec);
        assert_eq!("code/openapi-specifications/specifications/manual-tasks/openapi.yaml", sut);

    }
}
