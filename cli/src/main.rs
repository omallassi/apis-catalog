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

fn get_endpoints(api: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let url = format!("http://127.0.0.1:8088/v1/endpoints/{api}" , api = &api);
    let mut resp = client.get(&url).send()?;
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
    id: String,
}

fn get_apis() -> Result<(), reqwest::Error> {
    let client = Client::new();
    let mut resp = client.get("http://127.0.0.1:8088/v1/apis").send()?;
    debug!("body: {:?}", resp.status());
    let apis: Apis = resp.json()?;
    //
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row![b -> "Id", b -> "Apis", b -> "Deployed Version"]);
    for val in apis.apis {
        table.add_row(row![val.id, val.name, "..."]);
    };
    
    // Print the table to stdout
    table.printstd();

    Ok(())
}

//
#[derive(Serialize, Deserialize, Debug)]
struct Deployment {
    api: String, 
    env: String,
}

fn deploy(api: &str, env: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let deployment = Deployment {
        api: api.to_string(), 
        env: env.to_string(),
    };
    let resp = client.post("http://127.0.0.1:8088/v1/deployments").json(&deployment).send()?;
    debug!("body: {:?}", resp.status());

    Ok(())
}

fn get_deployments() -> Result<(), reqwest::Error> {
    let client = Client::new();

    let resp = client.get("http://127.0.0.1:8088/v1/deployments").send()?;
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
        .subcommand(SubCommand::with_name("deployments")
                    .about("List all deployments")
                    .version("0.1")
                    )         
        .subcommand(SubCommand::with_name("deploy")
                    .about("Deploy an api")
                    .version("0.1")
                    .arg(Arg::with_name("api")
                        .short("a")
                        .long("api")
                        .takes_value(true)
                        .required(true)
                        .help("the api to deploy"))
                     .arg(Arg::with_name("env")
                        .short("e")
                        .long("env")
                        .takes_value(true)
                        .required(true)
                        .help("the env of the deployment"))
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
    if let Some(matches) = matches.subcommand_matches("deploy") {
        if matches.is_present("api") &&  matches.is_present("env"){
            deploy(matches.value_of("api").unwrap(), matches.value_of("env").unwrap());
        }
    }  
    if let Some(matches) = matches.subcommand_matches("deployments") {
        get_deployments();
    }  
}
