extern crate clap;
use clap::{Arg, App, SubCommand};

#[macro_use] extern crate prettytable;
use prettytable::{Table, Row, Cell};
use prettytable::format;

extern crate reqwest;
use reqwest::{Client};
use serde::{Deserialize, Serialize};

extern crate log;
extern crate env_logger;
use log::{info, debug, warn, error};

use std::vec::Vec;

//
#[derive(Serialize, Deserialize, Debug)]
struct Endpoints {
    endpoints: Vec<Endpoint>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Endpoint {
    name: String,
}

fn get_endpoints(_api: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let mut resp = client.get("http://127.0.0.1:8088/v1/endpoints").send()?;
    debug!("body: {:?}", resp.status());
    let endpoints: Endpoints = resp.json()?;
    //
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row![b -> "Endpoints", b -> "Deployed Version"]);
    for val in endpoints.endpoints {
        table.add_row(row![val.name, "..."]);
    };
    
    // Print the table to stdout
    table.printstd();

    Ok(())
}

//
#[derive(Serialize, Deserialize)]
struct Apis {
    apis: Vec<Api>,
}

#[derive(Serialize, Deserialize)]
struct Api {
    name: String,
}

fn get_apis() -> Result<(), reqwest::Error> {
    let client = Client::new();
    let mut resp = client.get("http://127.0.0.1:8088/v1/apis").send()?;
    debug!("body: {:?}", resp.status());
    let apis: Apis = resp.json()?;
    //
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row![b -> "Apis", b -> "Deployed Version"]);
    for val in apis.apis {
        table.add_row(row![val.name, "..."]);
    };
    
    // Print the table to stdout
    table.printstd();

    Ok(())
}

//
#[derive(Serialize, Deserialize, Debug)]
struct Release {
    api: String, 
    commit_id: String,
}

fn release(api: &str, commit: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let release = Release {
        api: api.to_string(), 
        commit_id: commit.to_string(),
    };
    let mut resp = client.post("http://127.0.0.1:8088/v1/releases").json(&release).send()?;
    debug!("body: {:?}", resp.status());

    Ok(())
}

fn main() {
    let matches = App::new("apis")
        .version("0.1.0")
        .about("a CLI tool to get information on apis-catalog")
        .subcommand(SubCommand::with_name("list")
                    .about("List all available apis")
                    .version("0.1")
                    )
        .subcommand(SubCommand::with_name("endpoints")
                    .about("Give access to list of items")
                    .version("0.1")
                    .arg(Arg::with_name("api")
                        .short("a")
                        .long("api")
                        .takes_value(true)
                        .required(true)
                        .help("List all available endpoints for specified api"))
                    )  
        .subcommand(SubCommand::with_name("releases")
                    .about("List all releases")
                    .version("0.1")
                    .arg(Arg::with_name("api")
                        .short("a")
                        .long("api")
                        .takes_value(true)
                        .required(true)
                        .help("List all available releases for specified api"))
                    )         
        .subcommand(SubCommand::with_name("release")
                    .about("Release an api")
                    .version("0.1")
                    .arg(Arg::with_name("api")
                        .short("a")
                        .long("api")
                        .takes_value(true)
                        .required(true)
                        .help("the api to release"))
                     .arg(Arg::with_name("commit-id")
                        .short("cid")
                        .long("commit-id")
                        .takes_value(true)
                        .required(true)
                        .help("the commit-id of the release"))
                    )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("list") {
        get_apis();
    }  
    if let Some(matches) = matches.subcommand_matches("endpoints") {
        if matches.is_present("api") {
            get_endpoints(matches.value_of("api").unwrap());
        }
    }  
    if let Some(matches) = matches.subcommand_matches("release") {
        if matches.is_present("api") &&  matches.is_present("commit-id"){
            release(matches.value_of("api").unwrap(), matches.value_of("commit-id").unwrap());
        }
    }  
}
