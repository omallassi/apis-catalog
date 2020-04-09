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

#[macro_use]
extern crate lazy_static;

mod settings;
use settings::{*};

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
    let url = format!("http://{address}/v1/endpoints/{api}", address = &SETTINGS.server.address, api = &api);
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
    let url = format!("http://{address}/v1/specs", address = &SETTINGS.server.address);
    let mut resp = client.get(&url).send()?;  
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
    let url = format!("http://{address}/v1/deployments", address = &SETTINGS.server.address);
    let resp = client.post(&url).json(&deployment).send()?;
    debug!("body: {:?}", resp.status());

    Ok(())
}

fn get_deployments(api: Option<&str>) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let mut resp: Response;
    let api_id = match api {
        Some(api_id) => {
            debug!("get deployments for specificied api {:?}", api_id);
            let url = format!("http://{address}/v1/deployments/{api_id}", address = &SETTINGS.server.address, api_id = api_id);
            resp = client.get(&url).send()?;
        },
        None => {
            debug!("get all deployments");
            let url = format!("http://{address}/v1/deployments", address = &SETTINGS.server.address);
            resp = client.get(&url).send()?;
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
    description: String, 
}

#[derive(Serialize, Deserialize, Debug)]
struct Domains {
    domains: Vec<Domain>
}

fn get_domains() -> Result<(), reqwest::Error> {
    let client = Client::new();
    let url = format!("http://{address}/v1/domains", address = &SETTINGS.server.address);
    let mut resp = client.get(&url).send()?;
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

fn create_domain(name: &str, description: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let domain = Domain {
        id: Uuid::nil(),
        name: name.to_string(), 
        description: description.to_string(),
    };
    let url = format!("http://{address}/v1/domains", address = &SETTINGS.server.address);
    let resp = client.post(&url).json(&domain).send()?;
    debug!("body: {:?}", resp.status());

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Api {
    pub id: Uuid,
    pub name: String, 
    pub status: Status,
    pub domain_id: Uuid, 
    pub domain_name: String,
    pub spec_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Status {
    VALIDATED, DEPRECATED, RETIRED,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Apis {
    pub apis: Vec<Api>
}

fn create_api(name: &str, domain_id: &str, specs: Vec<&str> ) -> Result<(), reqwest::Error> {
    debug!("create_api() is called [{:?}], [{:?}], [{:?}]", name, domain_id, specs);

    let specs_as_string: Vec<String> = Vec::from_iter(specs.iter().map(|spec| spec.to_string()));

    let client =  Client::new();

    let api = Api {
        id: Uuid::nil(),
        name: name.to_string(),
        status: Status::VALIDATED,  
        domain_id: Uuid::parse_str(domain_id).unwrap(),
        domain_name: String::from(""),
        spec_ids: specs_as_string,
    };

    let url = format!("http://{address}/v1/apis", address = &SETTINGS.server.address);
    let resp = client.post(&url).json(&api).send()?;
    debug!("body: {:?}", resp.status());
    
    Ok(())
}

fn list_all_apis() -> Result<(), reqwest::Error> {
    let client =  Client::new();

    let url = format!("http://{address}/v1/apis", address = &SETTINGS.server.address);
    let mut resp = client.get(&url).send()?;
    debug!("body: {:?}", resp.status());

    let apis: Apis = resp.json()?;
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row![b -> "Id", b -> "Name", b -> "Domain", b -> "Domain", b -> "Specs"]);
    for api in apis.apis {
        table.add_row(row![api.id, api.name, api.domain_id, api.domain_name, format!("{:?}", api.spec_ids)]);
    };
    
    // Print the table to stdout
    table.printstd();
    
    Ok(())
}

fn update_api_status(api: &str, value: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let url = format!("http://{address}/v1/apis/{id}", address = &SETTINGS.server.address, id = api);

    let status = match value {
        "validated" => Status::VALIDATED,
        "deprecated" => Status::DEPRECATED,
        "retired" => Status::RETIRED,
        _ => panic!("Unable to get Status for {}", value),
    };

    let api = Api {
        id: Uuid::nil(),
        name: String::new(),
        status: Status::VALIDATED,
        domain_id: Uuid::nil(),
        domain_name: String::new(),
        spec_ids: Vec::new(),
    };

    let mut resp = client.patch(&url).json(&api).send()?;
    debug!("body: {:?}", resp.status());

    Ok(())
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Envs {
    pub envs: Vec<Env>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Env {
    pub id: Uuid, 
    pub name: String,
    pub description: String,
}

fn list_env() -> Result<(), reqwest::Error> {
    let client = Client::new();
    let url = format!("http://{address}/v1/envs", address = &SETTINGS.server.address);
    let mut resp = client.get(&url).send()?;
    debug!("body: {:?}", resp.status());
    let envs: Envs = resp.json()?;
    //
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row![b -> "Id", b -> "Env Name", b -> "Description"]);
    for env in envs.envs {
        table.add_row(row![env.id, env.name, env.description]);
    };
    
    // Print the table to stdout
    table.printstd();

    Ok(())
}

fn create_env(name: &str, description: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let env = Env {
        id: Uuid::nil(),
        name: name.to_string(), 
        description: description.to_string(),
    };
    let url = format!("http://{address}/v1/envs", address = &SETTINGS.server.address);
    let resp = client.post(&url).json(&env).send()?;
    debug!("body: {:?}", resp.status());

    Ok(())
}

lazy_static! {
    static ref SETTINGS : settings::Settings = Settings::new().unwrap();
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
                    .arg(Arg::with_name("description")
                        .short("d")
                        .long("description")
                        .takes_value(true)
                        .required(false)
                        .help("Some description, if you want to..."))
                ),
        )
        .subcommand(
            App::new("specs")
                .about("Manage Specifications")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(SubCommand::with_name("list").about("List All the Specs"))
        )
        .subcommand(
            App::new("apis")
                .about("Manage APIs")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(SubCommand::with_name("list")
                    .about("List all available apis")
                    .version("0.1")
                )
                .subcommand(SubCommand::with_name("create")
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
                .subcommand(SubCommand::with_name("status")
                    .about("set the status of the api")
                    .arg(Arg::with_name("value")
                        .short("v")
                        .long("value")
                        .takes_value(true)
                        .possible_values(&["validated", "deprecated", "retired"])
                        .required(true)
                    )
                    .arg(Arg::with_name("api")
                        .short("a")
                        .long("api")
                        .takes_value(true)
                        .required(true)
                )
            )
        )
        .subcommand(SubCommand::with_name("endpoints")
                    .about("Give access to list of items")
                    .version("0.1")
                    .arg(Arg::with_name("spec")
                        .short("s")
                        .long("spec")
                        .takes_value(true)
                        .required(true)
                        .help("List all available endpoints for specified spec"))
                    )                    
        .subcommand(
            App::new("deployments")
                .about("Manage deployments")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List all deployments (for the specified api)")
                        .arg(Arg::with_name("api")
                            .long("api")
                            .takes_value(true)
                            .required(false)
                            .help("The id of the api")
                        )
                )
                .subcommand(
                    SubCommand::with_name("create")
                        .about("Create a new deployment (for the specified api)")
                        .arg(Arg::with_name("api")
                            .long("api")
                            .takes_value(true)
                            .required(false)
                            .help("The id of the api")
                        )
                        .arg(Arg::with_name("env")
                        .short("e")
                        .long("env")
                        .takes_value(true)
                        .required(true)
                        .help("env id"))
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
                            .arg(Arg::with_name("description")
                                .long("description")
                                .takes_value(true)
                                .required(true)
                                .help("A description associated to the env")
                            )
                    )
                )
            // .subcommand(
            //     App::new("xxx - extensions: layers, services etc....").about("DO WE NEED THIS HERE?")
            // )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("endpoints") {
        if matches.is_present("spec") {
            get_endpoints(matches.value_of("spec").unwrap());
        }
    }  

    match matches.subcommand() {
        ("domains", Some(domains_matches)) => match domains_matches.subcommand() {
            ("list", Some(_matches)) => {
                get_domains();
            }
            ("create", Some(matches)) => {
                let description = match matches.value_of("description") {
                    Some(description) => description,
                    None => ""
                };

                create_domain(matches.value_of("name").unwrap(), description);
            }
            _ => unreachable!(),
        }
        ("specs", Some(matches)) => match matches.subcommand() {
            ("list", Some(_matches)) => {
                get_specs();
            }
            // ("deploy", Some(matches)) => {
            //     deploy(matches.value_of("spec-id").unwrap(), matches.value_of("env").unwrap());
            // }

            _ => unreachable!(),
        }
        ("deployments", Some(deployments)) => match deployments.subcommand() {
            ("list", Some(matches)) => {
                get_deployments(matches.value_of("api"));
            }
            ("create", Some(matches)) => {
                deploy(matches.value_of("api").unwrap(), matches.value_of("env").unwrap());
            }

            _ => unreachable!(),
        }
        ("env", Some(deployments)) => match deployments.subcommand() {
            ("list", Some(_matches)) => {
                list_env();
            }
            ("create", Some(matches)) => {
                create_env(matches.value_of("name").unwrap(), matches.value_of("description").unwrap()).unwrap();
            }
            _ => unreachable!(),
        }
        ("apis", Some(deployments)) => match deployments.subcommand() {
            ("create", Some(matches)) => {
                let specs: Vec<_> = matches.values_of("spec-ids").unwrap().collect();
                let name = matches.value_of("name").unwrap();
                let domain_id = matches.value_of("domain-id").unwrap();
    
                create_api(name, domain_id, specs).unwrap();
            }
            ("list", Some(_matches)) => {
                list_all_apis().unwrap();
            }
            ("status", Some(matches)) => {
                update_api_status(matches.value_of("api").unwrap(), matches.value_of("value").unwrap());
            }

            _ => unreachable!(),
        }

        ("", None) => println!("No subcommand was used"), // If no subcommand was usd it'll match the tuple ("", None)
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }
}
