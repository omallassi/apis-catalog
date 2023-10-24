extern crate clap;
use clap::{arg, command, value_parser, ArgAction, Command};

#[macro_use]
extern crate prettytable;
use prettytable::format;
use prettytable::{Table};

extern crate reqwest;
use reqwest::blocking::{Client, Response};
use serde::{Deserialize, Serialize};

extern crate env_logger;
extern crate log;
use log::{debug, info, error};

use std::vec::Vec;

extern crate uuid;
use uuid::Uuid;

use std::iter::FromIterator;

#[macro_use]
extern crate lazy_static;

mod settings;
use settings::*;




fn main() {
    env_logger::init();
    
    let matches = command!() 
        // .subcommand(
        //     Command::new("toto")
        //     .arg(
        //         arg!(
        //             -c --config <FILE> "Sets a custom config file"
        //         )
        //         // We don't have syntax yet for optional options, so manually calling `required`
        //         .required(false)
        //         //.value_parser(value_parser!(PathBuf)),
        //     )
        // )
        .subcommand(
        Command::new("catalogs")
            .about("everything related to catalogs")
            .arg(
                arg!(-l --list "lists all configured catalogs").action(ArgAction::SetTrue)
            )
            .arg(
                arg!(-r --refresh "refresh all configured catalogs").action(ArgAction::SetTrue)
            ),
        )
        .get_matches();


        // if let Some(matches) = matches.subcommand_matches("toto") {
        // if let Some(config_path) = matches.get_one::<String>("config") {
        //     println!("Value for config: {:?}", config_path);
        // }
        // }



    if let Some(matches) = matches.subcommand_matches("catalogs") {
        // "$ myapp test" was run
        if matches.get_flag("list") {
            let _ = list_all_catalogs();
        } 
        if matches.get_flag("refresh") {
            refresh_all_catalogs();
        } 

    }
}



//
#[derive(Deserialize)]
struct Catalog {
    name: String, 
    id: String, 
    http_base_uri: String
}


fn list_all_catalogs() -> Result<(), reqwest::Error> {
    let client = Client::new();
    let url = format!(
        "http://{address}/v1/catalogs",
        address = &SETTINGS.server.address
    );

    let resp = client.get(&url).send()?;
    debug!("body: {:?}", &resp.status());
    let catalogs: Vec<Catalog> = resp.json()?;
    //
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row![b -> "Id", b -> "Name", b -> "Base URI"]);
    for val in catalogs {
        table.add_row(row![val.id, val.name, val.http_base_uri]);
    }

    // Print the table to stdout
    table.printstd();

    Ok(())
}

fn refresh_all_catalogs() -> Result<(), reqwest::Error> {
    let client = Client::new();
    let url = format!(
        "http://{address}/v1/catalogs/refresh",
        address = &SETTINGS.server.address
    );

    let resp = client.post(&url).send()?;
    println!("Refreshed Catalogs with status {:?}", &resp.status());
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
    deployments: Vec<Deployment>,
}

fn deploy(api: &str, env: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let deployment = Deployment {
        api: api.to_string(),
        env: env.to_string(),
    };
    let url = format!(
        "http://{address}/v1/deployments",
        address = &SETTINGS.server.address
    );
    let resp = client.post(&url).json(&deployment).send()?;
    debug!("body: {:?}", resp.status());

    Ok(())
}

fn get_deployments(api: Option<&str>) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let mut resp: Response;
    match api {
        Some(api_id) => {
            debug!("get deployments for specificied api {:?}", api_id);
            let url = format!(
                "http://{address}/v1/deployments/{api_id}",
                address = &SETTINGS.server.address,
                api_id = api_id
            );
            resp = client.get(&url).send()?;
        }
        None => {
            debug!("get all deployments");
            let url = format!(
                "http://{address}/v1/deployments",
                address = &SETTINGS.server.address
            );
            resp = client.get(&url).send()?;
        }
    };

    debug!("body: {:?}", resp.status());
    let deployments: Deployments = resp.json()?;
    //
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row![b -> "Apis", b -> "Env"]);
    for val in deployments.deployments {
        table.add_row(row![val.api, val.env]);
    }

    // Print the table to stdout
    table.printstd();

    Ok(())
}



#[derive(Serialize, Deserialize, Debug)]
pub struct Tier {
    pub name: String,
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
struct Tiers {
    tiers: Vec<Tier>,
}

fn create_tier(name: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let tier = Tier {
        id: Uuid::nil(),
        name: name.to_string(),
    };
    let url = format!(
        "http://{address}/v1/tiers",
        address = &SETTINGS.server.address
    );
    let resp = client.post(&url).json(&tier).send()?;
    debug!("body: {:?}", resp.status());

    Ok(())
}

fn get_tiers() -> Result<(), reqwest::Error> {
    let client = Client::new();
    let url = format!(
        "http://{address}/v1/tiers",
        address = &SETTINGS.server.address
    );
    let mut resp = client.get(&url).send()?;
    debug!("body: {:?}", resp.status());
    let tiers: Tiers = resp.json()?;
    //
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row![b -> "Id", b -> "Name"]);
    for tier in tiers.tiers {
        table.add_row(row![tier.id, tier.name]);
    }

    // Print the table to stdout
    table.printstd();

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Api {
    pub id: Uuid,
    pub name: String,
    pub tier: String,
    pub status: Status,
    pub domain_id: Uuid,
    pub domain_name: String,
    pub spec_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Status {
    VALIDATED,
    DEPRECATED,
    RETIRED,
    NONE,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Apis {
    pub apis: Vec<Api>,
}

fn create_api(name: &str, domain_id: &str, specs: Vec<&str>) -> Result<(), reqwest::Error> {
    debug!(
        "create_api() is called [{:?}], [{:?}], [{:?}]",
        name, domain_id, specs
    );

    let specs_as_string: Vec<String> = Vec::from_iter(specs.iter().map(|spec| spec.to_string()));

    let client = Client::new();

    let api = Api {
        id: Uuid::nil(),
        name: name.to_string(),
        status: Status::NONE,
        domain_id: Uuid::parse_str(domain_id).unwrap(),
        domain_name: String::from(""),
        spec_ids: specs_as_string,
        tier: String::new(),
    };

    let url = format!(
        "http://{address}/v1/apis",
        address = &SETTINGS.server.address
    );
    let resp = client.post(&url).json(&api).send()?;
    debug!("body: {:?}", resp.status());

    Ok(())
}

fn list_all_apis() -> Result<(), reqwest::Error> {
    let client = Client::new();

    let url = format!(
        "http://{address}/v1/apis",
        address = &SETTINGS.server.address
    );
    let mut resp = client.get(&url).send()?;
    debug!("body: {:?}", resp.status());

    let apis: Apis = resp.json()?;
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(
        row![b -> "Id", b -> "Name", b -> "Tier", b -> "Domain", b -> "Domain", b -> "Specs"],
    );
    for api in apis.apis {
        table.add_row(row![
            api.id,
            api.name,
            api.tier,
            api.domain_id,
            api.domain_name,
            format!("{:?}", api.spec_ids)
        ]);
    }

    // Print the table to stdout
    table.printstd();

    Ok(())
}

fn update_api_status(api: &str, value: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let status = match value {
        "validated" => Status::VALIDATED,
        "deprecated" => Status::DEPRECATED,
        "retired" => Status::RETIRED,
        _ => Status::NONE,
    };

    let url = format!(
        "http://{address}/v1/apis/{id}/status",
        address = &SETTINGS.server.address,
        id = api
    );
    //update and send it and updated version back
    let resp = client.post(&url).json(&status).send()?;
    debug!("response: {:?}", resp.status());

    Ok(())
}

fn update_api_tier(api: &str, tier: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let url = format!(
        "http://{address}/v1/apis/{id}/tier",
        address = &SETTINGS.server.address,
        id = api
    );
    //update and send it and updated version back
    let resp = client.post(&url).json(&tier).send()?;
    debug!("response: {:?}", resp.status());

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Envs {
    pub envs: Vec<Env>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Env {
    pub id: Uuid,
    pub name: String,
    pub description: String,
}

fn list_env() -> Result<(), reqwest::Error> {
    let client = Client::new();
    let url = format!(
        "http://{address}/v1/envs",
        address = &SETTINGS.server.address
    );
    let mut resp = client.get(&url).send()?;
    debug!("body: {:?}", resp.status());
    let envs: Envs = resp.json()?;
    //
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row![b -> "Id", b -> "Env Name", b -> "Description"]);
    for env in envs.envs {
        table.add_row(row![env.id, env.name, env.description]);
    }

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
    let url = format!(
        "http://{address}/v1/envs",
        address = &SETTINGS.server.address
    );
    let resp = client.post(&url).json(&env).send()?;
    debug!("body: {:?}", resp.status());

    Ok(())
}

lazy_static! {
    static ref SETTINGS: settings::Settings = Settings::new().unwrap();
}