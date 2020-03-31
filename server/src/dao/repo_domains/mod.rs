extern crate failure;
extern crate rusqlite;
extern crate time;
extern crate uuid;

use uuid::Uuid;

use rusqlite::{params, Connection, Result};
use rusqlite::{NO_PARAMS};

//use rustbreak::{FileDatabase, deser::Ron};
use log::{debug};

use super::super::settings::{*};

pub struct DomainItem{
    pub name: std::string::String,
    pub id: Uuid,
    pub description: String,
}

pub fn list_all_domains(config:  &super::super::settings::Database) -> Result<Vec<DomainItem>> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-domains.db");
    {
        debug!("Reading all domains from Domain_Database [{:?}]", db_path);
    }

    let conn = Connection::open(db_path)?;
    conn.execute("CREATE TABLE IF NOT EXISTS domains (
            id UUID  NOT NULL UNIQUE,
            name TEXT NOT NULL, 
            description TEXT
        )", 
        NO_PARAMS,
    )?;

    let mut stmt = conn.prepare("SELECT id, name, description FROM domains")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut tuples = Vec::new();
    while let Some(row) = rows.next()? {
        let id = row.get("id")?;
        let name = row.get("name")?;
        let descripton = row.get("description")?;
        let domain = DomainItem{
            id: id,
            name: name,
            description: descripton,
        };

        tuples.push(domain);
    }

    Ok(tuples)
}

pub fn add_domain(config:  &super::super::settings::Database, name: &str, description: &str) -> Result<Uuid> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-domains.db");
    {
        debug!("Creating domain [{}] into Domain_Database [{:?}]", name, db_path);
    }

    let conn = Connection::open(db_path)?;
    conn.execute("CREATE TABLE IF NOT EXISTS domains (
            id UUID  NOT NULL UNIQUE,
            name TEXT NOT NULL,
            description TEXT
        )", 
        NO_PARAMS,
    )?;

    let id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO domains (id, name, description) VALUES (?1, ?2, ?3)",
        params![id, name, description],
    )?;

    conn.close().unwrap();
    Ok(id)
}

pub fn get_domain(config:  &super::super::settings::Database, id: Uuid) -> Result<DomainItem> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-domains.db");
    {
        debug!("Get domain [{}] into Domain_Database [{:?}]", id, db_path);
    }

    let conn = Connection::open(db_path)?;
    conn.execute("CREATE TABLE IF NOT EXISTS domains (
            id UUID  NOT NULL UNIQUE,
            name TEXT NOT NULL,
            description TEXT
        )", 
        NO_PARAMS,
    )?;

    let mut stmt = conn.prepare("SELECT id, name, description FROM domains WHERE id = ?1")?;
    let mut row = stmt.query_row(params![id], |row |{
        Ok(DomainItem {
            name: row.get(1)?,
            id: row.get(0)?,
            description: row.get(2)?,
        })
    })?;

    Ok(row)
}

