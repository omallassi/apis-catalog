extern crate glob;
use glob::glob;
use log::{info, debug, warn, error};
use std::vec::Vec;

extern crate git2;
use git2::Repository;

pub fn list_openapi_files(path: &str) -> Vec<std::string::String> {
    let mut endpoints = Vec::new();

    info!("Parsing yaml files from Git Repo [{}]", path.to_string() + &"**/*.yaml".to_string());

    let pattern = format!("{}{}", path, "**/*.yaml");
    for entry in glob(&pattern).unwrap().filter_map(Result::ok) {
        endpoints.push(entry.display().to_string())
    }


    ///Users/omallassi/code/apis-catalog/catalog/portfolio-management.full-revaluation.livebook-management/rest-apis/livebook-management.yaml
    let repo = match Repository::open("/Users/omallassi/code/apis-catalog/") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    let file_path = Path::new("/Users/omallassi/code/apis-catalog/catalog/portfolio-management.full-revaluation.livebook-management/rest-apis/livebook-management.yaml");
    let oid: Oid = match repo.blob_path(file_path) {
        Ok(oid) => oid,
        Err(why) => error!("Unable to get File"),
    }

    let blob: Blob = repo.find_blob(oid);
    

    endpoints
}