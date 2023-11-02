use crate::app::dao::catalog::handlers::SpecHandler;

#[derive(Debug, Clone)]
pub struct v2 {
    spec: String,
}

impl v2 {
    pub fn new(val: &str) -> v2 {
        v2 {
            spec: String::from(val),
        }
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
        spec_as_yaml["info"]["title"].as_str().unwrap().to_string()
    }

    fn get_description(&self) -> String {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
        spec_as_yaml["info"]["description"].as_str().unwrap().to_string()
    }

    fn get_paths_len(&self) -> usize {
        let spec_as_yaml: serde_yaml::Value = serde_yaml::from_str(&self.spec).unwrap();
        let channels: &serde_yaml::Value = &spec_as_yaml["channels"];
        
        channels.as_mapping().unwrap().len()        
    }

    fn get_paths(&self) -> Vec<crate::app::dao::catalog::handlers::Path> {
        Vec::new()
    }
}

// impl Clone for v2 {
//     fn clone(&self) -> Self {
//         v2 {
//             spec: self.spec.clone(),
//         }
//     }
// }

// impl std::fmt::Debug for v2 {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "{:?}", 
//             self.spec
//         )
//     }
// }

#[cfg(test)]
pub mod tests {
    use crate::app::dao::catalog::handlers::SpecHandler;


    #[test]
    fn test_play_with_trait(){
    
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
        
        let spec = crate::app::dao::catalog::handlers::implem::asyncapi::v2::new(asyncapi_spec);
        
        assert_eq!(spec.get_version(), "1.2.0");
        assert_eq!(spec.get_description(), "This service is in charge of processing user signups");
        assert_eq!(spec.get_paths_len(), 1);
        assert_eq!(spec.get_title(), "Account Service");
        
    }

}