use openapiv3::OpenAPI;
use regex::Regex;

use crate::app::dao::catalog::handlers::{SpecHandler, Path, Method};
use log::{debug, info, warn, error};

#[derive(Debug, Clone)]
pub struct V3 {
    pub spec: OpenAPI,
}
impl V3 {
  pub fn new(val: &str) -> Result<Self, String> {
    match serde_yaml::from_str::<OpenAPI>(val) {
      Ok(openapi) => {
        Ok(Self { spec: openapi })
      }
      Err(why) => {
        Err( format!("Unable to parse content") )
      }
    }
  }
}

impl SpecHandler for V3{
    fn get_version(&self) -> String {
        self.spec.info.version.clone()
    }

    fn get_title(&self) -> String {
        self.spec.info.title.clone()
    }

    fn get_description(&self) -> String {
        let description = match &self.spec.info.description {
            Some(d) => d,
            None => "",
        };

        description.to_string()
    }

    fn get_paths_len(&self) -> usize {
        * &self.spec.paths.paths.len()
    }

    fn get_paths(&self) -> Vec<Path> {
        let mut all_paths = Vec::new();

        let paths = &self.spec.paths;
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
                    warn!("No PathItem found for {:?} in spec title {:?}", path_value, self.get_title());
                }
            }
        }

        all_paths
    }

    fn get_audience(&self) -> String {
      let audience:String  = match self.spec.info.extensions.get("x-audience"){
        Some(aud) => String::from(aud.as_str().unwrap()),
        None => String::from(crate::app::dao::catalog::DEFAULT_SYSTEM_LAYER),
      };

      audience
    }

    fn get_api_id(&self) -> String {
      let api_id: String = match self.spec.info.extensions.get("x-api-id"){ // as specified https://opensource.zalando.com/restful-api-guidelines/#215
        Some(id)=> String::from(id.as_str().unwrap()),
        None => String::from("0"),
      };

      api_id
    }

    fn get_layer(&self) -> String {
      let layer:String  = match self.spec.extensions.get("x-layer"){
        Some(layer) => String::from(layer.as_str().unwrap()),
        None => String::from(crate::app::dao::catalog::DEFAULT_SYSTEM_LAYER),
      };

      layer.to_lowercase()
    }

    fn get_systems(&self) -> Vec<String> {
      let systems = match self.spec.extensions.get("x-systems"){
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
            systems.push(String::from(crate::app::dao::catalog::DEFAULT_SYSTEM_LAYER));        

            systems
        }
      };

      systems
    }

    fn get_domain(&self) -> String {
      let base_url = match self.spec.servers.is_empty() {
        true => "NA - servers attribute not specified",
        false => {
            //TODO can do better
            //base_url could have the following form http://baseurl/v1/xva-management/xva
            //will extract http://baseurl and keep the rest
            lazy_static! {
                static ref RE: Regex = Regex::new(r"(http[s]?://[a-z]*)(.*)").unwrap();
            }

            if let Some(cap) = RE.captures(&self.spec.servers[0].url) {
                cap.get(2).unwrap().as_str()
            } else {
                &self.spec.servers[0].url
            }
        }
      };

      base_url.to_string()
    }

}

#[cfg(test)]
pub mod tests {
    use crate::app::dao::catalog::handlers::SpecHandler;


    #[test]
    fn test_play_with_trait(){
        let openapi_spec = r#"
        openapi: 3.0.2
        info:
          title: Swagger Petstore
          description: |-
            This is a sample Pet Store....
          version: 1.0.17
        servers:
          - url: /api/v3
        paths:
          /pet:
            put:
              summary: Update an existing pet
              description: Update an existing pet by Id
              requestBody:
                description: Update an existent pet in the store
                content:
                  application/json:
                    schema:
                      $ref: '#/components/schemas/Pet'
                required: true
              responses:
                '200':
                  description: Successful operation
                  content:
                    application/xml:
                      schema:
                        $ref: '#/components/schemas/Pet'
                    application/json:
                      schema:
                        $ref: '#/components/schemas/Pet'
            post:
              summary: Add a new pet to the store
              description: Add a new pet to the store
              requestBody:
                description: Create a new pet in the store
                content:
                  application/json:
                    schema:
                      $ref: '#/components/schemas/Pet'
                  application/xml:
                    schema:
                      $ref: '#/components/schemas/Pet'
                  application/x-www-form-urlencoded:
                    schema:
                      $ref: '#/components/schemas/Pet'
                required: true
              responses:
                '200':
                  description: Successful operation
                  content:
                    application/xml:
                      schema:
                        $ref: '#/components/schemas/Pet'
                    application/json:
                      schema:
                        $ref: '#/components/schemas/Pet'
                '405':
                  description: Invalid input
        components:
          schemas:
            Pet:
              required:
                - name
                - photoUrls
              type: object
              properties:
                id:
                  type: integer
                  format: int64
                  example: 10
                name:
                  type: string
                  example: doggie
                photoUrls:
                  type: array
                  xml:
                    wrapped: true
                  items:
                    type: string
                    xml:
                      name: photoUrl
                tags:
                  type: array
                  xml:
                    wrapped: true
                status:
                  type: string
                  description: pet status in the store
                  enum:
                    - available
                    - pending
                    - sold
              xml:
                name: pet
          requestBodies:
            Pet:
              description: Pet object that needs to be added to the store
              content:
                application/json:
                  schema:
                    $ref: '#/components/schemas/Pet'
                application/xml:
                  schema:
                    $ref: '#/components/schemas/Pet'
        "#;


        let spec = crate::app::dao::catalog::handlers::implem::opanapi::V3::new(openapi_spec).unwrap();
        assert_eq!(spec.get_version(), "1.0.17");
        assert_eq!(spec.get_title(), "Swagger Petstore");
        assert_eq!(spec.get_description(), "This is a sample Pet Store....");
        assert_eq!(spec.get_paths_len(), 1)
    
    }

    #[test]
    fn test_get_api_id_from_spec_w_ext(){
        let mut custom_extension = indexmap::IndexMap::new();
        custom_extension.insert(
            "x-api-id".to_string(),
            serde_json::Value::String("134".to_string()),
        );

        let openapi_spec = openapiv3::OpenAPI {
            openapi: "3.0.0".to_string(),
            info: openapiv3::Info {
                title: "My API".to_string(),
                version: "1.0.0".to_string(),
                extensions: custom_extension,
                ..Default::default()
            },
            paths: Default::default(),
            ..Default::default()
        };

        let spec_as_str = serde_yaml::to_string(&openapi_spec).unwrap();

        let spec = crate::app::dao::catalog::spec::from_str("path".to_string(), "catalog_id".to_string(), "catalog_dir".to_string(), spec_as_str.as_str()).unwrap();
        let sut = spec.get_api_id();
        assert_eq!(sut, "134");
    }

    #[test]
    fn test_get_api_id_from_spec_wo_ext(){
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

        //
        let spec = crate::app::dao::catalog::spec::from_str("path".to_string(), "catalog_id".to_string(), "catalog_dir".to_string(), spec_as_str.as_str()).unwrap();

        let sut = spec.get_api_id();
        assert_eq!(sut, "0");
    }
}