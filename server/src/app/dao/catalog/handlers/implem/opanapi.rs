use openapiv3::OpenAPI;

use crate::app::dao::catalog::handlers::{SpecHandler, Path, Method};
use log::{debug, info, warn, error};

pub struct v3 {
    pub spec: OpenAPI,
}
impl v3 {
    pub fn new(val: &str) -> v3 {
        v3 {
            spec: serde_yaml::from_str(val).unwrap(),
        }
    }
}
impl SpecHandler for v3{
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
                    warn!("No path to index for spec {:?}", "TODO &self.path");
                }
            }
        }

        all_paths
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


        let spec = crate::app::dao::catalog::handlers::implem::opanapi::v3::new(openapi_spec);
        assert_eq!(spec.get_version(), "1.0.17");
        assert_eq!(spec.get_title(), "Swagger Petstore");
        assert_eq!(spec.get_description(), "This is a sample Pet Store....");
        assert_eq!(spec.get_paths_len(), 1)
    
    }
}