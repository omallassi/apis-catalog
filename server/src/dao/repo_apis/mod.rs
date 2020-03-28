extern crate failure;
extern crate rusqlite;
extern crate time;
extern crate uuid;

use uuid::Uuid;

use rusqlite::{params, Connection, Result};
use rusqlite::{NO_PARAMS, named_params};

//use rustbreak::{FileDatabase, deser::Ron};
use log::{debug, info};

use super::super::settings::{*};

pub struct ApiItem {
    pub name: std::string::String,
    pub id: Uuid,
    pub domain_id: Uuid,
}

pub fn list_all_apis(config:  &super::super::settings::Database) -> Result<Vec<ApiItem>> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-apis.db");
    {
        debug!("Reading all apis from Api_Database [{:?}]", db_path);
    }

    let conn = Connection::open(db_path)?;
    conn.execute("CREATE TABLE IF NOT EXISTS apis (
            id UUID  NOT NULL UNIQUE,
            name TEXT NOT NULL, 
            domain_id UUID NOT NULL
        )", 
        NO_PARAMS,
    )?;

    let mut stmt = conn.prepare("SELECT id, name, domain_id FROM apis")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut tuples = Vec::new();
    while let Some(row) = rows.next()? {
        let id = row.get("id")?;
        let name = row.get("name")?;
        let domain_id = row.get("domain_id")?;
        let domain = ApiItem{
            id: id,
            name: name,
            domain_id: domain_id,
        };

        tuples.push(domain);
    }

    Ok(tuples)
}

pub fn add_api(config:  &super::super::settings::Database, name: &str, domain_id: &Uuid) -> Result<()> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-apis.db");
    {
        debug!("Creating api [{}] into Api_Database [{:?}]", name, db_path);
    }

    let conn = Connection::open(db_path)?;
    conn.execute("CREATE TABLE IF NOT EXISTS apis (
            id UUID  NOT NULL UNIQUE,
            name TEXT NOT NULL, 
            domain_id UUID NOT NULL
        )", 
        NO_PARAMS,
    )?;

    let id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO apis (id, name, domain_id) VALUES (?1, ?2, ?3)",
        params![id, name, domain_id],
    )?;

    conn.close().unwrap();
    
    Ok(())
}

