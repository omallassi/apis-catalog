extern crate glob;
use glob::glob;
use log::{info, debug, warn, error};
use std::vec::Vec;
use std::path::Path;

extern crate git2;
use git2::{Repository, Oid, Blob};

//
pub struct ApiItem{
    pub name: std::string::String,
    pub id: std::string::String
}

pub fn list_openapi_files(path: &str) -> Vec<ApiItem> {
    let mut endpoints = Vec::new();
    //get connection to git repo (should be cloned as prerequisite)
    let repo: Repository = match Repository::open(path) {
        Ok(repo) => repo,
        Err(e) => {
            panic!("failed to open git repo [{}]", e);
        },
    };

    info!("Parsing yaml files from Git Repo [{}]", path.to_string() + &"**/*.yaml".to_string());

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
        //create the API Item and add it to the returned value
        let api = ApiItem {
            name: path, 
            id: format!("{:?}", oid),
        };
        endpoints.push(api);
    };
    
    endpoints
}


//
pub fn get_openapi_file(path: &str, id: &str) -> Vec<ApiItem> {
    let mut endpoints = Vec::new();
    //get connection to git repo (should be cloned as prerequisite)
    let repo: Repository = match Repository::open(path) {
        Ok(repo) => repo,
        Err(e) => {
            panic!("failed to open git repo [{}]", e);
        },
    };

    info!("Parsing yaml files from Git Repo [{}]", path.to_string() + &"**/*.yaml".to_string());

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
        //create the API Item and add it to the returned value
        let api = ApiItem {
            name: path, 
            id: format!("{:?}", oid),
        };
        endpoints.push(api);
    };

    

    // let file_path = Path::new("/Users/omallassi/code/apis-catalog/catalog/portfolio-management.full-revaluation.livebook-management/rest-apis/livebook-management.yaml");
    // let oid: Oid = match repo.blob_path(file_path) {
    //     Ok(oid) => oid,
    //     Err(why) => { 
    //         panic!("Unable to get File: {}", why);
    //     },
    // };
    
    // let blob: Blob = match repo.find_blob(oid) {
    //     Ok(blob) => blob,
    //     Err(why) => {
    //         panic!("Unable to get Blob: {}", why);
    //     },
    // };

    // println!("{}", blob.id());

    // let new_oid: Oid = Oid::from_str("50d7daabbdb611c15137bcfd92e4b9e134f6d417").unwrap();

    // println!("{:?}", new_oid);
    // let blob: Blob = match repo.find_blob(oid) {
    //     Ok(blob) => blob,
    //     Err(why) => {
    //         panic!("Unable to get Blob: {}", why);
    //     },
    // };

    // println!("{}", blob.id());
    
    endpoints
}