extern crate clap;
use clap::{App, AppSettings, Arg, SubCommand};

#[macro_use] extern crate prettytable;
use prettytable::{Table, Row, Cell};
use prettytable::format;

extern crate reqwest;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};

extern crate log;
extern crate env_logger;
use log::{info, debug, warn, error};

use std::vec::Vec;

extern crate uuid;
use uuid::Uuid;

use std::iter::FromIterator;

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
struct Specs {
    specs: Vec<Spec>,
}

#[derive(Serialize, Deserialize)]
struct Spec {
    name: String,
    id: String,
}

fn get_specs() -> Result<(), reqwest::Error> {
    let client = Client::new();
    let mut resp = client.get("http://127.0.0.1:8088/v1/specs").send()?;
    debug!("body: {:?}", resp.status());
    let specs: Specs = resp.json()?;
    //
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row![b -> "Id", b -> "Specs"]);
    for val in specs.specs {
        table.add_row(row![val.id, val.name]);
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

#[derive(Serialize, Deserialize, Debug)]
struct Deployments {
    deployments: Vec<Deployment>
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

fn get_deployments(api: Option<&str>) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let mut resp: Response;
    let api_id = match api {
        Some(api_id) => {
            debug!("get deployments for specificied api {:?}", api_id);
            let url = format!("http://127.0.0.1:8088/v1/deployments/{}", api_id);
            resp = client.get(&url).send()?;
        },
        None => {
            debug!("get all deployments");
            resp = client.get("http://127.0.0.1:8088/v1/deployments").send()?;
        },
    };
    
    debug!("body: {:?}", resp.status());
    let deployments: Deployments = resp.json()?;
    //
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row![b -> "Apis", b -> "Env"]);
    for val in deployments.deployments {
        table.add_row(row![val.api, val.env]);
    };
    
    // Print the table to stdout
    table.printstd();

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Domain {
    name: String,
    id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
struct Domains {
    domains: Vec<Domain>
}

fn get_domains() -> Result<(), reqwest::Error> {
    let client = Client::new();
    let mut resp = client.get("http://127.0.0.1:8088/v1/domains").send()?;
    debug!("body: {:?}", resp.status());
    let domains: Domains = resp.json()?;
    //
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row![b -> "Id", b -> "Domain Name"]);
    for domain in domains.domains {
        table.add_row(row![domain.id, domain.name]);
    };
    
    // Print the table to stdout
    table.printstd();

    Ok(())
}

fn create_domain(name: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let domain = Domain {
        id: Uuid::nil(),
        name: name.to_string(), 
    };
    let resp = client.post("http://127.0.0.1:8088/v1/domains").json(&domain).send()?;
    debug!("body: {:?}", resp.status());

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Api {
    pub name: String, 
    pub domain_id: Uuid, 
    pub spec_ids: Vec<String>,
}

fn create_api(name: &str, domain_id: &str, specs: Vec<&str> ) -> Result<(), reqwest::Error> {
    let specs_as_string: Vec<String> = Vec::from_iter(specs.iter().map(|spec| spec.to_string()));

    let client =  Client::new();

    let api = Api {
        name: name.to_string(),
        domain_id: Uuid::parse_str(domain_id).unwrap(),
        spec_ids: specs_as_string,
    };

    let resp = client.post("http://127.0.0.1:8088/v1/apis").json(&api).send()?;
    debug!("body: {:?}", resp.status());
    
    Ok(())
}


fn main() {
    env_logger::init();

    let matches = App::new("catalog")
        .version("0.1.0")
        .about("a CLI tool to get information on apis-catalog")
        .subcommand(
            App::new("domains")
                .about("Manage Domains")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(SubCommand::with_name("list").about("List All the Domains"))
                .subcommand(SubCommand::with_name("create")
                    .about("Create a new Domain")
                    .arg(Arg::with_name("name")
                        .short("n")
                        .long("name")
                        .takes_value(true)
                        .required(true)
                        .help("The name of the domain"))
                ),
        )
        .subcommand(
            App::new("specs")
                .about("Manage Specifications")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(SubCommand::with_name("list").about("List All the Specs"))
                .subcommand(SubCommand::with_name("deploy")
                    .about("Deploy a spec")
                    .version("0.1")
                    .arg(Arg::with_name("spec-id")
                        .long("spec-id")
                        .takes_value(true)
                        .required(true)
                        .help("the api to deploy"))
                    .arg(Arg::with_name("env")
                        .long("env")
                        .takes_value(true)
                        .required(true)
                        .help("the id of the deployment"))
                )
        )
        .subcommand(
            App::new("apis")
                .about("Manage APIs")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(SubCommand::with_name("list")
                    .about("List all available apis")
                    .version("0.1")
                )
                .subcommand(SubCommand::with_name("new")
                    .about("Create a new API")
                    .arg(Arg::with_name("name")
                            .short("n")
                            .long("name")
                            .takes_value(true)
                            .required(true)
                            .help("The name of the api"))
                    .arg(Arg::with_name("domain-id")
                            .long("domain-id")
                            .takes_value(true)
                            .required(true)
                            .help("The id of the domain the API belongs to"))
                    .arg(Arg::with_name("spec-ids")
                            .long("spec-ids")
                            .takes_value(true)
                            .required(true)
                            .min_values(1)
                            .help("The id(s) of the spec(s) to add to the api"))
                )
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
        .subcommand(
            App::new("deployments")
                .about("List deployments")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List all deployments (for a given api)")
                        .arg(Arg::with_name("api-id")
                            .long("api-id")
                            .takes_value(true)
                            .required(false)
                            .help("The name of the api")
                        )
                )
                .subcommand(
                    SubCommand::with_name("create")
                        .about("Create a new deployment")
                )
            )
            .subcommand(
                App::new("env")
                    .about("Manage environments")
                    .setting(AppSettings::SubcommandRequiredElseHelp)
                    .subcommand(
                        SubCommand::with_name("list")
                            .about("List all env")
                    )
                    .subcommand(
                        SubCommand::with_name("create")
                            .about("Create a new env")
                            .arg(Arg::with_name("name")
                                .long("name")
                                .takes_value(true)
                                .required(true)
                                .help("The name of the env")
                            )
                    )
                )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("endpoints") {
        if matches.is_present("api") {
            get_endpoints(matches.value_of("api").unwrap());
        }
    }  

    match matches.subcommand() {
        ("domains", Some(domains_matches)) => match domains_matches.subcommand() {
            ("list", Some(_matches)) => {
                get_domains();
            }
            ("create", Some(matches)) => {
                create_domain(matches.value_of("name").unwrap());
            }
            _ => unreachable!(),
        }
        ("specs", Some(matches)) => match matches.subcommand() {
            ("list", Some(_matches)) => {
                get_specs();
            }
            ("deploy", Some(matches)) => {
                deploy(matches.value_of("spec-id").unwrap(), matches.value_of("env").unwrap());
            }

            _ => unreachable!(),
        }
        ("deployments", Some(deployments)) => match deployments.subcommand() {
            ("list", Some(matches)) => {
                get_deployments(matches.value_of("api-id"));
            }
            _ => unreachable!(),
        }
        ("env", Some(deployments)) => match deployments.subcommand() {
            ("list", Some(matches)) => {
                println!("To Be Implemented");
            }
            ("create", Some(matches)) => {
                println!("To Be Implemented [{}]", matches.value_of("name").unwrap());
            }
            _ => unreachable!(),
        }
        ("apis", Some(deployments)) => match deployments.subcommand() {
            ("new", Some(matches)) => {
                let specs: Vec<_> = matches.values_of("spec-ids").unwrap().collect();
    
                create_api(matches.value_of("name").unwrap(), 
                    matches.value_of("domain-id").unwrap(), 
                    specs
                ).unwrap();
            }
            ("list", Some(matches)) => {
                println!("not implemented");
            }
            _ => unreachable!(),
        }

        ("", None) => println!("No subcommand was used"), // If no subcommand was usd it'll match the tuple ("", None)
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }
}
