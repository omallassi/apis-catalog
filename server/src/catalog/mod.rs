extern crate glob;
use glob::glob;
use log::{info, debug, warn, error};
use std::vec::Vec;
use std::path::Path;

extern crate git2;
use git2::{Repository, Oid, Blob};

pub fn list_openapi_files(path: &str) -> Vec<std::string::String> {
    let mut endpoints = Vec::new();

    info!("Parsing yaml files from Git Repo [{}]", path.to_string() + &"**/*.yaml".to_string());

    let pattern = format!("{}{}", path, "**/*.yaml");
    for entry in glob(&pattern).unwrap().filter_map(Result::ok) {
        endpoints.push(entry.display().to_string())
    }

    let repo: Repository = match Repository::open("/Users/omallassi/code/apis-catalog/") {
        Ok(repo) => repo,
        Err(e) => {
            panic!("failed to open: {}", e);
        },
    };

    let file_path = Path::new("/Users/omallassi/code/apis-catalog/catalog/portfolio-management.full-revaluation.livebook-management/rest-apis/livebook-management.yaml");
    let oid: Oid = match repo.blob_path(file_path) {
        Ok(oid) => oid,
        Err(why) => { 
            panic!("Unable to get File: {}", why);
        },
    };
    
    let blob: Blob = match repo.find_blob(oid) {
        Ok(blob) => blob,
        Err(why) => {
            panic!("Unable to get Blob: {}", why);
        },
    };

    println!("{}", blob.id());

    let new_oid: Oid = Oid::from_str("50d7daabbdb611c15137bcfd92e4b9e134f6d417").unwrap();

    println!("{:?}", new_oid);
    let blob: Blob = match repo.find_blob(oid) {
        Ok(blob) => blob,
        Err(why) => {
            panic!("Unable to get Blob: {}", why);
        },
    };

    println!("{}", blob.id());
    
    endpoints
}