extern crate glob;
use glob::glob;
use log::{debug, info, warn};

extern crate yaml_rust;
use yaml_rust::{Yaml, YamlLoader};

use std::collections::HashMap;

use std::path::Path;
use std::vec::Vec;

extern crate git2;
use git2::{Blob, Oid, Repository};

use openapiv3::OpenAPI;
use serde_yaml;

use cmd_lib::run_cmd;

extern crate regex;
use regex::Regex;

use std::fs;

//
#[derive(Debug, Clone)]
pub struct SpecItem {
    pub path: std::string::String,
    pub id: std::string::String,
    pub api_spec: OpenAPI,
    pub audience: std::string::String,
    pub domain: std::string::String,
}

pub fn list_specs(path: &str) -> Vec<SpecItem> {
    let mut specs = Vec::new();
    //get connection to git repo (should be cloned as prerequisite)
    if let Ok(repo) = get_git_repo(path) {
        let pattern = format!("{}{}", path, "/**/*.yaml"); //TODO fragile
        for entry in glob(&pattern).unwrap().filter_map(Result::ok) {
            let path = entry.display().to_string();
            let file_path = Path::new(&path);
            let oid: Oid = match repo.blob_path(file_path) {
                Ok(oid) => oid,
                Err(why) => {
                    panic!("Unable to get File: {}", why);
                }
            };
            //generate the OpenAPI
            let blob: Blob = match repo.find_blob(oid) {
                Ok(blob) => blob,
                Err(why) => {
                    panic!("Unable to get Blob: {}", why);
                }
            };

            match serde_yaml::from_reader(blob.content()) {
                Ok(openapi) => {
                    //audience is defiend as x-audience and extensions are not handled by OpenAPI crate
                    //TODO this whole thing has to be reworked
                    let audience = match get_audience_from_spec(&file_path) {
                        Some(aud) => aud,
                        None => String::from("N/A"),
                    };
                    let domain = get_domain_from_spec(&openapi);
                    //create the API Item and add it to the returned value
                    let spec = SpecItem {
                        path: path,
                        id: format!("{:?}", oid),
                        api_spec: openapi.clone(),
                        audience: audience,
                        domain: domain.to_string(),
                    };
                    specs.push(spec);
                }
                Err(why) => {
                    warn!("Unable to parse file [{:?}] - reason [{:?}]", path, why);
                }
            }
        }
    } else {
        warn!("Unable to get git repo from path [{}]", path);
    }

    specs
}

fn get_audience_from_spec(spec: &Path) -> Option<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(x-audience)(\s*:)(.+)").unwrap();
    }
    let spec_content = fs::read_to_string(spec).unwrap_or_default();

    if let Some(cap) = RE.captures(spec_content.as_str()) {
        debug!(
            "found x-audience [{}] in spec [{:?}]",
            cap[3].to_string(),
            spec
        );
        Some(cap[3].to_string())
    } else {
        debug!("Unable to x-audience from [{:?}]", spec);
        None
    }
}

//
pub fn get_spec(path: &str, id: &str) -> Vec<SpecItem> {
    let mut specs = Vec::new();
    //get connection to git repo (should be cloned as prerequisite)
    if let Ok(repo) = get_git_repo(path) {
        //generate the OpenAPI
        if let Ok(oid) = Oid::from_str(id) {
            let blob: Blob = match repo.find_blob(oid) {
                Ok(blob) => blob,
                Err(why) => {
                    panic!("Unable to get Blob: {}", why);
                }
            };

            if let Ok(openapi) = serde_yaml::from_reader(blob.content()) {
                //audience is defiend as x-audience and extensions are not handled by OpenAPI crate
                //TODO this whole thing has to be reworked
                let audience = match get_audience_from_spec(Path::new(path)) {
                    Some(aud) => aud,
                    None => String::from("N/A"),
                };
                let domain = get_domain_from_spec(&openapi);
                //create the API Item and add it to the returned value
                let spec = SpecItem {
                    path: path.to_string(),
                    id: format!("{:?}", oid),
                    api_spec: openapi.clone(),
                    audience: audience,
                    domain: domain.to_string(),
                };
                specs.push(spec);
            } else {
                warn!("Unable to parse file [{}]", path);
            }
        }
    } else {
        warn!("Unable to get git repo from path [{}]", path);
    }

    specs
}

pub fn get_spec_short_path(catalog_dir_srt: String, spec: &SpecItem) -> &str {
    let short_path = &spec.path[catalog_dir_srt.as_str().len()..spec.path.len()];

    short_path
}

//
fn get_git_repo(path: &str) -> Result<Repository, git2::Error> {
    let repo: Repository = Repository::open(path)?;
    info!(
        "Parsing yaml files from Git Repo [{}]",
        path.to_string() + &"**/*.yaml".to_string()
    );

    Ok(repo)
}

pub fn refresh_git_repo(path: &str) {
    //TODO maybe a cleaner way https://github.com/rust-lang/git2-rs/commit/f3b87baed1e33d6c2d94fe1fa6aa6503a071d837
    //TODO be more proper on error management here.typical case: credentials to git pull are no longer working...
    run_cmd!("cd {}; git pull", path).unwrap();
    info!("Refresh Git Repo with result [{:?}]", "result");
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

    // let iter = doc.iter();
    // for item in iter {
    //     println!("---------");
    //     println!("{:?}", &item);
    //     println!("---------");
    // }
    let mut stats = std::collections::HashMap::new();
    //get global zally-ignore
    {
        match doc.get(&Yaml::String(String::from("x-zally-ignore"))) {
            Some(val) => {
                //println!("x-zally-ignore {:?}", val);

                let paths = doc
                    .get(&Yaml::String(String::from("paths")))
                    .unwrap()
                    .as_hash()
                    .unwrap();

                for elt in val.as_vec().unwrap() {
                    stats.insert(elt.as_i64().unwrap(), paths.len());
                    // println!(
                    //     "x-zally-ignore {:?} {:?}",
                    //     elt.as_i64(),
                    //     elt.as_i64().unwrap()
                    // );
                    // println!("path len {:?}", paths.len());
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
        let num = spec.api_spec.paths.len();

        *data.entry(base_url.to_string()).or_insert(0) += num;
    }

    debug!("endpoints per subdomain [{:?}]", data);

    data
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

#[cfg(test)]
mod tests {

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
}
