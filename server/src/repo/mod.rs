use rustbreak::{FileDatabase, deser::Ron};
use log::{info, debug, warn, error};
use std::collections::HashMap;
extern crate failure;

pub fn release(api: String, commit_id: String) -> Result<(), failure::Error> {
    info!("hop {:?}", api);

    let db = FileDatabase::<HashMap<String, String>, Ron>::from_path("/tmp/apis-catalog-store.ron", HashMap::new())?;

    println!("Writing to Database");
    db.write(|db| {
        db.insert(api, commit_id);
    });

    Ok(())
}