use log::warn;

use crate::app::dao::catalog::handlers::{SpecHandler, Method, Path};

#[derive(Debug, Clone)]
pub struct V2 {
    spec: String,
}

impl V2 {
    pub fn new(val: &str) -> Result<Self, String> {
        Ok( Self { spec: String::from(val) } )
    }
}
impl crate::app::dao::catalog::handlers::SpecHandler for V2{
    fn get_version(&self) -> String{
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
    
        // Access the value and handle the potential absence or incorrect type
        if let Some(version) = spec_as_yaml["info"]["version"].as_str() {
            version.to_string()
        } else {
            "N/A".to_string()
        }
    }

    fn get_title(&self) -> String {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
        let mut title = "";
        if let Some(info) = spec_as_yaml.get("info") {
            if let Some(val) = info.get("title"){
                title = val.as_str().unwrap();
            }
        }

        title.to_string()
    }

    fn get_description(&self) -> String {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
        let mut description = "";
        if let Some(info) = spec_as_yaml.get("info") {
            if let Some(desc) = info.get("description"){
                description = desc.as_str().unwrap();
            }
        }

        description.to_string()
    }

    fn get_paths_len(&self) -> usize {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
        let channels: &serde_yaml::Value = &spec_as_yaml["channels"];
        let len = match channels.as_mapping(){
            Some(val) => val.len(),
            None => 0
        };

        len  
    }

    fn get_paths(&self) -> Vec<crate::app::dao::catalog::handlers::Path> {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
        let channels: &serde_yaml::Value = &spec_as_yaml["channels"];

        match channels.as_mapping(){
            Some(val) => {
                for (key, value) in val{
                    println!("got {:?} - {:?}", key, value);
                    match value.as_mapping(){
                        Some(ope) => {
                            for (key, value) in ope {
                                println!("got after {:?} - {:?}", key, value);
                            }
                        },
                        None => {
                            println!("none")
                        }
                    };
                }

            }
            None => {}
        };




        Vec::new()
    }

    fn get_audience(&self) -> String {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
        let mut audience = crate::app::dao::catalog::DEFAULT_SYSTEM_LAYER;
        if let Some(info) = spec_as_yaml.get("info") {
            if let Some(val) = info.get("x-audience"){
                audience = val.as_str().unwrap();
            }
        }

        audience.to_string()
    }

    fn get_api_id(&self) -> String {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
        let mut api_id = "0".to_string();
        if let Some(info) = spec_as_yaml.get("info") {
            if let Some(val) = info.get("x-api-id"){
                if val.is_string() {
                    api_id = String::from(val.as_str().unwrap());
                }
                if val.is_number() {
                    api_id = val.as_u64().unwrap().to_string();
                }
            }
        }

        api_id
    }
    fn get_layer(&self) -> String {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
        let mut layer = String::from( crate::app::dao::catalog::DEFAULT_SYSTEM_LAYER );
        if let Some(info) = spec_as_yaml.get("x-layer") {
            layer = String::from(info.as_str().unwrap());
        }

        layer
    }

    fn get_systems(&self) -> Vec<String> {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
        let systems = match spec_as_yaml.get("x-systems"){
            Some(systems) => {
                let mut returned_systems: Vec<String> = Vec::new();
                let list_of_systems = systems.as_sequence().unwrap();
                for system in list_of_systems{
                    //did not find a way to use into_iter().collect::Vec<String>>
                    returned_systems.push( String::from(system.as_str().unwrap()) );
                }
    
                returned_systems
            },
            None => {
                let mut systems: Vec<String> = Vec::new();
                systems.push(String::from(crate::app::dao::catalog::DEFAULT_SYSTEM_LAYER));        
    
                systems
            }
          };
    
          systems
    }

    fn get_domain(&self) -> String {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();


        "To Be Implemented".to_string()
    }
}

#[derive(Debug, Clone)]
pub struct V1 {
    spec: String,
}

impl V1 {
    pub fn new(val: &str) -> Result<Self, String> {
        Ok( Self { spec: String::from(val) } )
    }
}
impl crate::app::dao::catalog::handlers::SpecHandler for V1{
    fn get_version(&self) -> String{
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
    
        // Access the value and handle the potential absence or incorrect type
        if let Some(version) = spec_as_yaml["info"]["version"].as_str() {
            version.to_string()
        } else {
            "N/A".to_string()
        }
    }

    fn get_title(&self) -> String {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
        let mut title = "";
        if let Some(info) = spec_as_yaml.get("info") {
            if let Some(val) = info.get("title"){
                title = val.as_str().unwrap();
            }
        }

        title.to_string()
    }

    fn get_description(&self) -> String {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
        let mut description = "";
        if let Some(info) = spec_as_yaml.get("info") {
            if let Some(desc) = info.get("description"){
                description = desc.as_str().unwrap();
            }
        }

        description.to_string()
    }

    fn get_paths_len(&self) -> usize {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
        let topics: &serde_yaml::Value = &spec_as_yaml["topics"];
        let len = match topics.as_mapping(){
            Some(val) => val.len(),
            None => 0
        };

        len  
    }

    fn get_paths(&self) -> Vec<crate::app::dao::catalog::handlers::Path> {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
        let topics: &serde_yaml::Value = &spec_as_yaml["topics"];

        let mut all_paths: Vec<Path> = Vec::new();

        match topics.as_mapping(){
            Some(val) => {
                for (key, value) in val{
                    let mut methods: Vec<Method> = Vec::new();
                    match value.as_mapping(){
                        Some(ope) => {
                            for (key_1, value_1) in ope {
                                //to avoid having extension
                                let async_methods = vec!["publish", "subscribe"];
                                if async_methods.contains(&key_1.as_str().unwrap()) {
                                    let empty_val = serde_yaml::Value::String("".to_string());
                                    let method_name = key_1; 
                                    let method_description = value_1.get("description").unwrap_or( &empty_val );
                                    let method_summary = value_1.get("summary").unwrap_or( &empty_val );

                                    methods.push(Method { method: method_name.as_str().unwrap().to_string(), description: method_description.as_str().unwrap().to_string(), summary: method_summary.as_str().unwrap().to_string() })
                                }
                            }
                        },
                        None => {
                            warn!("No operation found on path {:?} for spec title {:?}", key, self.get_title());
                        }
                    };

                    all_paths.push(Path { path: key.as_str().unwrap().to_string(), methods: methods })
                }   

            }
            None => {}
        };

        all_paths
    }

    fn get_audience(&self) -> String {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
        let mut audience = crate::app::dao::catalog::DEFAULT_SYSTEM_LAYER;
        if let Some(info) = spec_as_yaml.get("info") {
            if let Some(val) = info.get("x-audience"){
                audience = val.as_str().unwrap();
            }
        }

        audience.to_string()
    }

    fn get_api_id(&self) -> String {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
        let mut api_id = "0".to_string();
        if let Some(info) = spec_as_yaml.get("info") {
            if let Some(val) = info.get("x-api-id"){
                if val.is_string() {
                    api_id = String::from(val.as_str().unwrap());
                }
                if val.is_number() {
                    api_id = val.as_u64().unwrap().to_string();
                }
            }
        }

        api_id
    }

    fn get_layer(&self) -> String {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
        let mut layer = String::from( crate::app::dao::catalog::DEFAULT_SYSTEM_LAYER );
        if let Some(info) = spec_as_yaml.get("x-layer") {
            layer = String::from(info.as_str().unwrap());
        }

        layer
    }

    fn get_systems(&self) -> Vec<String> {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
        let systems = match spec_as_yaml.get("x-systems"){
            Some(systems) => {
                let mut returned_systems: Vec<String> = Vec::new();
                let list_of_systems = systems.as_sequence().unwrap();
                for system in list_of_systems{
                    //did not find a way to use into_iter().collect::Vec<String>>
                    returned_systems.push( String::from(system.as_str().unwrap()) );
                }
    
                returned_systems
            },
            None => {
                let mut systems: Vec<String> = Vec::new();
                systems.push(String::from(crate::app::dao::catalog::DEFAULT_SYSTEM_LAYER));        
    
                systems
            }
          };
    
          systems
    }

    /// As for OpenAPI, in Async.V1, (the `servers` are specified)[https://github.com/asyncapi/spec/tree/1.0.0#A2SServers]. 
    /// We use *for now¨ the first item to define the domain.
    /// likely to evolve. 
    fn get_domain(&self) -> String {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
        let mut domaain = String::from( "NA - servers attribute not specified" );
        if let Some(servers) = spec_as_yaml.get("servers") {
            let urls = servers.as_sequence().unwrap();
            if let Some(val) = urls.get(0) {
                let url = val.as_mapping().unwrap();
                //only get the first one 
                if let Some((_first_key, first_value)) = url.iter().next() {
                    domaain = String::from( first_value.as_str().unwrap() );
                }
            }
        }

        domaain
    }
}

#[cfg(test)]
pub mod tests {
    use crate::app::dao::catalog::handlers::SpecHandler;


    #[test]
    fn test_async_v1(){
        let mut path = std::path::PathBuf::new();
        path.push(env!("CARGO_MANIFEST_DIR"));
        path.push("./tests/data/catalog/async/messaging-1.0.0.yml");

        let content = std::fs::read_to_string(path.as_path()).unwrap();

        let spec = crate::app::dao::catalog::handlers::implem::asyncapi::V1::new(content.as_str()).unwrap();
        
        assert_eq!(spec.get_version(), "1.12");
        assert_eq!(spec.get_description(), "");
        assert_eq!(spec.get_paths_len(), 4);
        assert_eq!(spec.get_title(), "Portfolio Management - Full Revaluation - Business action");
        assert_eq!(spec.get_audience(), "corporate");
        assert_eq!(spec.get_api_id(), "9e9880d5-ecb3-49eb-939d-c450aafe1a8d");

        let all_paths = spec.get_paths();

        assert_eq!(all_paths.len(), 4);

        assert_eq!("v1.portfolio-management.full-revaluation.business-action-request", all_paths.get(0).unwrap().path);
        assert_eq!("publish", all_paths.get(0).unwrap().methods.get(0).unwrap().method);
        assert_eq!("", all_paths.get(0).unwrap().methods.get(0).unwrap().description);
        assert_eq!("", all_paths.get(0).unwrap().methods.get(0).unwrap().summary);

        assert_eq!("v1.portfolio-management.full-revaluation.business-action-request-subscription", all_paths.get(1).unwrap().path);
        assert_eq!("subscribe", all_paths.get(1).unwrap().methods.get(0).unwrap().method);
        assert_eq!("", all_paths.get(1).unwrap().methods.get(0).unwrap().description);
        assert_eq!("", all_paths.get(1).unwrap().methods.get(0).unwrap().summary);

        assert_eq!("v1.portfolio-management.full-revaluation.business-action-response-subscription", all_paths.get(3).unwrap().path);
        assert_eq!(2, all_paths.get(3).unwrap().methods.len());

        
    }

    #[test] 
    fn test_async_v1_default_layer_and_system() {
        let spec = r#"
            asyncapi: "1.2.0"
            info:
                title: "Portfolio Management - Full Revaluation - Business action"
                version: "1.12"
                x-audience: corporate
                x-api-id: 9e9880d5-ecb3-49eb-939d-c450aafe1a8d
            topics:
                v1.portfolio-management.full-revaluation.business-action-request:
                    publish:
        "#;

        let spec = crate::app::dao::catalog::handlers::implem::asyncapi::V1::new(spec).unwrap();
        assert_eq!(spec.get_systems().len(), 1);
        assert_eq!(spec.get_systems().get(0).unwrap(), crate::app::dao::catalog::DEFAULT_SYSTEM_LAYER);
        assert_eq!(spec.get_layer(), crate::app::dao::catalog::DEFAULT_SYSTEM_LAYER);

        assert_eq!(spec.get_domain(), "NA - servers attribute not specified");
    }

    #[test] 
    fn test_async_v1_specified_layer_and_system() {
        let spec = r#"
            asyncapi: "1.2.0"
            info:
                title: "Portfolio Management - Full Revaluation - Business action"
                version: "1.12"
                x-audience: corporate
                x-api-id: 9e9880d5-ecb3-49eb-939d-c450aafe1a8d
            x-systems:
                - system1
            topics:
                v1.portfolio-management.full-revaluation.business-action-request:
                    publish:
        "#;

        let spec = crate::app::dao::catalog::handlers::implem::asyncapi::V1::new(spec).unwrap();
        assert_eq!(spec.get_systems().len(), 1);
        assert_eq!(spec.get_systems().get(0).unwrap(), "system1");
        assert_eq!(spec.get_layer(), crate::app::dao::catalog::DEFAULT_SYSTEM_LAYER);

        let spec = r#"
            asyncapi: "1.2.0"
            info:
                title: "Portfolio Management - Full Revaluation - Business action"
                version: "1.12"
                x-audience: corporate
                x-api-id: 9e9880d5-ecb3-49eb-939d-c450aafe1a8d
            x-systems:
                - system1
                - system2
            servers:
                - url: /v1/my-domain
            x-layer: layer1
            topics:
                v1.portfolio-management.full-revaluation.business-action-request:
                    publish:
        "#;

        let spec = crate::app::dao::catalog::handlers::implem::asyncapi::V1::new(spec).unwrap();
        assert_eq!(spec.get_systems().len(), 2);
        assert_eq!(spec.get_systems().get(0).unwrap(), "system1");
        assert_eq!(spec.get_systems().get(1).unwrap(), "system2");
        assert_eq!(spec.get_layer(), "layer1");

        assert_eq!(spec.get_domain(), "/v1/my-domain");
       
    }

    #[test]
    fn test_async_v2(){
    
        let asyncapi_spec = "
        asyncapi: '2.6.0'
        info:
          title: Account Service
          version: 1.2.0
          description: This service is in charge of processing user signups
        channels:
          user/signedup:
            subscribe:
              summary: ff
              description: ggggg
              message:
                $ref: '#/components/messages/UserSignedUp'
            publish: 
              summary: ff
              description: ggggg
              message:
                $ref: '#/components/messages/UserSignedUp'
        components:
          messages:
            UserSignedUp:
              payload:
                type: object
                properties:
                  displayName:
                    type: string
                    description: Name of the user
                  email:
                    type: string
                    format: email
                    description: Email of the user
        ";
        
        let spec = crate::app::dao::catalog::handlers::implem::asyncapi::V2::new(asyncapi_spec).unwrap();
        
        assert_eq!(spec.get_version(), "1.2.0");
        assert_eq!(spec.get_description(), "This service is in charge of processing user signups");
        assert_eq!(spec.get_paths_len(), 1);
        assert_eq!(spec.get_title(), "Account Service");
        assert_eq!(spec.get_audience(), crate::app::dao::catalog::DEFAULT_SYSTEM_LAYER);
        assert_eq!(spec.get_api_id(), "0");


        spec.get_paths();
        
    }

}