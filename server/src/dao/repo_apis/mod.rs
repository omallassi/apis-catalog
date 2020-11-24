extern crate failure;
extern crate rusqlite;
extern crate time;
extern crate uuid;

use chrono::Utc;
use uuid::Uuid;

use rusqlite::NO_PARAMS;
use rusqlite::{params, Connection, Result};

use log::{info, warn};

use std::sync::Once;

#[derive(Debug)]
pub struct ApiItem {
    pub name: std::string::String,
    pub id: Uuid,
    pub domain_id: Uuid,
    pub status: String, //TODO use the enum
    pub tier: TierItem,
}

#[derive(Debug)]
pub struct TierItem {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug)]
pub struct StatusItem {
    pub api_id: Uuid,
    pub status: String,
}

static INIT_DB: Once = Once::new();

fn get_init_db(rusqlite: &String) -> Result<String> {
    let mut db_path = String::from(rusqlite);
    db_path.push_str("/apis-catalog-all.db");

    // INIT_DB.call_once(|| {
    //     {
    //         debug!("Init Api_Database [{:?}]", db_path);
    //     }

    //     let conn = Connection::open(&db_path).unwrap();
    //     conn.execute(
    //         "CREATE TABLE IF NOT EXISTS apis (
    //             id UUID  NOT NULL UNIQUE,
    //             name TEXT NOT NULL,
    //             domain_id UUID NOT NULL,
    //             tier_id UUID NOT NULL
    //         )",
    //         NO_PARAMS,
    //     )
    //     .unwrap();
    //     //
    //     conn.execute(
    //         "CREATE TABLE IF NOT EXISTS status(
    //             api_id UUID NOT NULL,
    //             status TEXT NOT NULL,
    //             start_date_time TEXT NOT NULL,
    //             end_date_time TEXT
    //         )",
    //         NO_PARAMS,
    //     )
    //     .unwrap();
    //     //
    //     conn.execute(
    //         "CREATE TABLE IF NOT EXISTS tiers (
    //             id UUID NOT NULL,
    //             name TEXT NOT NULL
    //         )",
    //         NO_PARAMS,
    //     )
    //     .unwrap();
    // });
    info!(
        "Api_Database [{:?}] already TO BE CHANGED initialized",
        db_path
    );

    Ok(String::from(&db_path))
}

pub fn list_all_apis(config: &super::super::settings::Database) -> Result<Vec<ApiItem>> {
    let db_path = get_init_db(&config.rusqlite_path).unwrap();
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT id, name, domain_id, tier_id FROM apis")?;
    let mut rows = stmt.query(NO_PARAMS)?;
    let mut tuples = Vec::new();
    while let Some(row) = rows.next()? {
        let id = row.get("id")?;
        let name = row.get("name")?;
        let domain_id = row.get("domain_id")?;
        let tier_id = row.get("tier_id")?;

        //get last status
        let status = match get_last_status(config, id) {
            Ok(val) => val.status,
            Err(why) => {
                warn!("Unable to get status for api [{:?}] - [{:?}]", id, why);
                String::from("NONE") //TODO - reuse enum
            }
        };
        let tier = match get_related_tier(config, tier_id) {
            Ok(val) => val,
            Err(why) => {
                warn!("Unable to get tier for api [{:?}] - [{:?}]", id, why);
                TierItem {
                    id: Uuid::nil(),
                    name: String::from("N/A"),
                }
            }
        };
        //
        let domain = ApiItem {
            id: id,
            name: name,
            domain_id: domain_id,
            status: status,
            tier: tier,
        };

        tuples.push(domain);
    }

    Ok(tuples)
}

fn get_related_tier(config: &super::super::settings::Database, tier_id: Uuid) -> Result<TierItem> {
    let db_path = get_init_db(&config.rusqlite_path).unwrap();
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT id, name FROM tiers WHERE id = ?1")?; //ORDER BY end_date_time DESC limit 1 //start_date_time, end_date_time
    let row = stmt.query_row(params![tier_id], |row| {
        Ok(TierItem {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;

    Ok(row)
}

fn get_last_status(config: &super::super::settings::Database, api_id: Uuid) -> Result<StatusItem> {
    let db_path = get_init_db(&config.rusqlite_path).unwrap();
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT api_id, status FROM status WHERE api_id = ?1")?; //ORDER BY end_date_time DESC limit 1 //start_date_time, end_date_time
    let row = stmt.query_row(params![api_id], |row| {
        Ok(StatusItem {
            api_id: row.get(0)?,
            status: row.get(1)?,
        })
    })?;

    Ok(row)
}

pub fn add_api(
    config: &super::super::settings::Database,
    name: &str,
    domain_id: &Uuid,
) -> Result<()> {
    let db_path = get_init_db(&config.rusqlite_path).unwrap();
    let conn = Connection::open(db_path)?;

    let id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO apis (id, name, domain_id, tier_id) VALUES (?1, ?2, ?3, ?4)",
        params![id, name, domain_id, Uuid::nil()],
    )?;

    //TODO manage status

    conn.close().unwrap();

    Ok(())
}

pub fn get_api_by_id(config: &super::super::settings::Database, api: Uuid) -> Result<ApiItem> {
    let db_path = get_init_db(&config.rusqlite_path).unwrap();
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT id, name, domain_id, tier_id FROM apis WHERE id = ?1")?;
    let row = stmt.query_row(params![api], |row| {
        let id = row.get(0)?;
        let tier_id = row.get(3)?;
        //get last status
        let status = match get_last_status(config, id) {
            Ok(val) => val.status,
            Err(why) => {
                warn!("Unable to get status for api [{:?}] - [{:?}]", id, why);
                String::from("NONE") //TODO - reuse enum
            }
        };
        let tier = match get_related_tier(config, tier_id) {
            Ok(val) => val,
            Err(why) => {
                warn!("Unable to get tier for api [{:?}] - [{:?}]", id, why);
                TierItem {
                    id: Uuid::nil(),
                    name: String::from("N/A"),
                }
            }
        };

        Ok(ApiItem {
            name: row.get(1)?,
            id: id,
            tier: tier,
            domain_id: row.get(2)?,
            status: status,
        })
    })?;

    Ok(row)
}

pub fn get_apis_per_domain_id(
    config: &super::super::settings::Database,
    domain_id: Uuid,
) -> Result<Vec<ApiItem>> {
    let db_path = get_init_db(&config.rusqlite_path).unwrap();
    let conn = Connection::open(db_path)?;

    let mut stmt =
        conn.prepare("SELECT id, name, domain_id, tier_id FROM apis WHERE domain_id = ?1")?;
    let mut rows = stmt.query(params![domain_id])?;
    let mut results = Vec::new();
    //TODO O(2N+1)
    while let Some(row) = rows.next()? {
        let id = row.get(0)?;
        let tier_id = row.get(3)?;
        //get last status
        let status = match get_last_status(config, id) {
            Ok(val) => val.status,
            Err(why) => {
                warn!("Unable to get status for api [{:?}] - [{:?}]", id, why);
                String::from("NONE") //TODO - reuse enum
            }
        };
        let tier = match get_related_tier(config, tier_id) {
            Ok(val) => val,
            Err(why) => {
                warn!("Unable to get tier for api [{:?}] - [{:?}]", id, why);
                TierItem {
                    id: Uuid::nil(),
                    name: String::from("N/A"),
                }
            }
        };

        results.push(ApiItem {
            name: row.get(1)?,
            id: id,
            tier: tier,
            domain_id: row.get(2)?,
            status: status,
        });
    }

    Ok(results)
}

pub fn update_api_status(
    config: &super::super::settings::Database,
    status: StatusItem,
) -> Result<()> {
    let db_path = get_init_db(&config.rusqlite_path).unwrap();
    let conn = Connection::open(db_path)?;

    //At this stage, start_date_time / end_date_time is not managed so we can delete then insert
    conn.execute(
        "DELETE FROM status WHERE api_id = ?1",
        params![status.api_id],
    )?;

    conn.execute(
        "INSERT INTO status (api_id, status, start_date_time) VALUES (?1, ?2, ?3)",
        params![status.api_id, status.status.to_uppercase(), Utc::now()],
    )?;

    conn.close().unwrap();

    Ok(())
}

pub fn update_api_tier(
    config: &super::super::settings::Database,
    api_id: Uuid,
    tier_id: Uuid,
) -> Result<()> {
    let db_path = get_init_db(&config.rusqlite_path).unwrap();
    let conn = Connection::open(db_path)?;

    //At this stage, start_date_time / end_date_time is not managed so we can delete then insert
    conn.execute(
        "UPDATE apis SET tier_id = ?1 WHERE id = ?2",
        params![tier_id, api_id],
    )?;

    conn.close().unwrap();

    Ok(())
}

pub fn add_tier(config: &super::super::settings::Database, name: &str) -> Result<Uuid> {
    let db_path = get_init_db(&config.rusqlite_path).unwrap();
    let conn = Connection::open(db_path)?;

    let id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO tiers (id, name) VALUES (?1, ?2)",
        params![id, name],
    )?;

    conn.close().unwrap();

    Ok(id)
}

pub fn list_all_tiers(config: &super::super::settings::Database) -> Result<Vec<TierItem>> {
    let db_path = get_init_db(&config.rusqlite_path).unwrap();
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT id, name FROM tiers")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut tuples = Vec::new();
    while let Some(row) = rows.next()? {
        let id = row.get("id")?;
        let name = row.get("name")?;
        //
        let tier = TierItem { id: id, name: name };

        tuples.push(tier);
    }

    Ok(tuples)
}
