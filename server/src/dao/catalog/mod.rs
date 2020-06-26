extern crate glob;
use glob::glob;
use log::{info, warn};

use std::path::Path;
use std::vec::Vec;

extern crate git2;
use git2::{Blob, Oid, Repository};

use openapiv3::OpenAPI;
use serde_yaml;

use cmd_lib::run_cmd;

extern crate regex;
use regex::Regex;

//
pub struct SpecItem {
    pub name: std::string::String,
    pub id: std::string::String,
    pub api_spec: OpenAPI,
    pub audience: std::string::String,
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

            if let Ok(openapi) = serde_yaml::from_reader(blob.content()) {
                let audience = match get_audience_from_spec(&openapi) {
                    Some(aud) => aud,
                    None => String::from("N/A"),
                };
                //create the API Item and add it to the returned value
                let spec = SpecItem {
                    name: path,
                    id: format!("{:?}", oid),
                    api_spec: openapi,
                    audience: audience,
                };
                specs.push(spec);
            } else {
                warn!("Unable to parse file [{}]", path);
            }
        }
    } else {
        warn!("Unable to parse file [{}]", path);
    }

    specs
}

fn get_audience_from_spec(spec: &OpenAPI) -> Option<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(x-audience)(\s*:)(.+)").unwrap();
    }
    if let Some(cap) = RE.captures(spec.openapi.as_str()) {
        info!("found x-audience [{}]", cap[3].to_string());
        Some(cap[3].to_string())
    } else {
        info!("Unable to x-audience from [{}]", spec.info.title);
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
                let audience = match get_audience_from_spec(&openapi) {
                    Some(aud) => aud,
                    None => String::from("N/A"),
                };
                //create the API Item and add it to the returned value
                let spec = SpecItem {
                    name: path.to_string(),
                    id: format!("{:?}", oid),
                    api_spec: openapi,
                    audience: audience,
                };
                specs.push(spec);
            } else {
                warn!("Unable to parse file [{}]", path);
            }
        }
    } else {
        warn!("Unable to parse file [{}]", path);
    }

    specs
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
    run_cmd!("cd {}; git pull", path);
    info!("Refresh Git Repo with result [{:?}]", "result");
}
