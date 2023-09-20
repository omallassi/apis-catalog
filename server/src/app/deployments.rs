use actix_web::web::Json;
use actix_web::{get, post, Responder};
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
pub async fn add_deployment(deployment: Json<Deployment>) -> impl Responder {
    release(
        &SETTINGS.database,
        deployment.api.clone(),
        deployment.env.clone(),
    )
    .unwrap();

    HttpResponse::Ok().json("")
}

#[get("/v1/deployments")]
pub async fn get_deployments() -> impl Responder {
    let mut deployments = Deployments {
        deployments: Vec::new(),
    };

    let mut all_tuples: Vec<(String, String)> = match list_all_deployments(&SETTINGS.database) {
        Ok(all_tuples) => all_tuples,
        Err(why) => {
            debug!("No Deployments found - {:?}", why);
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

pub async fn get_deployments_for_api(path: web::Path<String>) -> impl Responder {
    let api = path.into_inner();
    let mut deployments = Deployments {
        deployments: Vec::new(),
    };

    let mut all_tuples: Vec<(String, String)> =
        match get_all_deployments_for_api(&SETTINGS.database, &api) {
            Ok(all_tuples) => all_tuples,
            Err(why) => {
                error!("No Deployments found for api [{:?}] - [{:?}]", &api, why);
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
