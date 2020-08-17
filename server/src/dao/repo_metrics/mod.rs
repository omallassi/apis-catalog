extern crate rusqlite;
extern crate yaml_rust;

use yaml_rust::{Yaml, YamlLoader};

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use rusqlite::NO_PARAMS;
use rusqlite::{params, Connection, Result};

use log::{debug, info, warn};

pub fn save_metrics_pull_requests_number(
    config: &super::super::settings::Database,
    datetime: DateTime<Utc>,
    size: i32,
) -> Result<()> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!(
            "Saving metrics_pr_num into Metrics_Database [{:?}]",
            db_path
        );
    }

    let conn = Connection::open(db_path)?;
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS metrics_pr_num (
    //         date_time TEXT NOT NULL UNIQUE,
    //         value INTEGER NOT NULL
    //     )",
    //     NO_PARAMS,
    // )?;

    conn.execute(
        "INSERT INTO metrics_pr_num (date_time, value) VALUES (?1, ?2)",
        params![datetime, size],
    )?;

    Ok(())
}

#[derive(Debug)]
pub struct TimeSeries {
    pub points: Vec<(DateTime<Utc>, i32)>,
}

pub fn get_metrics_pull_requests_number(
    config: &super::super::settings::Database,
) -> Result<TimeSeries> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!(
            "Reading all [pull_requests_number] metrics into Metrics_Database [{:?}]",
            db_path
        );
    }

    let conn = Connection::open(db_path)?;
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS metrics_pr_num (
    //         date_time TEXT NOT NULL UNIQUE,
    //         value INTEGER NOT NULL
    //     )",
    //     NO_PARAMS,
    // )?;

    let mut stmt = conn.prepare("SELECT date_time, value FROM metrics_pr_num")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut points = Vec::new();
    while let Some(row) = rows.next()? {
        let time = row.get("date_time")?;
        let val = row.get("value")?;

        points.push((time, val));
    }

    let timeseries = TimeSeries { points: points };

    Ok(timeseries)
}

pub fn save_metrics_pull_requests_ages(
    config: &super::super::settings::Database,
    datetime: DateTime<Utc>,
    p0: isize,
    p50: isize,
    p100: isize,
    mean: isize,
) -> Result<()> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!(
            "Saving metrics_pr_ages into Metrics_Database [{:?}]",
            db_path
        );
    }

    let conn = Connection::open(db_path)?;
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS metrics_pr_ages (
    //         date_time TEXT NOT NULL UNIQUE,
    //         p0 INTEGER NOT NULL,
    //         p50 INTEGER NOT NULL,
    //         p100 INTEGER NOT NULL,
    //         mean INTEGER NOT NULL
    //     )",
    //     NO_PARAMS,
    // )?;

    conn.execute(
        "INSERT INTO metrics_pr_ages (date_time, p0, p50, p100, mean) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![datetime, p0, p50, p100, mean],
    )?;

    Ok(())
}

#[derive(Debug)]
pub struct TupleTimeSeries {
    pub points: Vec<(DateTime<Utc>, i64, i64, i64, i64)>,
}

pub fn get_metrics_pull_requests_ages(
    config: &super::super::settings::Database,
) -> Result<TupleTimeSeries> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!(
            "Reading all [pull_requests_ages] metrics into Metrics_Database [{:?}]",
            db_path
        );
    }

    let conn = Connection::open(db_path)?;
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS metrics_pr_ages (
    //         date_time TEXT NOT NULL UNIQUE,
    //         p0 INTEGER NOT NULL,
    //         p50 INTEGER NOT NULL,
    //         p100 INTEGER NOT NULL,
    //         mean INTEGER NOT NULL
    //     )",
    //     NO_PARAMS,
    // )?;

    let mut stmt = conn.prepare("SELECT date_time, p0, p50, p100, mean FROM metrics_pr_ages")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut points = Vec::new();
    while let Some(row) = rows.next()? {
        points.push((
            row.get("date_time")?,
            row.get("p0")?,
            row.get("p50")?,
            row.get("p100")?,
            row.get("mean")?,
        ));
    }

    let timeseries = TupleTimeSeries { points: points };

    Ok(timeseries)
}

pub fn save_metrics_endpoints_num(
    config: &super::super::settings::Database,
    datetime: DateTime<Utc>,
    size: i32,
) -> Result<()> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!(
            "Saving metrics_endpoints_num into Metrics_Database [{:?}]",
            db_path
        );
    }

    let conn = Connection::open(db_path)?;
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS metrics_endpoints_num (
    //         date_time TEXT NOT NULL UNIQUE,
    //         value INTEGER NOT NULL
    //     )",
    //     NO_PARAMS,
    // )?;

    conn.execute(
        "INSERT INTO metrics_endpoints_num (date_time, value) VALUES (?1, ?2)",
        params![datetime, size],
    )?;

    Ok(())
}

pub fn get_metrics_endpoints_number(
    config: &super::super::settings::Database,
) -> Result<TimeSeries> {
    let mut db_path = String::from(&config.rusqlite_path);
    db_path.push_str("/apis-catalog-all.db");
    {
        debug!(
            "Reading all [metrics_endpoints_num] metrics into Metrics_Database [{:?}]",
            db_path
        );
    }

    let conn = Connection::open(db_path)?;
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS metrics_endpoints_num (
    //         date_time TEXT NOT NULL UNIQUE,
    //         value INTEGER NOT NULL
    //     )",
    //     NO_PARAMS,
    // )?;

    let mut stmt = conn.prepare("SELECT date_time, value FROM metrics_endpoints_num")?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut points = Vec::new();
    while let Some(row) = rows.next()? {
        let time = row.get("date_time")?;
        let val = row.get("value")?;

        points.push((time, val));
    }

    let timeseries = TimeSeries { points: points };

    Ok(timeseries)
}

pub fn get_zally_ignore_metrics(spec: &str) -> Result<std::collections::HashMap<i64, usize>> {
    debug!("get_zally_ignore_metrics is called");

    let docs = match YamlLoader::load_from_str(spec) {
        Ok(docs) => docs,
        Err(why) => {
            panic!("Error while parsing spec {} - :{:?}", spec, why);
        }
    }; // Result<Vec<Yaml>, ScanError>
    let doc = docs[0].as_hash().unwrap(); //Option<&Hash> et LinkedHashMap<Yaml, Yaml>;

    // let iter = doc.iter();
    // for item in iter {
    //     println!("---------");
    //     println!("{:?}", &item);
    //     println!("---------");
    // }
    let mut stats = std::collections::HashMap::new();
    //get global zally-ignore
    {
        match doc.get(&Yaml::String(String::from("x-zally-ignore"))) {
            Some(val) => {
                println!("x-zally-ignore {:?}", val);

                let paths = doc
                    .get(&Yaml::String(String::from("paths")))
                    .unwrap()
                    .as_hash()
                    .unwrap();

                for elt in val.as_vec().unwrap() {
                    stats.insert(elt.as_i64().unwrap(), paths.len());
                    // println!(
                    //     "x-zally-ignore {:?} {:?}",
                    //     elt.as_i64(),
                    //     elt.as_i64().unwrap()
                    // );
                    // println!("path len {:?}", paths.len());
                }
            }
            None => info!("no global zally-ignore for spec {:?}", spec),
        };
    }

    //get zally-ignore per path
    let mut stats_per_path: HashMap<i64, usize> = std::collections::HashMap::new();
    {
        let paths = doc
            .get(&Yaml::String(String::from("paths")))
            .unwrap()
            .as_hash()
            .unwrap();

        for path in paths.iter() {
            // println!("{:?}", path.0);
            // println!("{:?}", path.1);
            let zally = path
                .1
                .as_hash()
                .unwrap()
                .get(&Yaml::String(String::from("x-zally-ignore")));

            match zally {
                Some(val) => {
                    for elt in val.as_vec().unwrap() {
                        let stat = stats_per_path.get(&elt.as_i64().unwrap()).cloned();
                        match stat {
                            Some(val) => {
                                stats_per_path.insert(elt.as_i64().unwrap(), val + 1);
                            }
                            None => {
                                stats_per_path.insert(elt.as_i64().unwrap(), 1);
                            }
                        }
                        // println!(
                        //     "x-zally-ignore {:?} {:?}",
                        //     elt.as_i64(),
                        //     elt.as_i64().unwrap()
                        // );
                    }
                }
                None => {}
            }
        }
        // println!("stats_per_path {:?}", stats_per_path);
        // println!("len {:?}", paths.len());
    }

    //merge both maps
    for stat in stats_per_path.iter() {
        //check if stat already exist in global, if not add it to stats
        if stats.contains_key(stat.0) {
            debug!("stats {:?} already in global stats", stat.0);
        } else {
            stats.insert(*stat.0, *stat.1);
        }
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_get_zally_ignore_metrics_1() {
        let spec = "
        openapi: \"3.0.0\"
        info:
          version: 1.0.0
          title: an API ...
        x-zally-ignore:
          - 134
          - 120 # Rest maturity evolving
        
        paths:
          /v1/a/b:
            get:
              description: get ...
              responses:
                '200':
                  description: returns...

          /v2/a/b:
            x-zally-ignore:
              - 164
            get:
              description: get ...
              responses:
                '200':
                  description: returns...
                    
          /a/b:
            x-zally-ignore:
              - 164 # Rest maturity evolving
              - 134
            post:
              parameters:
                - name: chunk
                  in: query
                  required: true
                  schema:
                    type: integer
                    format: int32
                    minimum: 1
              responses:
                200:
                  description: ...     
        ";

        let results = super::get_zally_ignore_metrics(spec).unwrap();

        println!("*** results : {:?}", results);

        assert_eq!(results.get(&134i64).unwrap(), &3usize);
        assert_eq!(results.get(&120i64).unwrap(), &3usize);
        assert_eq!(results.get(&164i64).unwrap(), &2usize);
    }

    #[test]
    fn test_get_zally_ignore_metrics_2() {
        let spec = "
        openapi: \"3.0.0\"
        info:
          version: 1.0.0
          title: an API ...
        
        paths:
          /v1/a/b:
            get:
              description: get ...
              responses:
                '200':
                  description: returns...
                    
          /a/b:
            x-zally-ignore:
              - 164 # Rest maturity evolving
            post:
              parameters:
                - name: chunk
                  in: query
                  required: true
                  schema:
                    type: integer
                    format: int32
                    minimum: 1
              responses:
                200:
                  description: ...       
        ";

        let results = super::get_zally_ignore_metrics(spec).unwrap();

        println!("*** results : {:?}", results);

        assert_eq!(results.get(&164i64).unwrap(), &1usize);
    }
}
