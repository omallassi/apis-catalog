use actix_web::web::Json;
use actix_web::{get, post};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[path = "../dao/mod.rs"]
mod dao;
use dao::repo_deployments::*;

use log::{debug, error};

#[path = "../settings/mod.rs"]
mod settings;
use settings::Settings;

lazy_static! {
    static ref SETTINGS: settings::Settings = Settings::new().unwrap();
}

/*
 * deployments related APIs
 */

#[derive(Serialize, Deserialize, Debug)]
pub struct Deployment {
    api: String,
    env: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Deployments {
    deployments: Vec<Deployment>,
}

#[post("/v1/deployments")]
pub fn add_deployment(deployment: Json<Deployment>) -> HttpResponse {
    release(
        &SETTINGS.database,
        deployment.api.clone(),
        deployment.env.clone(),
    )
    .unwrap();

    HttpResponse::Ok().json("")
}

#[get("/v1/deployments")]
pub fn get_deployments() -> HttpResponse {
    let mut deployments = Deployments {
        deployments: Vec::new(),
    };

    let mut all_tuples: Vec<(String, String)> = match list_all_deployments(&SETTINGS.database) {
        Ok(all_tuples) => all_tuples,
        Err(why) => {
            debug!("No Deployments found");
            Vec::new()
        }
    };

    while let Some(tuple) = all_tuples.pop() {
        let deployment = Deployment {
            api: tuple.0,
            env: tuple.1,
        };
        deployments.deployments.push(deployment);
    }

    HttpResponse::Ok().json(deployments)
}

pub fn get_deployments_for_api(path: web::Path<(String,)>) -> HttpResponse {
    let mut deployments = Deployments {
        deployments: Vec::new(),
    };

    let mut all_tuples: Vec<(String, String)> =
        match get_all_deployments_for_api(&SETTINGS.database, &path.0) {
            Ok(all_tuples) => all_tuples,
            Err(why) => {
                error!("No Deployments found for api [{:?}] - [{:?}]", &path.0, why);
                Vec::new()
            }
        };

    while let Some(tuple) = all_tuples.pop() {
        let deployment = Deployment {
            api: tuple.0,
            env: tuple.1,
        };
        deployments.deployments.push(deployment);
    }

    HttpResponse::Ok().json(deployments)
}
