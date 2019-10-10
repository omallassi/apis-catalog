extern crate glob;
use glob::glob;
use log::{info, warn};

use std::vec::Vec;
use std::path::Path;

extern crate git2;
use git2::{Repository, Oid, Blob};

use serde_yaml;
use openapiv3::OpenAPI;

//
pub struct ApiItem{
    pub name: std::string::String,
    pub id: std::string::String,
    pub api_spec: OpenAPI,
}

pub fn list_apis(path: &str) -> Vec<ApiItem> {
    let mut apis = Vec::new();
    //get connection to git repo (should be cloned as prerequisite)
    if let Ok(repo) = get_git_repo(path) {
        let pattern = format!("{}{}", path, "**/*.yaml");
        for entry in glob(&pattern).unwrap().filter_map(Result::ok) {
            let path = entry.display().to_string();
            let file_path = Path::new(&path);
            let oid: Oid = match repo.blob_path(file_path) {
                Ok(oid) => oid,
                Err(why) => { 
                    panic!("Unable to get File: {}", why);
                },
            };
            //generate the OpenAPI
            let blob: Blob = match repo.find_blob(oid) {
                Ok(blob) => blob,
                Err(why) => {
                    panic!("Unable to get Blob: {}", why);
                },
            };

            if let Ok(openapi) = serde_yaml::from_reader(blob.content()) {
                //create the API Item and add it to the returned value
                let api = ApiItem {
                    name: path, 
                    id: format!("{:?}", oid),
                    api_spec: openapi,
                };
                apis.push(api);
            }
            else{
                warn!("Unable to parse file [{}]", path);
            }
        };
    }
    else {
        warn!("Unable to parse file [{}]", path);
    }
    
    apis
}

//
pub fn get_api(path: &str, id: &str) -> Vec<ApiItem> {
    let mut apis = Vec::new();
    //get connection to git repo (should be cloned as prerequisite)
    if let Ok(repo) = get_git_repo(path) {
        //generate the OpenAPI
        if let Ok(oid) = Oid::from_str(id) {
            let blob: Blob = match repo.find_blob(oid) {
                Ok(blob) => blob,
                Err(why) => {
                    panic!("Unable to get Blob: {}", why);
                },
            };

            if let Ok(openapi) = serde_yaml::from_reader(blob.content()) {
                //create the API Item and add it to the returned value
                let api = ApiItem {
                    name: path.to_string(), 
                    id: format!("{:?}", oid),
                    api_spec: openapi,
                };
                apis.push(api);
            }
            else{
                warn!("Unable to parse file [{}]", path);
            }
        }
    }
    else {
            warn!("Unable to parse file [{}]", path);
    }

    apis
}

//
fn get_git_repo(path: &str) -> Result<Repository, git2::Error> {
    let repo: Repository = Repository::open(path)?;
    info!("Parsing yaml files from Git Repo [{}]", path.to_string() + &"**/*.yaml".to_string());

    Ok(repo)
}