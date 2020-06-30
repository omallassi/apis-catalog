extern crate failure;
extern crate rusqlite;
extern crate time;
extern crate uuid;

use rusqlite::{named_params, NO_PARAMS};
use rusqlite::{params, Connection, Result};

//use rustbreak::{FileDatabase, deser::Ron};
use log::debug;

pub fn release(config: &super::super::settings::Database, api: String, env: String) -> Result<()> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!(
            "Releasing [{}] to env [{}] from Deployments_Database [{:?}]",
            api, env, db_path
        );
    }

    let conn = Connection::open(db_path)?;
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS deployments (
    //         api TEXT NOT NULL,
    //         env TEXT NOT NULL
    //     )",
    //     NO_PARAMS,
    // )?;

    debug!("Writing to Database");
    conn.execute(
        "INSERT INTO deployments (api, env)
                  VALUES (?1, ?2)",
        params![api, env],
    )?;

    conn.close().unwrap();

    Ok(())
}

pub fn list_all_deployments(
    config: &super::super::settings::Database,
) -> Result<Vec<(String, String)>> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!(
            "Reading all deployments from Deployments_Database [{:?}]",
            db_path
        );
    }

    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT api, env FROM deployments")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut tuples = Vec::new();
    while let Some(row) = rows.next()? {
        let api: String = row.get(0)?;
        let env: String = row.get(1)?;
        let value = (api, env);

        tuples.push(value);
    }

    Ok(tuples)
}

pub fn get_all_deployments_for_api(
    config: &super::super::settings::Database,
    api: &str,
) -> Result<Vec<(String, String)>> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!(
            "Reading all deployments for api [{}] from Deployments_Database [{:?}]",
            api, db_path
        );
    }

    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT api, env FROM deployments WHERE api = :api")?;
    let mut rows = stmt.query_named(named_params! { ":api": api })?;

    let mut tuples = Vec::new();
    while let Some(row) = rows.next()? {
        let api: String = row.get(0)?;
        let env: String = row.get(1)?;
        let value = (api, env);

        tuples.push(value);
    }

    Ok(tuples)
}
