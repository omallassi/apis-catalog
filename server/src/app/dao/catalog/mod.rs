///sub module declaration
pub mod handlers;
pub mod spec;

///import
use log::{debug, info, warn, error};
extern crate yaml_rust;
use yaml_rust::{Yaml, YamlLoader};
use std::collections::HashMap;
use std::vec::Vec;
use cmd_lib::run_cmd;
use crate::shared::settings::{Catalog, SETTINGS};

use self::spec::SpecItem;

#[derive(Debug, Clone)]
pub struct SpecInError {
    pub file_path: String, 
    pub reason: String,
}

const DEFAULT_SYSTEM_LAYER: &str = "default";

pub fn list_specs(catalogs: &Vec<Catalog>) -> Vec<SpecItem> {
    let specs = match CACHE.cache.get(&String::from("all")) {
        Some(val) => {
            info!("got [{:?}] specs from cache ", &val.len() );
            val
        },
        None => {
            info!("no specs from cache - will load the catalogs");

            let mut specs = Vec::new();
            let mut last_len = 0;
            let mut specs_in_error: Vec<SpecInError> = Vec::new();
        
            for catalog in catalogs{
                let path = catalog.catalog_path.as_str();
        
                info!("Is loading OAI specs files from catalog [{:?}] - [{:?}] with glob pattern {:?}", &catalog.catalog_id, path, &catalog.catalog_include_glob_pattern);
                //let pattern = format!("{}{}", path, "/**/*.yaml");

                let walker = globwalk::GlobWalkerBuilder::from_patterns(
                    path,
                    &catalog.catalog_include_glob_pattern //&["**/*.{yml,yaml}", "!**/{test,tests}/*"],
                    )
                    .build()
                    .unwrap()
                    .into_iter()
                    .filter_map(Result::ok);

                for entry in walker {
                    let file_path: &std::path::Path = entry.path();
                    debug!("getting spec file [{:?}]", &file_path);

                    let file_content = std::fs::read_to_string(&file_path).unwrap();
                    let path = String::from(file_path.to_str().unwrap());
                    let catalog_id = String::from(&catalog.catalog_id);
                    let catalog_dir = String::from(&catalog.catalog_dir);
                
                    ///
                    match crate::app::dao::catalog::spec::from_str(path, catalog_id, catalog_dir, file_content.as_str()) {
                        Ok(spec) => {
                            specs.push(spec);
                        }
                        Err(why) => {
                            specs_in_error.push(SpecInError { file_path: format!("{:?}", file_path) , reason: format!("{:?}", why) })
                        }
                    }
                }
                
                debug!("OAI specs # from catalog [{:?}] - [{:?}] is [{:?}]", &catalog.catalog_id, path, specs.len() - last_len);
                last_len = specs.len();
            }
        
            info!("OAI specs # from all catalogs - [{:?}]", &specs.len());
        
            //TODO  the underlying line create a stack overflow (while clone the box)
            CACHE.cache.insert(String::from("all"), specs.to_vec());


            CACHE.errors.insert(String::from("all"), specs_in_error.to_vec());

            specs

        }
    };
    

    specs
}

pub fn list_all_errors() -> Vec<SpecInError> {
    let errors = match CACHE.errors.get(&String::from("all")) {
        Some(val) => val,
        None => {
            error!("Unable to get all errors from cache");
            Vec::new()
        }
    };

    errors
}


pub fn refresh_catalogs(catalogs: &Vec<Catalog>, init: bool) {

    for catalog in catalogs {
        match init {
            true => {

                match catalog.catalog_scm_clone {
                    true => {
                        let catalog_scm_cmd = catalog.catalog_scm_clone_cmd.to_owned();
                        let catalog_path = catalog.catalog_path.to_owned();
        
                        let cmd = format!("{catalog_scm_cmd} {catalog_path}" );
                        cmd_lib::set_debug(true);
                        match run_cmd!{ 
                            //var a considered as String here; bash -c will make it work (refer to man bash)
                            bash -c ${cmd}; 
                        }
                        {
                            Ok(val) => {
                                info!("Clone Git Repo [{:?}] into [{:?}] - got [{:?}]", catalog_scm_cmd, catalog_path, val);
                            }, 
                            Err(e) => {
                                error!("Error while cloning Git Repo [{:?}] into [{:?}] - [{:?}]", catalog_scm_cmd, catalog_path, e);
                            }
                        }
                    }, 
                    false => {
                        warn!("Catalog [{:?}] - [{:?}] will not be cloned", catalog.catalog_id, catalog.catalog_name);
                    }
                }
            }, 
            false => {
                let catalog_scm_cmd = catalog.catalog_scm_pull_cmd.as_str();
                let catalog_path = catalog.catalog_path.as_str();
                
                cmd_lib::set_debug(true);

                match run_cmd!{ cd ${catalog_path}; bash -c ${catalog_scm_cmd} }{
                    Ok(val) => {
                        info!("Refresh Git Repo [{:?}] into [{:?}] - got [{:?}]", catalog_scm_cmd, catalog_path, val);
                    }, 
                    Err(e) => {
                        error!("Error while refreshing Git Repo [{:?}] into [{:?}] - [{:?}]", catalog_scm_cmd, catalog_path, e);
                    }
                }
            }
        };
    }

    //will force data back in cache
    let _ = &CACHE.invalidate_all();
    
    self::list_specs(&SETTINGS.catalogs);
}

pub fn get_zally_ignore(all_specs: &Vec<SpecItem>) -> std::collections::HashMap<i64, usize> {
    let mut merged_stats = std::collections::HashMap::new();

    // let specs = list_specs(path);
    for spec in all_specs.iter() {
        //need to load the yaml file as OpenAPI crate will remove the x-zally-ignore...
        let yaml_spec_as_string = std::fs::read_to_string(spec.get_file_path()).unwrap();
        let stats = get_zally_ignore_metrics(yaml_spec_as_string.as_str(), spec.get_file_path());

        //some the maps
        for (key, val) in stats.iter() {
            match merged_stats.get(key) {
                Some(known_val) => {
                    merged_stats.insert(*key, val + known_val);
                }
                None => {
                    merged_stats.insert(*key, *val);
                }
            }
        }
    }
    merged_stats
}

fn get_zally_ignore_metrics(spec: &str, spec_name: &str) -> std::collections::HashMap<i64, usize> {
    debug!(
        "get_zally_ignore_metrics is called for spec {:?}",
        spec_name
    );

    let docs = match YamlLoader::load_from_str(spec) {
        Ok(docs) => docs,
        Err(why) => {
            panic!("Error while parsing spec {} - :{:?}", spec, why);
        }
    }; // Result<Vec<Yaml>, ScanError>
    let doc = docs[0].as_hash().unwrap(); //Option<&Hash> et LinkedHashMap<Yaml, Yaml>;

    let mut stats = std::collections::HashMap::new();
    //get global zally-ignore
    {
        match doc.get(&Yaml::String(String::from("x-zally-ignore"))) {
            Some(val) => {
                info!("x-zally-ignore {:?}", val);

                let paths = doc
                    .get(&Yaml::String(String::from("paths")))
                    .unwrap()
                    .as_hash()
                    .unwrap();

                info!("x-paths {:?}", paths);

                for elt in val.as_vec().unwrap() {
                    match elt.as_i64() {
                        Some(val) => {
                            stats.insert(val, paths.len());
                        }, 
                        None => {
                            //some zally ignore are String . as exple - tt String("M010")
                            warn!("unable to parse zally-ignore {:?}", elt);
                        },
                    };

                    
                }
            }
            None => info!("no global zally-ignore for spec {:?}", spec_name),
        };
    }

    //get zally-ignore per path
    let mut stats_per_path: HashMap<i64, usize> = std::collections::HashMap::new();
    {
        let paths = doc
            .get(&Yaml::String(String::from("paths")))
            .unwrap()
            .as_hash()
            .unwrap();

        for path in paths.iter() {
            // println!("{:?}", path.0);
            // println!("{:?}", path.1);
            let zally = path
                .1
                .as_hash()
                .unwrap()
                .get(&Yaml::String(String::from("x-zally-ignore")));

            match zally {
                Some(val) => {
                    for elt in val.as_vec().unwrap() {
                        let elt = match elt.as_i64() {
                            Some(val) => val,
                            None => {
                                warn!("Got zally-ignore [{:?}]", elt);
                                -1
                            }
                        };
                        let stat = stats_per_path.get(&elt).cloned();
                        match stat {
                            Some(val) => {
                                stats_per_path.insert(elt, val + 1);
                            }
                            None => {
                                stats_per_path.insert(elt, 1);
                            }
                        }
                    }
                }
                None => {
                    info!(
                        "no zally-ignore on paths for spec {:?} and path {:?}",
                        spec_name, path.0
                    );
                }
            }
        }
        // println!("stats_per_path {:?}", stats_per_path);
        // println!("len {:?}", paths.len());
    }

    //merge both maps
    for stat in stats_per_path.iter() {
        //check if stat already exist in global, if not add it to stats
        if stats.contains_key(stat.0) {
            debug!("stats {:?} already in global stats", stat.0);
        } else {
            stats.insert(*stat.0, *stat.1);
        }
    }

    stats
}

pub fn get_endpoints_num_per_audience(
    all_specs: &Vec<SpecItem>,
) -> std::collections::HashMap<String, usize> {
    let mut merged_stats = std::collections::HashMap::new();

    for spec in all_specs.iter() {
        //need to load the yaml file as OpenAPI crate will remove the x-zally-ignore...
        let yaml_spec_as_string = std::fs::read_to_string(spec.get_file_path()).unwrap();
        let stats = get_endpoints_num_per_audience_metrics(
            yaml_spec_as_string.as_str(),
            spec.get_file_path(),
        );

        //sum the maps
        for (key, val) in stats.iter() {
            match merged_stats.get(key) {
                Some(known_val) => {
                    merged_stats.insert(key.clone(), val + known_val);
                }
                None => {
                    merged_stats.insert(key.clone(), *val);
                }
            }
        }
    }
    merged_stats
}

fn get_endpoints_num_per_audience_metrics(
    spec: &str,
    spec_name: &str,
) -> std::collections::HashMap<String, usize> {
    debug!(
        "get_endpoints_num_per_audience_metrics is called for spec {:?}",
        spec_name
    );

    let docs = match YamlLoader::load_from_str(spec) {
        Ok(docs) => docs,
        Err(why) => {
            panic!("Error while parsing spec {} - :{:?}", spec, why);
        }
    }; // Result<Vec<Yaml>, ScanError>
    let doc = docs[0].as_hash().unwrap(); //Option<&Hash> et LinkedHashMap<Yaml, Yaml>;
    let doc_info_tag = doc
        .get(&Yaml::String(String::from("info")))
        .unwrap()
        .as_hash()
        .unwrap(); //Option<&Hash> et LinkedHashMap<Yaml, Yaml>;

    let paths = doc
        .get(&Yaml::String(String::from("paths")))
        .unwrap()
        .as_hash()
        .unwrap();
    let num_of_endpoints = paths.len();

    let mut stats = std::collections::HashMap::new();
    {
        match doc_info_tag.get(&Yaml::String(String::from("x-audience"))) {
            Some(val) => {
                info!("found audience [{:?}] for spec [{:?}]", val, spec_name);
                let audience_name = String::from(val.as_str().unwrap());
                match stats.get(&audience_name) {
                    Some(val) => {
                        stats.insert(audience_name, val + num_of_endpoints);
                    }
                    None => {
                        stats.insert(audience_name, num_of_endpoints);
                    }
                };
            }
            None => {
                info!("no audience for spec [{:?}]", spec_name);
                stats.insert(String::from("no audience"), num_of_endpoints);
            }
        };
    };

    stats
}

pub fn get_endpoints_num_per_subdomain(all_specs: &Vec<SpecItem>) -> HashMap<String, usize> {
    let mut data: HashMap<String, usize> = HashMap::new();
    for spec in all_specs {
        debug!(
            "get_endpoints_num_per_subdomain - parsing spec [{:?}]",
            spec.get_file_path()
        );
        let base_url = spec.get_domain();
        let num = spec.get_paths().len();

        *data.entry(base_url.to_string()).or_insert(0) += num;
    }

    debug!("endpoints per subdomain [{:?}]", data);

    data
}

struct Cache {
    //TODO there is likely a way to have a Cache that can Store Any - but I am struggling with + Send + Sync
    cache: quick_cache::sync::Cache<String, Vec<SpecItem>>,
    errors: quick_cache::sync::Cache<String, Vec<SpecInError>>,
}

lazy_static! {
    static ref CACHE: Cache = Cache::new();
}

impl Cache {
    fn new() -> Self {
        let cache = Cache{
            cache: quick_cache::sync::Cache::new(2),
            errors: quick_cache::sync::Cache::new(2),
        };

        cache
    }

    fn invalidate_all(&self){
        let _ = self.cache.remove("all");
        let _ = self.errors.remove("all");
    } 
}

#[cfg(test)]
pub mod tests {
    use crate::{app::dao::catalog::SpecItem, shared::settings::Catalog};

    /// This method will return a mocked Vec<SpecItem> that can be used for 
    /// test purposes.
    /// # Examples 
    /// 
    /// ```
    /// let specs = crate::app::dao::catalog::tests::get_mocked_specs();
    /// ```
    pub fn get_mocked_specs() -> Vec<SpecItem>{

        let spec = "
        openapi: 3.0.0
        info:
          version: 1.0.0
          title: sample
        paths:
          /resource_1:
            summary: Update an existing pet
            get:
              summary: Update an existing pet
              description: Update an existing pet by Id
              operationId: updatePet
              responses:
                '206':
                  description: Partial Content
        ";

        let path = String::from("/path/to/spec.yaml");
        let catalog_id = String::from("an id");
        let catalog_dir = String::from("not used here");

        let spec_item = super::spec::from_str(path, catalog_id, catalog_dir, spec).unwrap();

        let mut specs = Vec::new();
        specs.push(spec_item);

        specs
    }

    #[test]
    fn test_get_endpoints_num_per_subdomain_1() {
        let mut specs = Vec::new();

        let spec = r#"
            openapi: 3.0.0
            info:
                version: 1.0.0
                title: sample
            servers: 
                - url: /v1/a/b
            paths:
                /resource_1:
                    get:
                    summary: Update an existing pet
                    description: Update an existing pet by Id
                    operationId: updatePet
        "#;

        let path = String::from("std::string::String");
        let catalog_id = String::from("not used here");
        let catalog_dir = String::from("not used here");

        let spec_item = super::spec::from_str(path, catalog_id, catalog_dir, spec).unwrap();

        specs.push(spec_item);

        let spec = "
        openapi: 3.0.0
        info:
          version: 1.0.0
          title: sample
        servers: 
          - url: /v1/a/c
        paths:
          /resource_1:
            get:
              responses:
                '206':
                  description: Partial Content
                  /resource_1:
            post:
              responses:
                '206':
                  description: Partial Content
        ";

        let catalog_id = String::from("not used here");
        let catalog_dir = String::from("not used here");
        let path = String::from("std::string::String");

        let spec_item = super::spec::from_str(path, catalog_id, catalog_dir, spec).unwrap();

        specs.push(spec_item);

        let spec = "
        openapi: 3.0.0
        info:
          version: 1.0.0
          title: sample
        paths:
          /resource_1:
            get:
              responses:
                '206':
                  description: Partial Content
        ";

        let path = String::from("std::string::String");
        let catalog_id = String::from("not used here");
        let catalog_dir = String::from("not used here");

        let spec_item = super::spec::from_str(path, catalog_id, catalog_dir, spec).unwrap();

        specs.push(spec_item);

        let data = super::get_endpoints_num_per_subdomain(&specs);

        assert_eq!(data.get("/v1/a/c").unwrap(), &1usize);
        assert_eq!(data.get("/v1/a/b").unwrap(), &1usize);
        assert_eq!(
            data.get("NA - servers attribute not specified").unwrap(),
            &1usize
        );
    }

    #[test]
    fn test_get_zally_ignore_metrics_1() {
        let spec = "
        openapi: \"3.0.0\"
        info:
          version: 1.0.0
          title: an API ...
        x-zally-ignore:
          - 134
          - 120 # Rest maturity evolving
        
        paths:
          /v1/a/b:
            get:
              description: get ...
              responses:
                '200':
                  description: returns...

          /v2/a/b:
            x-zally-ignore:
              - 164
            get:
              description: get ...
              responses:
                '200':
                  description: returns...
                    
          /a/b:
            x-zally-ignore:
              - 164 # Rest maturity evolving
              - 134
            post:
              parameters:
                - name: chunk
                  in: query
                  required: true
                  schema:
                    type: integer
                    format: int32
                    minimum: 1
              responses:
                200:
                  description: ...     
        ";

        let results = super::get_zally_ignore_metrics(spec, "name");

        println!("*** results : {:?}", results);

        assert_eq!(results.get(&134i64).unwrap(), &3usize);
        assert_eq!(results.get(&120i64).unwrap(), &3usize);
        assert_eq!(results.get(&164i64).unwrap(), &2usize);
    }

    #[test]
    fn test_get_zally_ignore_metrics_2() {
        let spec = "
        openapi: \"3.0.0\"
        info:
          version: 1.0.0
          title: an API ...
        
        paths:
          /v1/a/b:
            get:
              description: get ...
              responses:
                '200':
                  description: returns...
                    
          /a/b:
            x-zally-ignore:
              - M10
              - 164 
            post:
              parameters:
                - name: chunk
                  in: query
                  required: true
                  schema:
                    type: integer
                    format: int32
                    minimum: 1
              responses:
                200:
                  description: ...       
        ";

        let results = super::get_zally_ignore_metrics(spec, "name");

        println!("*** results : {:?}", results);

        assert_eq!(results.get(&164i64).unwrap(), &1usize);
        assert_eq!(results.get(&-1i64).unwrap(), &1usize);
    }

    #[test]
    fn test_get_endpoints_num_per_audience_ignore_metrics_1() {
        let spec = "
        openapi: \"3.0.0\"
        info:
          version: 1.0.0
          title: an API ...
          x-audience: an audience
        
        paths:
          /v1/a/b:
            get:
              description: get ...
              responses:
                '200':
                  description: returns...

          /v2/a/b:
            get:
              description: get ...
              responses:
                '200':
                  description: returns...  
        ";

        let results = super::get_endpoints_num_per_audience_metrics(spec, "name");

        assert_eq!(results.get("an audience").unwrap(), &2usize);
    }

    #[test]
    fn test_list_all_specs() {
        let mut path = std::path::PathBuf::new();
        path.push(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/data/catalog/");

        let catalog = Catalog{
            catalog_id: String::from("uuid"),
            catalog_name: String::from("name"), 
            catalog_dir: String::from("not used here"),
            catalog_include_glob_pattern: vec![ String::from("**/*.{yml,yaml}"), String::from("!**/{test,tests}/*") ],
            catalog_scm_clone_cmd: String::from("not used here"), 
            catalog_scm_pull_cmd: String::from("not used here"), 
            catalog_path: path.into_os_string().into_string().unwrap(),
            catalog_scm_clone: false,
            catalog_http_base_uri: String::from("not used here")
        };
        let mut catalogs = Vec::new();
        catalogs.push(catalog);

        let results = super::list_specs(&catalogs);
        assert_eq!(results.len(), 2);
        //
        let spec: &SpecItem = results.get(0).unwrap();
        assert_eq!( &spec.get_audience(), "company");
        assert_eq!( spec.get_domain(), "/v1/analytics/time-series");
        assert_eq!( &spec.get_layer(), super::DEFAULT_SYSTEM_LAYER);
        assert_eq!( spec.get_systems().len(), 1);
        assert_eq!( &spec.get_systems()[0], super::DEFAULT_SYSTEM_LAYER);

        let spec: &SpecItem = results.get(1).unwrap();
        assert_eq!( &spec.get_audience(), "company");
        assert_eq!( spec.get_domain(), "/v1/audit/trails");
        assert_eq!( &spec.get_layer(), "application");
        assert_eq!( &spec.get_systems()[0], "bpaas");
    }

}

