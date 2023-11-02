use log::warn;
use regex::Regex;
use super::handlers::{Path, SpecHandler, SpecType};

#[derive(Debug, Clone)]
pub struct SpecItem {
    pub spec_type: SpecType,
    handler: Box<dyn SpecHandler>,
    path: std::string::String,
    catalog_id: String,
    catalog_dir: String,
}

impl SpecItem {

    pub fn get_file_path(&self) -> &str {
        &self.path
    }

    pub fn get_catalog_id(&self) -> &str {
        &self.catalog_id
    }

    pub fn get_catalog_dir(&self) -> &str {
        &self.catalog_dir
    }

    pub fn get_version(&self) -> String {
        self.handler.get_version()
    }

    pub fn get_title(&self) -> String {
        self.handler.get_title()
    }

    pub fn get_description(&self) -> String {
        self.handler.get_description()
    }

    pub fn get_paths_len(&self) -> usize {
        self.handler.get_paths_len()
    }

    pub fn get_paths(&self) -> Vec<Path> {
        let all_paths = self.handler.get_paths();
        if all_paths.len() == 0 {
            warn!("No path found for spec {:?}", &self.path);
        }

        all_paths
    }

    pub fn get_audience(&self) -> String {
        self.handler.get_audience()
    }

    pub fn get_api_id(&self) -> String {
        self.handler.get_api_id()
    }
    
    pub fn get_layer(&self) -> String {
        self.handler.get_layer()
    }
    
    pub fn get_systems(&self) -> Vec<String> {
        self.handler.get_systems()
    }

    pub fn get_domain(&self) -> String {
        self.handler.get_domain()
    }

    pub fn get_spec_short_path(&self) -> &str {
        let catalog_dir_srt = &self.catalog_dir;
        let path_str = &self.path;
        let short_path = self::extact_relative_path(path_str, catalog_dir_srt);
    
        short_path
    }

    pub fn get_spec_type(&self) -> SpecType {
        self.spec_type
    }
    
}

pub fn from_str(path: std::string::String, catalog_id: String, catalog_dir: String, spec: &str) -> Result<SpecItem, String> {

    let patterns = [
        r"openapi:\W*3", 
        r"asyncapi:\W*1",
        r"asyncapi:\W*2",
        r"syntax=.proto3",
    ];

    let patterns_as_regexp: Vec<Regex> = patterns.iter().map(|pattern| Regex::new(pattern).expect("Invalid regex pattern")).collect();

    let mut returned_val = Err( format!("Unable to parse content for path {:?}", path) );
    for (index, regex) in patterns_as_regexp.iter().enumerate() {
        if regex.is_match(spec) {
            returned_val = match index {
                0 => {
                    let val = match crate::app::dao::catalog::handlers::implem::opanapi::v3::new(&spec) {
                        Ok(v3) => {
                            let spec = SpecItem{
                                spec_type: SpecType::OpenAPIv3,
                                path: path.clone(), 
                                catalog_id: catalog_id.clone(),
                                catalog_dir: catalog_dir.clone(),
                                handler: Box::new( v3 ),
                            };

                            Ok(spec)
                        },
                        Err(why)=> {
                            warn!("Unable to parse file [{:?}] - reason [{:?}]", &path, &why);
                            let error_message = format!("Unable to parse file [{:?}] - reason [{:?}]", path, &why);
                        
                            Err( error_message )
                        }
                    };

                    val
                }
                1 => {
                    let val = match crate::app::dao::catalog::handlers::implem::asyncapi::v1::new(&spec){
                        Ok(v1) => {
                            let spec = SpecItem{
                                spec_type: SpecType::AsyncAPIv1,
                                path: path.clone(), 
                                catalog_id: catalog_id.clone(),
                                catalog_dir: catalog_dir.clone(),
                                handler: Box::new( v1 ),
                            };

                            Ok(spec)
                        }
                        Err(why) => {
                            warn!("Unable to parse file [{:?}] - reason [{:?}]", &path, &why);
                            let error_message = format!("Unable to parse file [{:?}] - reason [{:?}]", path, &why);
                        
                            Err( error_message )
                        }
                    };

                    val
                }
                2 => {
                    let val = match crate::app::dao::catalog::handlers::implem::asyncapi::v2::new(&spec){
                        Ok(v2) => {
                            let spec = SpecItem{
                                spec_type: SpecType::AsyncAPIv2,
                                path: path.clone(), 
                                catalog_id: catalog_id.clone(),
                                catalog_dir: catalog_dir.clone(),
                                handler: Box::new( v2 ),
                            };

                            Ok(spec)
                        }
                        Err(why) => {
                            warn!("Unable to parse file [{:?}] - reason [{:?}]", &path, &why);
                            let error_message = format!("Unable to parse file [{:?}] - reason [{:?}]", path, &why);
                        
                            Err( error_message )
                        }
                    };

                    val
                }
                // 3 => {
                //     println!("index {:?}", index);
                // }
                _ => {
                    warn!("Content for spec  [{:?}] does not match any of the support spec format", &path);
                    let error_message = format!("Content for spec  [{:?}] does not match any of the support spec format", &path);
        
                    Err( error_message )
                }
            };
            break;
        }
    }

    returned_val

}


pub fn extact_relative_path<'a>(spec_path: &'a String, catalog_dir_srt: &'a String) -> &'a str {
    let catalog_dir = catalog_dir_srt.as_str().len();
    let len = spec_path.len();

    let short_path = &spec_path[ catalog_dir..len ];
    
    short_path
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

        let spec_as_str = serde_yaml::to_string(&openapi_spec).unwrap();

        let spec = SpecItem {
            spec_type: SpecType::OpenAPIv3,
            path: String::from("/home/catalog/code/openapi-specifications/specifications/manual-tasks/openapi.yaml"), 
            catalog_id: String::from("not used here"),
            catalog_dir: String::from("/home/catalog/"),
            handler: Box::new(crate::app::dao::catalog::handlers::implem::opanapi::v3::new(&spec_as_str).unwrap()),
        };

        let sut = super::SpecItem::get_spec_short_path(&spec);
        assert_eq!("code/openapi-specifications/specifications/manual-tasks/openapi.yaml", sut);

    }

    #[test]
    fn test_spec_item_from_str_for_openapi(){
        let mut path_item = openapiv3::PathItem::default();
        let mut get_operation = openapiv3::Operation::default();
        get_operation.summary = Some("Get example".to_string());
        path_item.get = Some(get_operation);
        let mut post_operation = openapiv3::Operation::default();
        post_operation.summary = Some("Post Example".to_string());
        path_item.post = Some(post_operation);
        
        let mut openapi_spec = openapiv3::OpenAPI {
            openapi: "3.0.0".to_string(),
            info: openapiv3::Info {
                title: "My API".to_string(),
                version: "1.4.0".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        let mut paths = indexmap::IndexMap::new();//openapiv3::Paths::default();
        paths.insert("/example".to_string(), openapiv3::ReferenceOr::Item((path_item)));
        openapi_spec.paths.paths = paths;

        let spec_as_str = serde_yaml::to_string(&openapi_spec).unwrap();

        let catalog_id ="rr".to_string();
        let catalog_dir = "fff".to_string(); 
        let path = "a path".to_string();

        ///
        let spec = crate::app::dao::catalog::spec::from_str(path, catalog_id, catalog_dir, spec_as_str.as_str()).unwrap();

        assert_eq!(spec.get_spec_type(), SpecType::OpenAPIv3);
        assert_eq!(spec.get_version(), "1.4.0");
        assert_eq!(spec.get_title(), "My API");
        assert_eq!(spec.get_description(), "");
        assert_eq!(spec.get_paths_len(), 1);

        assert_eq!(spec.get_paths()[0].methods[0].method, "GET");
        assert_eq!(spec.get_paths()[0].methods[1].method, "POST");

    }

    #[test]
    fn test_spec_item_from_str_for_asyncapi_v1(){
        let mut path = std::path::PathBuf::new();
        path.push(env!("CARGO_MANIFEST_DIR"));
        path.push("./tests/data/catalog/async/messaging-1.0.0.yml");

        let content = std::fs::read_to_string(path.as_path()).unwrap();

        let spec = crate::app::dao::catalog::spec::from_str("path".to_string(), "catalog_id".to_string(), "catalog_dir".to_string(), content.as_str()).unwrap();

        assert_eq!(spec.get_spec_type(), SpecType::AsyncAPIv1);
        assert_eq!(spec.get_version(), "1.12");
        assert_eq!(spec.get_title(), "Portfolio Management - Full Revaluation - Business action");
        assert_eq!(spec.get_description(), "");
        assert_eq!(spec.get_paths_len(), 4);
    }

    #[test]
    fn test_spec_item_from_str_for_asyncapi_v2(){
        let mut path = std::path::PathBuf::new();
        path.push(env!("CARGO_MANIFEST_DIR"));
        path.push("./tests/data/catalog/async/messaging-2.6.0.yml");

        let content = std::fs::read_to_string(path.as_path()).unwrap();

        let spec = crate::app::dao::catalog::spec::from_str("path".to_string(), "catalog_id".to_string(), "catalog_dir".to_string(), content.as_str()).unwrap();

        assert_eq!(spec.get_spec_type(), SpecType::AsyncAPIv2);
        assert_eq!(spec.get_version(), "1.12.0");
        assert_eq!(spec.get_title(), "Account Service");
        assert_eq!(spec.get_description(), "This service is in charge of processing user signups");
        assert_eq!(spec.get_paths_len(), 1);
    }

}
