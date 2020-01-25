extern crate failure;
extern crate rusqlite;
extern crate time;
extern crate uuid;

use uuid::Uuid;

use rusqlite::{params, Connection, Result};
use rusqlite::{NO_PARAMS, named_params};

//use rustbreak::{FileDatabase, deser::Ron};
use log::{info, debug};

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

    let mut tuples = Vec::new();
    while let Some(row) = rows.next()? {
        let api: String = row.get(0)?;
        let env: String = row.get(1)?;
        let value = (api, env);

        tuples.push(value);
    }

    Ok(tuples)
}

pub fn get_all_deployments_for_api(api: &str) -> Result<Vec<(String, String)>> {
    debug!("Reading from Database");
    let conn = Connection::open("/tmp/apis-catalog-store.db")?;
    let mut stmt = conn.prepare("SELECT api, env FROM deployments WHERE api = :api")?;
    let mut rows = stmt.query_named(named_params!{ ":api": api })?;

    let mut tuples = Vec::new();
    while let Some(row) = rows.next()? {
        let api: String = row.get(0)?;
        let env: String = row.get(1)?;
        let value = (api, env);
        
        tuples.push(value);
    }

    Ok(tuples)
}