extern crate glob;
use glob::glob;
use log::{debug, info, warn, error};

extern crate yaml_rust;
use yaml_rust::{Yaml, YamlLoader};

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use std::path::Path;
use std::vec::Vec;

use openapiv3::OpenAPI;
use serde_yaml;

use cmd_lib::run_cmd;

extern crate regex;
use regex::Regex;

use crate::shared::settings::{Catalog, SETTINGS};


//
#[derive(Debug, Clone)]
pub struct SpecItem {
    pub path: std::string::String,
    pub id: std::string::String,
    pub api_spec: OpenAPI,
    pub audience: std::string::String,
    pub domain: std::string::String,
    pub layer: String,
    pub systems: Vec<String>,
    pub catalog_id: String,
    pub catalog_dir: String,
}

const DEFAULT_SYSTEM_LAYER: &str = "default";

pub fn list_specs(catalogs: &Vec<Catalog>) -> Vec<SpecItem> {
    //quite counter intuitive to me but following the doc https://docs.rs/moka/0.12.0/moka/sync/struct.Cache.html 
    //To share the same cache across the threads, clone it.
    // This is a cheap operation.
    let my_cache = CACHE.cache.clone();
    let specs = match my_cache.get(&String::from("all")) {
        Some(val) => {
            info!("got [{:?}] specs from cache ", &val.len() );
            val
        },
        None => {
            info!("no specs from cache - will load the catalogs");

            let mut specs = Vec::new();
            let mut last_len = 0;
        
        
            for catalog in catalogs{
                let path = catalog.catalog_path.as_str();
        
                info!("Is loading OAI specs files from catalog [{:?}] - [{:?}]", &catalog.catalog_id, path);
                let pattern = format!("{}{}", path, "/**/*.yaml");
                for entry in glob(&pattern).unwrap().filter_map(Result::ok) {
                    let path = entry.display().to_string();
                    let file_path = Path::new(&path);
        
                    debug!("getting spec file [{:?}]", file_path);
        
                    let f = std::fs::File::open(file_path).unwrap();
        
                    match serde_yaml::from_reader(f) {
                        Ok(openapi) => {
                            //audience is defiend as x-audience and extensions are not handled by OpenAPI crate
                            let audience = get_audience_from_spec(&openapi);
                            let domain = get_domain_from_spec(&openapi);
                            let layer = get_layer_from_spec(&openapi);
                            let systems = get_systems_from_spec(&openapi);
        
                            //create the API Item and add it to the returned value
                            let mut hasher = std::collections::hash_map::DefaultHasher::new();
                            path.hash(&mut hasher);
                            let hash = hasher.finish();
                            
                            let spec = SpecItem {
                                path: path,
                                id: format!("{:?}", hash),
                                api_spec: openapi.clone(),
                                audience: audience,
                                domain: domain.to_string(),
                                layer: layer,
                                systems: systems,
                                catalog_id: String::from(&catalog.catalog_id),
                                catalog_dir: String::from(&catalog.catalog_dir)
                            };
                            specs.push(spec);
                        }
                        Err(why) => {
                            warn!("Unable to parse file [{:?}] - reason [{:?}]", path, why);
                        }
                    }
                }
                debug!("OAI specs # from catalog [{:?}] - [{:?}] is [{:?}]", &catalog.catalog_id, path, specs.len() - last_len);
                last_len = specs.len();
            }
        
            info!("OAI specs # from all catalogs - [{:?}]", &specs.len());
        
            my_cache.insert(String::from("all"), specs.to_vec());

            specs

        }
    };
    

    specs
}

fn get_audience_from_spec(spec: &OpenAPI) -> String {
    let audience:String  = match spec.info.extensions.get("x-audience"){
        Some(aud) => String::from(aud.as_str().unwrap()),
        None => String::from(DEFAULT_SYSTEM_LAYER),
    };

    audience
}

fn get_layer_from_spec(spec: &OpenAPI) -> String {
    let layer:String  = match spec.extensions.get("x-layer"){
        Some(layer) => String::from(layer.as_str().unwrap()),
        None => String::from(DEFAULT_SYSTEM_LAYER),
    };

    layer
}

fn get_systems_from_spec(openapi: &OpenAPI) -> Vec<String> {
    
    let systems = match openapi.extensions.get("x-systems"){
        Some(systems) => {
            let mut returned_systems: Vec<String> = Vec::new();
            for system in systems.as_array().unwrap(){
                //did not find a way to use into_iter().collect::Vec<String>>
                returned_systems.push(String::from(system.as_str().unwrap()));
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

pub fn get_spec_short_path(catalog_dir_srt: String, spec: &SpecItem) -> &str {
    let short_path = &spec.path[catalog_dir_srt.as_str().len()..spec.path.len()];

    short_path
}

fn get_domain_from_spec(spec: &OpenAPI) -> &str {
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
    let _ = &CACHE.cache.clone().invalidate_all();
    self::list_specs(&SETTINGS.catalogs);
}

pub fn get_zally_ignore(all_specs: &Vec<SpecItem>) -> std::collections::HashMap<i64, usize> {
    let mut merged_stats = std::collections::HashMap::new();

    // let specs = list_specs(path);
    for spec in all_specs.iter() {
        //need to load the yaml file as OpenAPI crate will remove the x-zally-ignore...
        let yaml_spec_as_string = std::fs::read_to_string(spec.path.as_str()).unwrap();
        let stats = get_zally_ignore_metrics(yaml_spec_as_string.as_str(), spec.path.as_str());

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
        let yaml_spec_as_string = std::fs::read_to_string(spec.path.as_str()).unwrap();
        let stats = get_endpoints_num_per_audience_metrics(
            yaml_spec_as_string.as_str(),
            spec.path.as_str(),
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
            spec.path
        );
        let base_url = get_domain_from_spec(&spec.api_spec);
        let num = spec.api_spec.paths.paths.len();

        *data.entry(base_url.to_string()).or_insert(0) += num;
    }

    debug!("endpoints per subdomain [{:?}]", data);

    data
}

struct Cache {
    pub cache: moka::sync::Cache<String, Vec<SpecItem>>,
}

lazy_static! {
    static ref CACHE: Cache = Cache::new();
}

impl Cache {
    fn new() -> Self {
        let cache = Cache{
            cache: moka::sync::Cache::new(2),
        };

        cache
    }

}

#[cfg(test)]
mod tests {
    use crate::{app::dao::catalog::SpecItem, shared::settings::Catalog};


    #[test]
    fn test_get_endpoints_num_per_subdomain_1() {
        let mut specs = Vec::new();
        let spec = "
        openapi: 3.0.0
        info:
          version: 1.0.0
          title: sample
        servers: 
          - url: /v1/a/b
        paths:
          /resource_1:
            get:
              responses:
                '206':
                  description: Partial Content
        ";

        let spec_item = super::SpecItem {
            path: String::from("std::string::String"),
            id: String::from("std::string::String"),
            api_spec: serde_yaml::from_str(spec).unwrap(),
            audience: String::from("std::string::String"),
            domain: String::from("std::string::String"),
            layer: String::from("std::string::String"),
            systems: Vec::new(),
            catalog_id: String::from("not used here"),
            catalog_dir: String::from("not used here"),
        };

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

        let spec_item = super::SpecItem {
            path: String::from("std::string::String"),
            id: String::from("std::string::String"),
            api_spec: serde_yaml::from_str(spec).unwrap(),
            audience: String::from("std::string::String"),
            domain: String::from("std::string::String"),
            layer: String::from("std::string::String"),
            systems: Vec::new(),
            catalog_id: String::from("not used here"),
            catalog_dir: String::from("not used here"),
        };

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

        let spec_item = super::SpecItem {
            path: String::from("std::string::String"),
            id: String::from("std::string::String"),
            api_spec: serde_yaml::from_str(spec).unwrap(),
            audience: String::from("std::string::String"),
            domain: String::from("std::string::String"),
            layer: String::from("std::string::String"),
            systems: Vec::new(),
            catalog_id: String::from("not used here"),
            catalog_dir: String::from("not used here"),
        };

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
        path.push("./tests/data/catalog/");

        let catalog = Catalog{
            catalog_id: String::from("uuid"),
            catalog_name: String::from("name"), 
            catalog_dir: String::from("not used here"),
            catalog_scm_clone_cmd: String::from("not used here"), 
            catalog_scm_pull_cmd: String::from("not used here"), 
            catalog_path: path.into_os_string().into_string().unwrap(),
            catalog_scm_clone: false,
        };
        let mut catalogs = Vec::new();
        catalogs.push(catalog);

        let results = super::list_specs(&catalogs);
        assert_eq!(results.len(), 2);
        //
        let spec: &SpecItem = results.get(0).unwrap();
        assert_eq!(spec.audience, "company");
        assert_eq!(spec.domain, "/v1/analytics/time-series");
        assert_eq!(spec.layer, super::DEFAULT_SYSTEM_LAYER);
        assert_eq!(spec.systems.len(), 1);
        assert_eq!(spec.systems[0], super::DEFAULT_SYSTEM_LAYER);

        let spec: &SpecItem = results.get(1).unwrap();
        assert_eq!(spec.audience, "company");
        assert_eq!(spec.domain, "/v1/audit/trails");
        assert_eq!(spec.layer, "application");
        assert_eq!(spec.systems[0], "bpaas");
    }
}
