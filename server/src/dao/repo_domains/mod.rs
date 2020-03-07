extern crate failure;
extern crate rusqlite;
extern crate time;
extern crate uuid;

use uuid::Uuid;

use rusqlite::{params, Connection, Result};
use rusqlite::{NO_PARAMS, named_params};

//use rustbreak::{FileDatabase, deser::Ron};
use log::{debug, info};

pub struct DomainItem{
    pub name: std::string::String,
    pub id: Uuid,
}

pub fn list_all_domains() -> Result<Vec<DomainItem>> {
    debug!("Reading all domains from Domain_Database");
    let conn = Connection::open("/tmp/apis-catalog-domains.db")?;
    conn.execute("CREATE TABLE IF NOT EXISTS domains (
            id UUID  NOT NULL UNIQUE,
            name TEXT NOT NULL
        )", 
        NO_PARAMS,
    )?;

    let mut stmt = conn.prepare("SELECT id, name FROM domains")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut tuples = Vec::new();
    while let Some(row) = rows.next()? {
        let id = row.get("id")?;
        let name = row.get("name")?;
        let domain = DomainItem{
            id: id,
            name: name,
        };

        tuples.push(domain);
    }

    Ok(tuples)
}

pub fn add_domain(name: &str) -> Result<()> {
    debug!("Creating domain [{}] into Domain_Database", name);

    let conn = Connection::open("/tmp/apis-catalog-domains.db")?;
    conn.execute("CREATE TABLE IF NOT EXISTS domains (
            id UUID  NOT NULL UNIQUE,
            name TEXT NOT NULL
        )", 
        NO_PARAMS,
    )?;

    let id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO domains (id, name) VALUES (?1, ?2)",
        params![id, name],
    )?;

    conn.close().unwrap();
    Ok(())
}

pub fn get_domain(id: Uuid) -> Result<DomainItem> {
    debug!("Get domain [{}] into Domain_Database", id);

    let conn = Connection::open("/tmp/apis-catalog-domains.db")?;
    conn.execute("CREATE TABLE IF NOT EXISTS domains (
            id UUID  NOT NULL UNIQUE,
            name TEXT NOT NULL
        )", 
        NO_PARAMS,
    )?;

    let mut stmt = conn.prepare("SELECT id, name FROM domains WHERE id = ?1")?;
    let mut row = stmt.query_row(params![id], |row |{
        Ok(DomainItem {
            name: row.get(1)?,
            id: row.get(0)?,
        })
    })?;

    Ok(row)
}

