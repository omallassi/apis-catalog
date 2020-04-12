extern crate failure;
extern crate rusqlite;
extern crate time;
extern crate uuid;

use uuid::Uuid;
use chrono::{Utc};

use rusqlite::{params, Connection, Result};
use rusqlite::{NO_PARAMS};

//use rustbreak::{FileDatabase, deser::Ron};
use log::{debug, info, warn};

use std::sync::Once;

#[derive(Debug)]
pub struct ApiItem {
    pub name: std::string::String,
    pub id: Uuid,
    pub domain_id: Uuid,
    pub status: String, //TODO use the enum
}

#[derive(Debug)]
struct StatusItem {
    api_id: Uuid, 
    status: String
}

static INIT_DB : Once = Once::new();

fn get_init_db(rusqlite: &String) -> Result<String> {
    let mut db_path = String::from(rusqlite);
    db_path.push_str("/apis-catalog-apis.db");

    INIT_DB.call_once( || {
        {debug!("Init Api_Database [{:?}]", db_path);}
        
        let conn = Connection::open(&db_path).unwrap();
        conn.execute("CREATE TABLE IF NOT EXISTS apis (
                id UUID  NOT NULL UNIQUE,
                name TEXT NOT NULL, 
                domain_id UUID NOT NULL
            )", 
            NO_PARAMS,
        ).unwrap();
        //
        conn.execute("CREATE TABLE IF NOT EXISTS status(
                api_id UUID NOT NULL,
                status TEXT NOT NULL,
                start_date_time TEXT NOT NULL, 
                end_date_time TEXT
            )",
            NO_PARAMS
        ).unwrap();
    });
    // debug!("Api_Database [{:?}] already initialized", db_path);

    Ok(String::from(&db_path))
}

pub fn list_all_apis(config:  &super::super::settings::Database) -> Result<Vec<ApiItem>> {
    let db_path = get_init_db(&config.rusqlite_path).unwrap();
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT id, name, domain_id FROM apis")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut tuples = Vec::new();
    while let Some(row) = rows.next()? {
        let id = row.get("id")?;
        let name = row.get("name")?;
        let domain_id = row.get("domain_id")?;
        //get last status
        let status = match get_last_status(config, id) {
            Ok(val) => {
                val.status
            },
            Err(why) => {
                warn!("Unable to get status for api [{:?}] - [{:?}]", id, why);
                String::from("NONE")    //TODO - reuse enum
            }
        };
        //
        let domain = ApiItem{
            id: id,
            name: name,
            domain_id: domain_id,
            status: status,
        };

        tuples.push(domain);
    }

    Ok(tuples)
}

fn get_last_status(config:  &super::super::settings::Database, api_id: Uuid) -> Result<StatusItem> {
    let db_path = get_init_db(&config.rusqlite_path).unwrap();
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT api_id, status FROM status WHERE api_id = ?1")?; //ORDER BY end_date_time DESC limit 1 //start_date_time, end_date_time
    let row = stmt.query_row(params![api_id], |row |{
        Ok(StatusItem {
            api_id: row.get(0)?,
            status: row.get(1)?,
        })
    })?;

    Ok(row)
}

pub fn add_api(config:  &super::super::settings::Database, name: &str, domain_id: &Uuid) -> Result<()> {
    let db_path = get_init_db(&config.rusqlite_path).unwrap();
    let conn = Connection::open(db_path)?;

    let id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO apis (id, name, domain_id) VALUES (?1, ?2, ?3)",
        params![id, name, domain_id],
    )?;

    //TODO manage status

    conn.close().unwrap();
    
    Ok(())
}

pub fn get_api_by_id(config:  &super::super::settings::Database, api: Uuid) -> Result<ApiItem> {
    let db_path = get_init_db(&config.rusqlite_path).unwrap();
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT id, name, domain_id FROM apis WHERE id = ?1")?;
    let row = stmt.query_row(params![api], |row |{
        let id = row.get(0)?;
        //get last status
        let status = match get_last_status(config, id) {
            Ok(val) => {
                val.status
            },
            Err(why) => {
                warn!("Unable to get status for api [{:?}] - [{:?}]", id, why);
                String::from("NONE")    //TODO - reuse enum
            }
        };

        Ok(ApiItem {
            name: row.get(1)?,
            id: id,
            domain_id: row.get(2)?,
            status: status,
        })
    })?;

    Ok(row)
}

//TODO put StatusItem as input + api_id
pub fn update_api_status(config : &super::super::settings::Database, api: ApiItem) -> Result<()> {
    let db_path = get_init_db(&config.rusqlite_path).unwrap();
    let conn = Connection::open(db_path)?;


    //At this stage, start_date_time / end_date_time is not managed so we can delete then insert
    conn.execute(
        "DELETE FROM status WHERE api_id = ?1",
        params![api.id],
    )?;

    conn.execute(
        "INSERT INTO status (api_id, status, start_date_time) VALUES (?1, ?2, ?3)",
        params![api.id, api.status.to_uppercase(), Utc::now()],
    )?;

    conn.close().unwrap();

    Ok(())
}