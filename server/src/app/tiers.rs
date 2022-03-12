use actix_web::web::Json;
use actix_web::{get, post};
use actix_web::{HttpResponse};
use serde::{Deserialize, Serialize};

#[path = "../dao/mod.rs"]
mod dao;
use dao::repo_apis::*;

use log::{info};

#[path = "../settings/mod.rs"]
mod settings;
use settings::Settings;

use uuid::Uuid;

lazy_static! {
    static ref SETTINGS: settings::Settings = Settings::new().unwrap();
}

/*
 * Tier(s) related APIs
 */

#[derive(Serialize, Deserialize, Debug)]
pub struct Tier {
    pub name: String,
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tiers {
    pub tiers: Vec<Tier>,
}

#[get("/v1/tiers")]
pub fn get_tiers() -> HttpResponse {
    info!("get tiers");
    let mut all_tiers: Vec<TierItem> = match list_all_tiers(&SETTINGS.database) {
        Ok(all_tiers) => all_tiers,
        Err(why) => {
            panic!("Unable to get domains: {}", why);
        }
    };

    let mut tiers = Vec::new();

    while let Some(tier) = all_tiers.pop() {
        let tier = Tier {
            name: tier.name,
            id: tier.id,
        };
        tiers.push(tier);
    }

    let tiers_obj = Tiers { tiers: tiers };

    HttpResponse::Ok().json(tiers_obj)
}

#[post("/v1/tiers")]
pub fn create_tier(tier: Json<Tier>) -> HttpResponse {
    let uuid = add_tier(&SETTINGS.database, &tier.name).unwrap();

    HttpResponse::Created()
        .header("Location", format!("/v1/tiers/{}", uuid))
        .finish()
}
