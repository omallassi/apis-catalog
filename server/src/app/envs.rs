use actix_web::web::Json;
use actix_web::{get, post, Responder};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::app::dao::repo_envs::*;
use crate::shared::settings::*;

use log::{debug, info};

use uuid::Uuid;


/*
 * Envs related APIs
 */

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

#[post("/v1/envs")]
pub async fn create_env(env: Json<Env>) -> impl Responder {
    info!("create env [{:?}]", env);
    add_env(&SETTINGS.database, &env.name, &env.description).unwrap();

    HttpResponse::Ok().json("")
}

pub async fn get_env(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    let env_id = Uuid::parse_str(&id).unwrap();

    let response = match crate::app::dao::repo_envs::get_env(&SETTINGS.database, env_id) {
        Ok(env) => {
            let returned_env = Env {
                id: env.id,
                name: env.name,
                description: env.description,
            };
            debug!("Got Env [{:?}]", returned_env);

            HttpResponse::Ok().json(returned_env)
        }
        Err(_) => {
            debug!("No Env found for api [{:?}]", &id);

            HttpResponse::NotFound().finish()
        }
    };

    response
}

#[get("/v1/envs")]
pub async fn list_env() -> impl Responder {
    info!("list envs");

    let mut envs = Envs { envs: Vec::new() };

    let mut all_tuples: Vec<EnvItem> = match list_all_envs(&SETTINGS.database) {
        Ok(all_tuples) => all_tuples,
        Err(why) => {
            debug!("No env found [{:?}]", why);
            Vec::new()
        }
    };

    while let Some(tuple) = all_tuples.pop() {
        let env = Env {
            id: tuple.id,
            name: tuple.name,
            description: tuple.description,
        };
        envs.envs.push(env);
    }

    HttpResponse::Ok().json(envs)
}
