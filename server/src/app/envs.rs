use actix_web::web::Json;
use actix_web::{get, post};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[path = "../dao/mod.rs"]
mod dao;
use dao::repo_envs::*;

use log::{debug, info};

#[path = "../settings/mod.rs"]
mod settings;
use settings::Settings;

use uuid::Uuid;

lazy_static! {
    static ref SETTINGS: settings::Settings = Settings::new().unwrap();
}

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
pub fn create_env(env: Json<Env>) -> HttpResponse {
    info!("create env [{:?}]", env);
    add_env(&SETTINGS.database, &env.name, &env.description).unwrap();

    HttpResponse::Ok().json("")
}

pub fn get_env(path: web::Path<String>) -> HttpResponse {
    let env_id = Uuid::parse_str(&path.0).unwrap();

    let response = match dao::repo_envs::get_env(&SETTINGS.database, env_id) {
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
            debug!("No Env found for api [{:?}]", &path.0);

            HttpResponse::NotFound().finish()
        }
    };

    response
}

#[get("/v1/envs")]
pub fn list_env() -> HttpResponse {
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
