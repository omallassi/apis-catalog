extern crate glob;
use glob::glob;
use log::{info, warn};

use std::vec::Vec;
use std::path::Path;

extern crate git2;
use git2::{Repository, Oid, Blob};

use serde_yaml;
use openapiv3::OpenAPI;

use cmd_lib::{run_cmd, CmdResult};

//
pub struct SpecItem{
    pub name: std::string::String,
    pub id: std::string::String,
    pub api_spec: OpenAPI,
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
                let spec = SpecItem {
                    name: path, 
                    id: format!("{:?}", oid),
                    api_spec: openapi,
                };
                specs.push(spec);
            }
            else{
                warn!("Unable to parse file [{}]", path);
            }
        };
    }
    else {
        warn!("Unable to parse file [{}]", path);
    }
    
    specs
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
                },
            };

            if let Ok(openapi) = serde_yaml::from_reader(blob.content()) {
                //create the API Item and add it to the returned value
                let spec = SpecItem {
                    name: path.to_string(), 
                    id: format!("{:?}", oid),
                    api_spec: openapi,
                };
                specs.push(spec);
            }
            else{
                warn!("Unable to parse file [{}]", path);
            }
        }
    }
    else {
            warn!("Unable to parse file [{}]", path);
    }

    specs
}

//
fn get_git_repo(path: &str) -> Result<Repository, git2::Error> {
    let repo: Repository = Repository::open(path)?;
    info!("Parsing yaml files from Git Repo [{}]", path.to_string() + &"**/*.yaml".to_string());

    Ok(repo)
}

pub fn refresh_git_repo(path: &str)  {
    run_cmd!("cd {}; git pull", path);
    info!("Refresh Git Repo with result [{:?}]", "result");
}

