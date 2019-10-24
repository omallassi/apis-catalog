extern crate failure;
extern crate rusqlite;
extern crate time;

use rusqlite::{params, Connection, Result};
use rusqlite::NO_PARAMS;
use time::Timespec;

//use rustbreak::{FileDatabase, deser::Ron};
use log::{info, debug};
use std::collections::HashMap;

pub fn release(api: String, env: String) -> Result<()> {
    info!("hop {:?}", api);

    //let db = FileDatabase::<HashMap<String, String>, Ron>::from_path("/tmp/apis-catalog-store.ron", HashMap::new())?;
    //db.load()?;

    
    //db.write(|db| {
    //    db.insert(api, env);
    //});
    //db.save().unwrap();
    debug!("Creating Schema");
    let conn = Connection::open("/tmp/apis-catalog-store.db")?;
    conn.execute("CREATE TABLE IF NOT EXISTS deployments (
            api TEXT NOT NULL,
            env TEXT NOT NULL
        )", 
        NO_PARAMS,
    )?;

    debug!("Writing to Database");
    conn.execute(
        "INSERT INTO deployments (api, env)
                  VALUES (?1, ?2)",
        params![api, env],
    )?;

    conn.close().unwrap();

    Ok(())
}

pub fn list_all_deployments() -> Result<Vec<(String, String)>> {
    //let db = FileDatabase::<HashMap<String, String>, Ron>::from_path("/tmp/apis-catalog-store.ron", HashMap::new())?;
    //db.load().unwrap();

    //db.read(|db| {
    //    println!("Results:");
    //    println!("{:#?}", db);
    //})?;

    debug!("Reading from Database");
    let conn = Connection::open("/tmp/apis-catalog-store.db")?;
    let mut stmt = conn.prepare("SELECT api, env FROM deployments")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut names = Vec::new();
    while let Some(row) = rows.next()? {
        let api: String = row.get(0)?;
        let env: String = row.get(1)?;
        let value = (api, env);

        println!("deployment {:?}", value);
        names.push(value);
    }

    Ok(names)
}