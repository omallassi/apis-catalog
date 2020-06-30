extern crate failure;
extern crate rusqlite;
extern crate time;
extern crate uuid;

use uuid::Uuid;

use rusqlite::NO_PARAMS;
use rusqlite::{params, Connection, Result};

//use rustbreak::{FileDatabase, deser::Ron};
use log::debug;

pub struct EnvItem {
    pub id: Uuid,
    pub name: String,
    pub description: String,
}

pub fn list_all_envs(config: &super::super::settings::Database) -> Result<Vec<EnvItem>> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!("Reading all envs from Env_Database [{:?}]", db_path);
    }

    let conn = Connection::open(db_path)?;
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS envs (
    //         id UUID  NOT NULL UNIQUE,
    //         name TEXT NOT NULL UNIQUE,
    //         description TEXT NOT NULL)",
    //     NO_PARAMS,
    // )?;

    let mut stmt = conn.prepare("SELECT id, name, description FROM envs")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut tuples = Vec::new();
    while let Some(row) = rows.next()? {
        let id = row.get("id")?;
        let name = row.get("name")?;
        let description = row.get("description")?;
        let env = EnvItem {
            id: id,
            name: name,
            description: description,
        };

        tuples.push(env);
    }

    Ok(tuples)
}

pub fn get_env(config: &super::super::settings::Database, id: Uuid) -> Result<EnvItem> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!("Reading envs [{:?}] from Env_Database [{:?}]", id, db_path);
    }

    let conn = Connection::open(db_path)?;
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS envs (
    //         id UUID  NOT NULL UNIQUE,
    //         name TEXT NOT NULL UNIQUE,
    //         description TEXT NOT NULL)",
    //     NO_PARAMS,
    // )?;

    let mut stmt = conn.prepare("SELECT id, name, description FROM envs WHERE id = ?1")?;
    let row = stmt.query_row(params![id], |row| {
        Ok(EnvItem {
            name: row.get(1)?,
            id: row.get(0)?,
            description: row.get(2)?,
        })
    })?;

    Ok(row)
}

pub fn add_env(
    config: &super::super::settings::Database,
    name: &str,
    description: &str,
) -> Result<()> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!("Creating env [{}] into Env_Database [{:?}]", name, db_path);
    }

    let conn = Connection::open(db_path)?;
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS envs (
    //         id UUID  NOT NULL UNIQUE,
    //         name TEXT NOT NULL UNIQUE,
    //         description TEXT NOT NULL)",
    //     NO_PARAMS,
    // )?;

    let id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO envs (id, name, description) VALUES (?1, ?2, ?3)",
        params![id, name, description],
    )?;

    conn.close().unwrap();

    Ok(())
}
