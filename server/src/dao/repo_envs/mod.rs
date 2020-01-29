extern crate failure;
extern crate rusqlite;
extern crate time;
extern crate uuid;

use uuid::Uuid;

use rusqlite::{params, Connection, Result};
use rusqlite::{NO_PARAMS, named_params};

//use rustbreak::{FileDatabase, deser::Ron};
use log::{debug, info};

pub struct EnvItem{
    pub id: Uuid, 
    pub name: String,
    pub description: String,
}

pub fn list_all_envs() -> Result<Vec<EnvItem>> {
    debug!("Reading all envs from Env_Database");
    let conn = Connection::open("/tmp/apis-catalog-envs.db")?;
    conn.execute("CREATE TABLE IF NOT EXISTS envs (
            id UUID  NOT NULL UNIQUE,
            name TEXT NOT NULL UNIQUE,
            description TEXT NOT NULL)", 
        NO_PARAMS,
    )?;

    let mut stmt = conn.prepare("SELECT id, name, description FROM envs")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut tuples = Vec::new();
    while let Some(row) = rows.next()? {
        let id = row.get("id")?;
        let name = row.get("name")?;
        let description = row.get("description")?;
        let env = EnvItem{
            id: id,
            name: name,
            description: description,
        };

        tuples.push(env);
    }

    Ok(tuples)
}

pub fn add_env(name: &str, description: &str) -> Result<()> {
    debug!("Creating env [{}] [{}] into Env_Database", name, description);

    let conn = Connection::open("/tmp/apis-catalog-envs.db")?;
    conn.execute("CREATE TABLE IF NOT EXISTS envs (
            id UUID  NOT NULL UNIQUE,
            name TEXT NOT NULL UNIQUE,
            description TEXT NOT NULL)", 
        NO_PARAMS,
    )?;

    let id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO envs (id, name, description) VALUES (?1, ?2, ?3)",
        params![id, name, description],
    )?;

    conn.close().unwrap();
    
    Ok(())
}

