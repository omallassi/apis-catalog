use log::warn;

use crate::app::dao::catalog::handlers::{SpecHandler, Method, Path};

#[derive(Debug, Clone)]
pub struct v2 {
    spec: String,
}

impl v2 {
    pub fn new(val: &str) -> Result<Self, String> {
        Ok( Self { spec: String::from(val) } )
    }
}
impl crate::app::dao::catalog::handlers::SpecHandler for v2{
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
        "To Be Implemented".to_string()
    }

    fn get_api_id(&self) -> String {
        todo!()
    }

    fn get_layer(&self) -> String {
        "To Be Implemented".to_string()
    }

    fn get_systems(&self) -> Vec<String> {
        vec![ "To Be Implemented".to_string() ]
    }

    fn get_domain(&self) -> String {
        "To Be Implemented".to_string()
    }
}

#[derive(Debug, Clone)]
pub struct v1 {
    spec: String,
}

impl v1 {
    pub fn new(val: &str) -> Result<Self, String> {
        Ok( Self { spec: String::from(val) } )
    }
}
impl crate::app::dao::catalog::handlers::SpecHandler for v1{
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
                                let empty_val = serde_yaml::Value::String("".to_string());
                                let method_name = key_1; 
                                let method_description = value_1.get("description").unwrap_or( &empty_val );
                                let method_summary = value_1.get("summary").unwrap_or( &empty_val );

                                methods.push(Method { method: method_name.as_str().unwrap().to_string(), description: method_description.as_str().unwrap().to_string(), summary: method_summary.as_str().unwrap().to_string() })
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
        "To Be Implemented".to_string()
    }

    fn get_api_id(&self) -> String {
        todo!()
    }

    fn get_layer(&self) -> String {
        "To Be Implemented".to_string()
    }

    fn get_systems(&self) -> Vec<String> {
        vec![ "To Be Implemented".to_string() ]
    }

    fn get_domain(&self) -> String {
        "To Be Implemented".to_string()
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

        let spec = crate::app::dao::catalog::handlers::implem::asyncapi::v1::new(content.as_str()).unwrap();
        
        assert_eq!(spec.get_version(), "1.12");
        assert_eq!(spec.get_description(), "");
        assert_eq!(spec.get_paths_len(), 4);
        assert_eq!(spec.get_title(), "Portfolio Management - Full Revaluation - Business action");

        let all_paths = spec.get_paths();

        assert_eq!("v1.portfolio-management.full-revaluation.business-action-request", all_paths.get(0).unwrap().path);
        assert_eq!("publish", all_paths.get(0).unwrap().methods.get(0).unwrap().method);
        assert_eq!("", all_paths.get(0).unwrap().methods.get(0).unwrap().description);
        assert_eq!("", all_paths.get(0).unwrap().methods.get(0).unwrap().summary);

        assert_eq!("v1.portfolio-management.full-revaluation.business-action-request-subscription", all_paths.get(1).unwrap().path);
        assert_eq!("subscribe", all_paths.get(1).unwrap().methods.get(0).unwrap().method);
        assert_eq!("", all_paths.get(1).unwrap().methods.get(0).unwrap().description);
        assert_eq!("", all_paths.get(1).unwrap().methods.get(0).unwrap().summary);

        
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
        
        let spec = crate::app::dao::catalog::handlers::implem::asyncapi::v2::new(asyncapi_spec).unwrap();
        
        assert_eq!(spec.get_version(), "1.2.0");
        assert_eq!(spec.get_description(), "This service is in charge of processing user signups");
        assert_eq!(spec.get_paths_len(), 1);
        assert_eq!(spec.get_title(), "Account Service");

        spec.get_paths();
        
    }

}