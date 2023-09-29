use actix_web::web::Json;
use actix_web::{get, post, Responder};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

use crate::app::dao::repo_apis::*;
use crate::shared::settings::*;

use log:: info;

use uuid::Uuid;


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
pub async fn get_tiers() -> impl Responder {
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
pub async fn create_tier(tier: Json<Tier>) -> impl Responder {
    let uuid = add_tier(&SETTINGS.database, &tier.name).unwrap();

    HttpResponse::Created()
        .header("Location", format!("/v1/tiers/{}", uuid))
        .finish()
}
